# sc-hooks Architecture

## 1. Source Of Truth

This document describes the current architecture only.

- normative product behavior lives in `docs/requirements.md`
- host/plugin wire shapes live in `docs/protocol-contract.md`
- logging shapes live in `docs/logging-contract.md`
- missing or overstated behavior lives in `docs/implementation-gaps.md`

If a behavior is not present in code, this document shall not describe it as current architecture.

## 2. Current System Boundary

`sc-hooks` is a process-based hook dispatcher.

The host:
- loads `.sc-hooks/config.toml`
- resolves a hook chain
- assembles metadata
- validates plugin manifests and metadata requirements
- executes builtins in process
- executes plugins as child processes over stdin/stdout
- enforces timeout and session-disable policy
- writes JSONL log records

The host does not:
- load plugins as shared libraries
- expose a C ABI
- store handler-specific config inside the dispatcher config
- promise production-ready behavior for the reference plugin crates in `plugins/`

## 3. Crate Ownership

| Crate | Ownership |
| --- | --- |
| `sc-hooks-cli` | CLI commands, config loading, resolution, metadata assembly, dispatch, timeout handling, audit, install-plan generation, logging, exit behavior |
| `sc-hooks-core` | Shared data types for manifests, hook results, dispatch mode, events, validation rules, and exit codes |
| `sc-hooks-sdk` | Rust convenience helpers: manifest parsing/building, condition helpers, runner helpers, result helpers, and lightweight traits |
| `sc-hooks-test` | Reusable compliance harness and shell-plugin fixtures |

Important boundary:
- runtime plugin discovery uses `.sc-hooks/plugins/`
- source crates under `plugins/` are reference implementations in this repository, not the runtime discovery directory
- current source plugin inventory in `plugins/` is: `audit-logger`, `conditional-source`, `event-relay`, `guard-paths`, `identity-state`, `notify`, `policy-enforcer`, `save-context`, and `template-source`

## 3.1 Public Contract Vs Internal Typed Model

The public contract is not the Rust type graph.

Public contract:
- manifest JSON
- runtime stdin/stdout JSON
- environment variables for external plugin processes
- documented exit codes

Internal implementation detail:
- `FieldType`
- `ValidationRule`
- `DispatchMode`
- `HookAction`
- `ResolutionError`
- `ValidationError`
- `CliError`

The host uses those internal Rust types to implement the contract, but plugin authors do not depend on Rust typestate or enum names unless they choose to use `sc-hooks-sdk`.

## 4. Execution Model

### 4.1 Config And Resolution

1. `sc-hooks-cli` loads `.sc-hooks/config.toml`.
2. The requested hook and optional event are matched against the configured handler chain.
3. Builtins are resolved first.
4. Non-builtin handlers are resolved to `.sc-hooks/plugins/<name>`.
5. Plugin manifests are loaded and cached within the current invocation.
6. Matchers and payload conditions determine whether each plugin is eligible.

### 4.2 Metadata And Environment

Before plugin execution, the host assembles metadata from:
- runtime discovery: PID, working directory, Git repo root, Git branch
- selected environment variables: `SC_HOOK_AGENT_TYPE`, `SC_HOOK_SESSION_ID`
- `[context]` values from config
- the requested hook type and event
- the optional hook payload

The host then:
- writes metadata JSON to a temp file under the system temp directory
- exports `SC_HOOK_TYPE`
- exports `SC_HOOK_EVENT` when an event exists
- exports `SC_HOOK_METADATA` pointing at the temp file

The temp metadata file is created and owned by the host, not by the plugin. The plugin may read it as an ephemeral convenience artifact only. The file is removed automatically when dispatch scope exits.

### 4.3 Plugin Invocation

For each resolved plugin:

1. The host builds stdin JSON from the plugin manifest:
   - declared `requires` fields
   - declared `optional` fields when present
   - `hook`
   - `payload` only when supplied
2. The host spawns the plugin process.
3. The host writes the filtered JSON payload to stdin.
4. The host waits for completion or timeout.
5. The host reads stdout and stderr.
6. The host parses the first JSON object from stdout as a `HookResult`.

Failure handling:
- spawn failure disables the plugin for the session and fails the chain
- invalid JSON disables the plugin for the session and fails the chain
- non-zero exit disables the plugin for the session and fails the chain
- timeout disables the plugin for the session; sync dispatch fails, async dispatch logs and continues
- async `action=block` is treated as a protocol violation and disables the plugin

## 4.6 Error Hierarchy And Exit Mapping

Current CLI error layering is:

- `ResolutionError`
  - unresolved handler
  - manifest load / manifest compatibility failure during resolution
- `ValidationError`
  - missing required metadata field
  - invalid required metadata field
- `CliError`
  - wraps config, resolution, and validation errors
  - carries blocked, plugin, timeout, audit, and internal host failures

Exit-code mapping is intentionally coarse:
- resolution and manifest-load failures share exit code `4`
- metadata requirement failures alone use exit code `5`
- runtime plugin/protocol failures use exit code `2`

That coarser taxonomy is current architecture, not an accident in the docs.

### 4.4 Sync And Async Behavior

Sync mode:
- handlers run in order
- `block` short-circuits the chain
- `error` short-circuits the chain

Async mode:
- only async handlers run
- `additionalContext` values are concatenated with `\n---\n`
- `systemMessage` values are concatenated with `\n`
- async block attempts are treated as protocol errors
- timeout does not turn the async host invocation into a blocking failure

### 4.5 Session Disable State

Disabled plugin state is persisted at `.sc-hooks/state/session.json`, keyed by session ID.

Current behavior:
- a missing or unreadable state file is fail-open
- `SessionEnd` clears state for the current session
- `sc-hooks audit --reset` clears all stored session state

## 5. Logging Architecture

There are currently two log record shapes written to the configured hook log path:

1. builtin log records from `builtins::log::write_entry()`
2. dispatch records from `logging::append_dispatch_log()`

This mixed-schema reality is intentional current behavior and is documented exactly in `docs/logging-contract.md`.

## 5.1 Planned sc-observability Boundary

Before the next logging implementation expansion, `schook` should adopt the same boundary pattern used in `scterm`:

- use the sibling `sc-observability` workspace at `../sc-observability` for logging integration
- use the logging-focused `sc-observability` crate only in the initial adoption
- do not adopt higher-layer crates from the sibling `sc-observability` workspace in the first pass
- keep logger lifecycle, sink configuration, and initialization in `sc-hooks-cli` or the final binary wiring
- keep `sc-hooks-core`, `sc-hooks-sdk`, and `sc-hooks-test` logging-implementation-agnostic
- keep lower crates focused on typed data, typed errors, and return values rather than logger ownership

This is not current implementation. It is the required ownership boundary for the next logging integration pass before broader observability work starts.

## 6. Current Extension Points

### 6.1 Supported Today

- custom plugins implemented as executables
- manifest-declared metadata requirements
- payload-condition filtering
- sandbox declarations validated by audit
- Codex and Gemini shell shims

### 6.2 Deferred Or Unstable

- SDK-level `LongRunning` ergonomics beyond the host's manifest handling
- release-grade bundled plugins
- stronger compliance-harness coverage that proves the entire documented release contract
- a more granular exit-code split for manifest compatibility vs other resolution failures

These items are not part of the current mainline architecture contract and must remain documented as gaps or deferred work.

## 7. Non-Goals

The current architecture does not aim to provide:
- dynamic library loading
- plugin hot reloading
- plugin marketplace/distribution
- merged editing of existing `.claude/settings.json` content
- a single normalized log schema with a discriminant field

## 8. Enforcement Notes

- Builtins win name resolution because resolution checks them before plugin paths.
- Plugins remain processes because dispatch always shells out to executables.
- JSON remains the only host/plugin contract because manifests and runtime results are serialized through serde-backed JSON.
- Crate boundaries remain narrow because `sc-hooks-core` carries shared data only, `sc-hooks-sdk` is convenience code, and `sc-hooks-cli` owns orchestration.
