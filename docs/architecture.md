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
| `plugins/agent-session-foundation` | Runtime-implementation source crate for session lifecycle persistence keyed by `session_id` and normalized `agent_state` transitions |
| `plugins/agent-spawn-gates` | Runtime-implementation source crate for `PreToolUse(Agent)` policy checks and subagent linkage writes |
| `plugins/tool-output-gates` | Runtime-implementation source crate for `PostToolUse(Bash)` fenced-JSON validation and retryable block responses |
| `plugins/atm-extension` | Runtime-implementation source crate for ATM-specific identity-file handling and relay enrichment |

Important boundary:
- runtime plugin discovery uses `.sc-hooks/plugins/`
- the checked contributor example for that runtime shape lives at `examples/runtime-layout/.sc-hooks/`
- source crates under `plugins/` are repository-owned implementations, not the runtime discovery directory itself
- current source plugin inventory in `plugins/` is: `agent-session-foundation`, `agent-spawn-gates`, `atm-extension`, `audit-logger`, `conditional-source`, `event-relay`, `guard-paths`, `identity-state`, `notify`, `policy-enforcer`, `save-context`, `template-source`, and `tool-output-gates`
- `agent-session-foundation`, `agent-spawn-gates`, `tool-output-gates`, and `atm-extension` are the current runtime-implementation crates in this branch
- the remaining source crates under `plugins/` stay scaffold/reference only in the current release baseline
- planning docs may still refer to the session lifecycle package as `sc-hooks-session-foundation`; the current source crate name remains `plugins/agent-session-foundation` until install/package naming is finalized

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

The OBS-007/OBS-008 violation corrected in this pass was:
- `default_logger_config()` and env-flag sink routing had drifted into
  `sc-hooks-core`
- `plugins/agent-session-foundation` had picked up direct
  `sc-observability` dependencies and its own logger creation path

Current restored boundary:
- logger config and sink lifecycle live in `sc-hooks-cli`
- `sc-hooks-core` keeps only shared path-resolution helpers/constants used for
  agreement on file locations
- scaffold/reference plugin crates do not depend on `sc-observability`

Current post-file-sink expansion status:

- the file-sink JSONL contract remains the release baseline
- console-sink verification is now implemented through the same real
  `sc-hooks-cli` dispatch path used by the file-sink contract tests
- console-sink coverage is the first operator-facing debugging expansion
  because it is the most useful immediate surface for live multi-agent and
  background-agent monitoring
- custom sink registration coverage and multi-hook monitoring correlation remain
  deferred after the console-sink contract

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

The detailed post-capture design authority for this track lives in
`docs/phase-bc-hook-runtime-design.md`.

### 9.1 Claude-First Development Gate

The first hook-extension development path is:

1. build a Claude-focused schema harness under `test-harness/hooks/`
2. capture and validate real Claude hook payloads
3. build and QA the global HTML reporting stack
4. revise hook docs and the implementation plan from captured evidence
5. implement the Claude ATM hook crates

Current status:

- steps 1-2 are complete for the Claude baseline
- post-capture doc and plan revision from captured evidence is complete for the
  current Claude baseline
- the global HTML reporting stack is now an explicit prerequisite before
  schema-drift/report-generating work can close
- captured `SessionStart.source` values now include `startup`, `compact`,
  `resume`, and `clear`
- `Notification(idle_prompt)` remains part of the documented Claude surface, but
  is currently wired-but-unresolved in local Haiku capture

### 9.2 Planned Harness Subsystem

The planned hook harness owns:

- provider launch adapters
- captured raw fixtures
- provider-specific validation models
- manual schema-drift detection tooling
- review artifacts for newly observed or changed payload fields
- integration with the global HTML reporting stack for self-contained reports

Initial execution scope:

- Claude only

Documented but deferred from the first harness pass:

- Codex
- Gemini
- Cursor Agent

### 9.3 Implemented Hook Runtime Targets In This Branch

The current runtime-implementation path in this branch includes:

- `sc-hooks-session-foundation` via `plugins/agent-session-foundation`
- `sc-hooks-agent-spawn-gates` via `plugins/agent-spawn-gates`
- `sc-hooks-tool-output-gates` via `plugins/tool-output-gates`
- `sc-hooks-atm-extension` via `plugins/atm-extension`

All four hook-runtime targets now exist as current source crates in this branch.

Current architecture guardrails for these targets in this branch:

- ATM-specific behavior remains isolated in `docs/hook-api/atm-hook-extension.md`
- the generic implementation baseline remains the Claude hook API doc plus the
  captured Claude fixtures
- the detailed post-capture BC design in
  `docs/phase-bc-hook-runtime-design.md` is authoritative for crate roles,
  state ownership, and trait boundaries
- `sc-hooks-session-foundation` is responsible for the canonical session-state
  record keyed by `session_id`, `active_pid`, and `ai_root_dir`
- `ai_root_dir` is the immutable working directory established from the
  root-establishing `SessionStart` for the runtime instance and must not be
  rewritten from later `cwd` drift
- `ai_current_dir` is captured from each lifecycle payload `cwd` as current
  working-directory context, not as root identity
- inbound `CLAUDE_PROJECT_DIR`, when present, is a required equality check
  against the persisted canonical root rather than the source of truth
- any divergence between the persisted canonical root and inbound
  `CLAUDE_PROJECT_DIR` must emit prominent error-level observability for
  investigation
- downstream hook consumers receive normalized project-root context from the
  persisted canonical session root even when Claude omits or varies raw env
  values across hook surfaces
- canonical `session.json` updates use atomic write semantics and skip unchanged
  rewrites while still emitting hook logs
- the internal in-process hook trait remains sealed in `sc-hooks-core`, while
  the public executable-plugin traits in `sc-hooks-sdk::traits` remain
  intentionally unsealed for sibling workspace crates; this deviation from the
  original BC sealed-trait assumption is tracked in
  `docs/implementation-gaps.md`
- the earlier hook trait-freeze gate is treated as satisfied through the
  executable-plugin JSON schema contract rather than Rust trait sealing at the
  SDK boundary; see `SEAL-001` in `docs/implementation-gaps.md`
- legacy prototype names (`atm-session-lifecycle`, `atm-bash-identity`,
  `gate-agent-spawns`, `atm-state-relay`) are retired planning names and are
  not the clean design authority
- every hook runtime crate listed above is current architecture in this branch
  because it now lands with code, tests, and the same-PR
  `docs/architecture.md` crate inventory update

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
