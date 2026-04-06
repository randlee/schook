# sc-hooks Architecture

## 1. Source Of Truth

This document describes the current architecture only.

- normative product behavior lives in `docs/requirements.md`
- host/plugin wire shapes live in `docs/protocol-contract.md`
- observability event shapes live in `docs/observability-contract.md`
- current JSONL dispatch-log consumer contract lives in `docs/logging-contract.md`
- archived gap and sprint-planning artifacts live in `docs/archive/`

Crate-local ownership detail now lives in:

- `docs/sc-hooks-cli/`
- `docs/sc-hooks-core/`
- `docs/sc-hooks-sdk/`

If a behavior is not present in code, this document shall not describe it as current architecture.

## 1.1 Stable Product ADR IDs

Top-level architectural decisions use stable `ADR-SHK-*` identifiers.

| ADR ID | Decision |
| --- | --- |
| `ADR-SHK-001` | `sc-hooks` remains a process-based hook dispatcher rather than an in-process plugin runtime. |
| `ADR-SHK-002` | The public contract is JSON, environment variables, and documented exit codes; internal Rust enums and typestates are implementation detail. |
| `ADR-SHK-003` | `sc-hooks-cli` is the only workspace crate that owns observability sink setup and emission. |
| `ADR-SHK-004` | `sc-hooks-sdk` is an authoring convenience layer and does not define the release contract on its own. |
| `ADR-SHK-005` | Top-level docs remain product-level and cross-cutting; crate-local ownership detail belongs in crate doc subdirectories. |

Crate-local ADR delegation:
- crate-local `ADR-SHK-CLI-*`, `ADR-SHK-CORE-*`, and `ADR-SHK-SDK-*` IDs are
  defined in the crate architecture docs under `docs/sc-hooks-cli/`,
  `docs/sc-hooks-core/`, and `docs/sc-hooks-sdk/`
- those crate-local ADRs are subordinate to the product-level `ADR-SHK-001`
  through `ADR-SHK-005` decisions in this document

## 2. Current System Boundary

`sc-hooks` is a process-based hook dispatcher.

The host:
- loads `.sc-hooks/config.toml`
- merges observability defaults from `~/.sc-hooks/config.toml`, repo-local
  `.sc-hooks/config.toml`, and supported environment overrides
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
- expose a public sink-extension API, exporter/OTel transport config, or a
  `[logging]` section outside the supported `[observability]` surface
- promise production-ready behavior for the reference plugin crates in `plugins/`

## 3. Crate Ownership

### 3.1 Workspace Crates

| Path | Ownership |
| --- | --- |
| `crates/sc-hooks-cli` | CLI commands, config loading, resolution, metadata assembly, dispatch, timeout handling, audit, install-plan generation, `sc-observability` integration, exit behavior |
| `crates/sc-hooks-core` | Shared data types for manifests, hook results, dispatch mode, events, validation rules, and exit codes |
| `crates/sc-hooks-sdk` | Rust convenience helpers: manifest parsing/building, condition helpers, runner helpers, and result helpers; this crate is an authoring aid, not the release-defining public contract |
| `crates/sc-hooks-test` | Reusable compliance harness and shell-plugin fixtures; tracked in the release manifest for validation, not published to crates.io |

Important boundary:
- runtime plugin discovery uses `.sc-hooks/plugins/`
- the checked contributor example for that runtime shape lives at `examples/runtime-layout/.sc-hooks/`
- source crates under `plugins/` are source-owned implementation or scaffold/reference crates in this repository, not the runtime discovery directory
- the initial publish scope covers only the complete working crates under `crates/`: `sc-hooks-core`, `sc-hooks-sdk`, and `sc-hooks-cli`; `sc-hooks-test` remains tracked but unpublished, and no `plugins/` source crate is part of the first crates.io release
- crate-owned boundary detail for the host, core types, and SDK helpers lives in the crate architecture docs under `docs/sc-hooks-cli/`, `docs/sc-hooks-core/`, and `docs/sc-hooks-sdk/`

### 3.2 Plugin Source Crates

| Path | Classification | Notes |
| --- | --- | --- |
| `plugins/agent-session-foundation` | Scaffold/reference | Planned hook-extension target; not part of the current release scope |
| `plugins/agent-spawn-gates` | Scaffold/reference | Planned hook-extension target; not part of the current release scope |
| `plugins/atm-extension` | Scaffold/reference | Planned hook-extension target; not part of the current release scope |
| `plugins/tool-output-gates` | Scaffold/reference | Planned hook-extension target; not part of the current release scope |
| `plugins/audit-logger` | Scaffold/reference | Source-owned scaffold/reference crate; not part of the initial crates.io release |
| `plugins/conditional-source` | Scaffold/reference | Source-owned scaffold/reference crate; not part of the initial crates.io release |
| `plugins/event-relay` | Scaffold/reference | Source-owned scaffold/reference crate; not part of the initial crates.io release |
| `plugins/guard-paths` | Scaffold/reference | Source-owned scaffold/reference crate; not part of the initial crates.io release |
| `plugins/identity-state` | Scaffold/reference | Source-owned scaffold/reference crate; not part of the initial crates.io release |
| `plugins/notify` | Scaffold/reference | Source-owned scaffold/reference crate; not part of the initial crates.io release |
| `plugins/policy-enforcer` | Scaffold/reference | Source-owned scaffold/reference crate; not part of the initial crates.io release |
| `plugins/save-context` | Scaffold/reference | Source-owned scaffold/reference crate; not part of the initial crates.io release |
| `plugins/template-source` | Scaffold/reference | Source-owned scaffold/reference crate; not part of the initial crates.io release |

## 3.3 Public Contract Vs Internal Typed Model

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

1. `sc-hooks-cli` loads repo-local `.sc-hooks/config.toml`, merges supported
   observability defaults from `~/.sc-hooks/config.toml`, and then applies the
   supported environment overrides.
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
- `sc-hooks-cli` also owns layered `[observability]` config loading,
  `off | standard | full` mode resolution, and sink-selection policy
- the implementation uses the external `sc-observability` workspace referenced by `sc-hooks-cli/Cargo.toml` at `../../../sc-observability/...`
- `sc-hooks-core`, `sc-hooks-sdk`, and `sc-hooks-test` remain observability-implementation-agnostic
- the current file sink path is `.sc-hooks/observability/sc-hooks/logs/sc-hooks.log.jsonl`
- dispatch outcomes are emitted as `LogEvent` JSONL records, not as ad hoc dispatcher-specific record envelopes
- the current config surface is `[observability]`, not `[logging]`
- `off` suppresses durable structured sink emission while leaving direct
  stderr warnings and degraded notices visible to the operator
- when `standard` mode is active and a pre-dispatch failure prevents
  `dispatch.complete`, `sc-hooks-cli` emits a deterministic degraded stderr
  signal instead of silently losing observability for that runtime attempt
- when `full` mode is active, `sc-hooks-cli` also writes run-scoped audit files
  under `.sc-hooks/audit/runs/<run-id>/`

Next planned observability expansion:

- keep the current file-sink JSONL contract as the release baseline and the
  baseline operational mode
- keep the new lean full-audit sink as the durable machine-readable source for
  audit-grade runs
- complete the remaining observability phase with debug-profile fields,
  redaction hardening, retention, and 50-agent hardening
- keep durable audit JSONL as the canonical machine-readable source for the
  committed phase; the human console sink is operator-facing only
- treat structured live streaming plus exporter, spans, metrics, and OTLP work
  as explicit follow-on scope rather than observability-phase acceptance gates

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
- spans, metrics, or OTLP export in the current `sc-hooks` host

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

### 9.2a Planned Version-Bump Detection Boundary

The hook harness must also track which AI CLI version produced the latest
approved schema-drift artifacts.

The planned boundary is:

- `scripts/verify-claude-hook-api.py` is a harness-side verification tool, not
  a runtime dispatcher component
- the detector reads the approved Claude manifest at
  `test-harness/hooks/claude/fixtures/approved/manifest.json`
- the detector compares `claude --version` output with the manifest's
  `claude_version`
- a version mismatch is a release-process signal to rerun the live hook-schema
  validation flow before accepting provider-contract changes

Extensibility rule:

- if other providers later need the same guardrail, the design must be revisited
  explicitly rather than inferred from a premature multi-provider detector

### 9.3 Planned Hook Crate Targets

These are planned hook-extension targets only. They are not current source
inventory and are not current runtime crates.

The post-capture intended split is:

- generic hook utility layer
  - session lifecycle / session-record persistence
  - normalized agent-state tracking
  - subagent linkage and spawn policy
  - tool/blocking/fenced-JSON guard behavior
- ATM extension layer
  - ATM routing enrichment
  - temp identity-file behavior for `atm` Bash calls
  - teammate-idle / ATM relay emission behavior

Recommended planned crate targets:

- `plugins/agent-session-foundation`
- `plugins/agent-spawn-gates`
- `plugins/tool-output-gates`
- `plugins/atm-extension`

Planned responsibility split:

- `plugins/agent-session-foundation`
  - owns the canonical session-state file
  - owns `SessionStart`, `SessionEnd`, and `PreCompact`
  - owns normalized `agent_state` transitions
- `plugins/agent-spawn-gates`
  - owns named-agent vs background-agent policy
  - owns subagent linkage/tracking
  - owns schema-governed fenced-JSON spawn validation
- `plugins/tool-output-gates`
  - owns generic blocking/fenced-JSON tool-output policy
- `plugins/atm-extension`
  - owns ATM routing enrichment on the same session-state record
  - owns ATM identity-file behavior for Bash `atm` calls
  - owns ATM relay emission and teammate-idle mapping

Planned shared session-state schema rules:

- one canonical session-state file per `session_id`
- required base fields:
  - `session_id`
  - `active_pid`
  - `agent_state`
  - `created_at`
  - `updated_at`
  - `ai_root_dir`
  - `ai_current_dir`
- optional ATM fields live in an extension object on the same file rather than
  a second authoritative ATM-only file
- `session_id`, `active_pid`, and hook event identifiers should be represented
  as semantic newtypes in implementation code rather than bare primitives

Planned trait-freeze rule before the first runtime crate lands:

- `sc-hooks-core` / `sc-hooks-sdk` must freeze a hook trait that exposes:
  - normalized context
  - raw provider payload
  - typed result / failure posture
  - fail-open versus fail-closed semantics per hook class
- the frozen hook trait in `sc-hooks-core` shall be sealed (private supertrait
  or mod-private pattern) so that only `sc-hooks-sdk` can provide base
  implementations. Unsealed traits permit external plugin crates to bypass
  normalized-context and fail-open/fail-closed invariants; retrofitting a seal
  after downstream adoption is a breaking API change.
- runtime crates must not define their own competing hook trait surfaces
- `agent_state` remains a runtime enum rather than typestate because hook state
  persists across process boundaries and must round-trip through the canonical
  session-state file

Archived prototype crates remain reference-only inputs for design review:

- `plugins/atm-session-lifecycle`
- `plugins/atm-bash-identity`
- `plugins/gate-agent-spawns`
- `plugins/atm-state-relay`

Planning rules for these targets:

- ATM-specific behavior remains isolated in `docs/hook-api/atm-hook-extension.md`
- the generic implementation baseline remains the Claude hook API doc plus the
  captured Claude fixtures
- these planned targets are not part of the current §3 source inventory
  (`BND-001a`) and will not appear there until they land with code, tests, and
  a same-PR architecture inventory update
- no planned hook crate becomes current architecture until it lands with code,
  tests, and the same-PR `docs/architecture.md` crate inventory update
- archived prototype crates do not define the final crate split; they are
  reviewed only as reference against the post-capture design

Planned fail posture by crate:

| Planned crate | Default posture | Reason |
| --- | --- | --- |
| `plugins/agent-session-foundation` | fail-open | session persistence loss should not prevent the host from continuing a Claude run |
| `plugins/agent-spawn-gates` | fail-closed | malformed or policy-breaking subagent launches must be blocked deterministically |
| `plugins/tool-output-gates` | fail-closed | fenced-JSON and blocking-output violations must stop the tool result before it reaches the caller |
| `plugins/atm-extension` | fail-open | ATM routing enrichment should not make the generic hook host unusable when ATM context is absent or degraded |

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

## 10. Observability Phase Planning Boundary

The next observability phase is planned work only. It is not current
architecture until code and contract docs land together.

Planning direction frozen for that phase:

- naming converges on `sc-hooks` as the canonical product/runtime/binary name
  with `hooks` as a convenience CLI alias
- filesystem/config namespace remains `.sc-hooks/`
- observability config becomes layered:
  - built-in defaults
  - global user config at `~/.sc-hooks/config.toml`
  - repo-local config at `.sc-hooks/config.toml`
  - environment overrides for temporary operator control
- observability modes are planned as:
  - `off`
  - `standard`
  - `full`
- global config may set defaults and future exporter wiring, but does not
  enable `full` audit by itself
- repo-local config owns plugin-specific settings, repo-specific observability
  policy, and `full` audit activation
- `full` audit uses durable file output under `.sc-hooks/audit/` by default,
  with run-scoped files rather than one shared hot file
- any future machine-readable live stream is a separate structured sink, not
  the current human console renderer
- audit and observability failures remain non-blocking for hook execution
- the scale target for the phase is production readiness with at least 50
  simultaneous agents on the same repo root without log corruption or
  unbounded contention

Detailed sequencing for that phase lives in
`docs/phase-observability-plan.md` and `docs/project-plan.md`.
