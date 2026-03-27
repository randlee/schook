# sc-hooks Architecture

## 1. Source Of Truth

This document describes the current architecture only.

- normative product behavior lives in `docs/requirements.md`
- host/plugin wire shapes live in `docs/protocol-contract.md`
- observability event shapes live in `docs/observability-contract.md`
- current JSONL dispatch-log consumer contract lives in `docs/logging-contract.md`
- missing or overstated behavior lives in `docs/implementation-gaps.md`

If a behavior is not present in code, this document shall not describe it as current architecture.

## 2. Current System Boundary

`sc-hooks` is a process-based hook dispatcher.

The host:
- loads `.sc-hooks/config.toml`
- resolves a hook chain
- assembles metadata
- validates plugin manifests and metadata requirements
- executes plugins as child processes over stdin/stdout
- enforces timeout and session-disable policy
- emits service-scoped `sc-observability` JSONL events

The host does not:
- load plugins as shared libraries
- expose a C ABI
- store handler-specific config inside the dispatcher config
- resolve builtin handlers inside the dispatcher; any future builtin path is deferred
- expose config-driven observability sink routing or a `[logging]` section in `.sc-hooks/config.toml`
- promise production-ready behavior for the reference plugin crates in `plugins/`

## 3. Crate Ownership

| Crate | Ownership |
| --- | --- |
| `sc-hooks-cli` | CLI commands, config loading, resolution, metadata assembly, dispatch, timeout handling, audit, install-plan generation, `sc-observability` integration, exit behavior |
| `sc-hooks-core` | Shared data types for manifests, hook results, dispatch mode, events, validation rules, and exit codes |
| `sc-hooks-sdk` | Rust convenience helpers: manifest parsing/building, condition helpers, runner helpers, and result helpers; this crate is an authoring aid, not the release-defining public contract |
| `sc-hooks-test` | Reusable compliance harness and shell-plugin fixtures |

Important boundary:
- runtime plugin discovery uses `.sc-hooks/plugins/`
- the checked contributor example for that runtime shape lives at `examples/runtime-layout/.sc-hooks/`
- source crates under `plugins/` are reference implementations in this repository, not the runtime discovery directory
- current source plugin inventory in `plugins/` is: `atm-session-lifecycle`, `audit-logger`, `conditional-source`, `event-relay`, `guard-paths`, `identity-state`, `notify`, `policy-enforcer`, `save-context`, and `template-source`
- `plugins/atm-session-lifecycle` is the current Claude lifecycle implementation source crate for `SessionStart` / `SessionEnd`
- the older reference crates under `plugins/` remain scaffold/reference only; source presence alone does not make any plugin a runtime discovery entry or a shipped release plugin

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

Important SDK boundary:
- `sc-hooks-sdk` may offer authoring conveniences that are broader than the host's guaranteed runtime contract
- runner-helper behavior such as empty-stdin fallback is convenience behavior, not a statement that the host omits required runtime fields during real dispatch
- if SDK helpers and the documented executable/JSON contract diverge, the contract docs and host behavior win

## 4. Execution Model

### 4.1 Config And Resolution

1. `sc-hooks-cli` loads `.sc-hooks/config.toml`.
2. The requested hook and optional event are matched against the configured handler chain.
3. Handlers are resolved to `.sc-hooks/plugins/<name>`.
4. Plugin manifests are loaded and cached within the current invocation.
5. Matchers and payload conditions determine whether each plugin is eligible.

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
- timeout disables the plugin for the session; sync dispatch fails, async dispatch records the failure and continues
- async `action=block` is treated as a protocol violation and disables the plugin
- async manifests using `long_running=true` are rejected during manifest validation and resolution

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
- `long_running=true` removes the default sync timeout when no explicit `timeout_ms` override is set

Async mode:
- only async handlers run
- `additionalContext` values are concatenated with `\n---\n`
- `systemMessage` values are concatenated with `\n`
- async block attempts are treated as protocol errors
- timeout does not turn the async host invocation into a blocking failure
- `long_running=true` is not part of the valid async manifest contract

### 4.5 Session Disable State

Disabled plugin state is persisted at `.sc-hooks/state/session.json`, keyed by session ID.

Current behavior:
- a missing or unreadable state file is fail-open
- `SessionEnd` clears state for the current session
- `sc-hooks audit --reset` clears all stored session state

## 5. Observability Architecture

Current observability ownership follows the intended boundary directly:

- `sc-hooks-cli` owns logger creation, emission, flush, and shutdown
- the implementation uses the external `sc-observability` workspace referenced by `sc-hooks-cli/Cargo.toml` at `../../../sc-observability/...`
- `sc-hooks-core`, `sc-hooks-sdk`, and `sc-hooks-test` remain observability-implementation-agnostic
- the current file sink path is `.sc-hooks/observability/sc-hooks/logs/sc-hooks.log.jsonl`
- dispatch outcomes are emitted as `LogEvent` JSONL records, not as ad hoc dispatcher-specific record envelopes
- there is no `[logging]` config section; observability sink routing is fixed by the current CLI boundary

This boundary is current architecture, not deferred intent.

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
- promotion of any `plugins/` source crate to shipped runtime behavior without install guidance and direct behavior tests
- a more granular exit-code split for manifest compatibility vs other resolution failures

These items are not part of the current mainline architecture contract and must remain documented as gaps or deferred work.

## 7. Non-Goals

The current architecture does not aim to provide:
- dynamic library loading
- plugin hot reloading
- plugin marketplace/distribution
- merged editing of existing `.claude/settings.json` content
- spans, metrics, or OTLP export in the current `schook` host

## 8. Enforcement Notes

- Handler names resolve only through `.sc-hooks/plugins/`; the runtime has no builtin resolution path.
- Plugins remain processes because dispatch always shells out to executables.
- JSON remains the only host/plugin contract because manifests and runtime results are serialized through serde-backed JSON.
- Crate boundaries remain narrow because `sc-hooks-core` carries shared data only, `sc-hooks-sdk` is convenience code, and `sc-hooks-cli` owns orchestration.

## 9. Hook Extension Planning Boundary

The next hook-extension track is a planned architecture addition, not part of
the current release implementation boundary above.

### 9.1 Claude-First Development Gate

The first hook-extension development path is:

1. build a Claude-focused schema harness under `test-harness/hooks/`
2. capture and validate real Claude hook payloads
3. revise hook docs and the implementation plan from captured evidence
4. implement the Claude ATM hook crates

Until steps 1-3 are complete, hook runtime crates remain planning targets only.

### 9.2 Planned Harness Subsystem

The planned hook harness owns:

- provider launch adapters
- captured raw fixtures
- provider-specific validation models
- schema-drift CI checks
- review artifacts for newly observed or changed payload fields

Initial execution scope:

- Claude only

Documented but deferred from the first harness pass:

- Codex
- Gemini
- Cursor Agent

### 9.3 Planned Hook Crate Targets

These are planned hook-extension targets only. They are not current source
inventory and are not current runtime crates:

- `plugins/atm-bash-identity`
- `plugins/gate-agent-spawns`
- `plugins/atm-state-relay`

Planning rules for these targets:

- ATM-specific behavior remains isolated in `docs/hook-api/atm-hook-extension.md`
- the generic implementation baseline remains the Claude hook API doc plus the
  captured Claude fixtures
- no planned hook crate becomes current architecture until it lands with code,
  tests, and the same-PR `docs/architecture.md` crate inventory update

### 9.4 Cursor Follow-On Boundary

Cursor Agent is documented in `docs/hook-api/cursor-agent-hook-api.md`, but the
current architecture does not yet include:

- Cursor harness capture
- Cursor-targeting runtime crates
- Cursor hook payloads as an implementation dependency

Planning targets only for a later approved Cursor pass:

- `plugins/cursor-agent-gates`
- `plugins/cursor-agent-relay`

Those remain later follow-on work after the Claude ATM baseline is captured,
reviewed, revised, and implemented.
