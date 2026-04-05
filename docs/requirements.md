# sc-hooks Requirements

## 1. Purpose

This document defines the release-facing behavior for `sc-hooks` as it exists today or must exist before release. It is intentionally narrower than the prior draft: if the code does not implement a behavior and the release does not require it, the behavior is deferred or tracked as a gap.

## 2. Status Model

| Status | Meaning |
| --- | --- |
| `Implemented` | Backed by current code and direct tests, or by code plus obvious mechanical proof |
| `Required Before Release` | Intended release behavior that is not yet proved cleanly enough by code, tests, or contracts |
| `Planned` | Committed phase work that is not implemented yet but is required for the approved next phase to close |
| `Deferred` | Explicitly out of the current release and approved next-phase baseline |
| `Superseded` | Older requirement text retired in favor of a newer requirement or contract amendment |

## 3. Release Baseline

Current release scope is the host dispatcher foundation:
- config parsing from `.sc-hooks/config.toml`
- hook routing to external plugins
- manifest loading and metadata filtering
- sync/async dispatch with timeouts and per-session disable state
- Claude settings generation from matchers
- audit, diagnostic fire, compliance-test entry points, and exit-code reporting
- `sc-observability` JSONL dispatch events

Current release scope does not include:
- shipped runtime plugin behavior from the scaffold/reference crates under `plugins/`; all source crates under `plugins/` remain outside the current release scope
- a stable end-to-end `LongRunning` SDK surface beyond the manifest fields the host already enforces
- builtin handler resolution inside the dispatcher
- config-driven observability sink routing or a `[logging]` config section

## 4. Functional Requirements

### 4.1 Configuration

| ID | Status | Priority | Requirement | Acceptance Scenario |
| --- | --- | --- | --- | --- |
| CFG-001 | Implemented | Must | The host shall load its default config from `.sc-hooks/config.toml` relative to the current repository. | `sc-hooks config` reads the default path through `load_default_config()`. |
| CFG-002 | Implemented | Must | The config shall recognize exactly `[meta]`, `[context]`, `[hooks]`, and `[sandbox]`; only `[meta]` and `[hooks]` are required. | Unknown top-level sections fail parsing. |
| CFG-003 | Implemented | Must | `[hooks]` shall map hook names to ordered handler arrays. | Resolution and dispatch preserve config order. |
| CFG-004 | Implemented | Must | `[context] team = "<name>"` shall map to `metadata.team.name`; other context keys remain top-level metadata fields. | `map_context_to_metadata()` applies the special-case mapping only for `team`. |
| CFG-008 | Implemented | Should | `[sandbox]` shall allow per-plugin network and path overrides for audit validation. | `SandboxConfig` exposes `allow_network` and `allow_paths`. |

### 4.2 Resolution And Matching

| ID | Status | Priority | Requirement | Acceptance Scenario |
| --- | --- | --- | --- | --- |
| RES-001 | Implemented | Must | Handler resolution shall treat configured handler names as external plugin executables under `.sc-hooks/plugins/`; there are no reserved builtin handler names in the current runtime. | A plugin named `log` resolves like any other plugin executable. |
| RES-002 | Implemented | Must | Runtime plugin executables shall be resolved from `.sc-hooks/plugins/`. | Resolution and handler discovery compute plugin paths under `.sc-hooks/plugins/`. |
| RES-003 | Implemented | Must | Unresolvable handlers shall fail both runtime and audit. | Missing plugin paths produce audit errors and runtime resolution failures. |
| MTR-001 | Implemented | Must | `sc-hooks install` shall generate matcher entries from plugin-declared matchers instead of blanket wildcard routing. | Install output is matcher-specific and omits empty matcher/mode combinations. |
| PLC-001 | Implemented | Must | Payload conditions shall be evaluated after hook/event matching and before plugin spawn; a non-match silently skips the plugin. | Condition failures do not produce spawn attempts or errors. |
| PLC-002 | Implemented | Must | The host shall support `exists`, `not_exists`, `equals`, `not_equals`, `contains`, `starts_with`, `matches`, `one_of`, and `regex` payload-condition operators. | The SDK condition evaluator accepts and evaluates those operators. |

### 4.3 Host/Plugin Contract

| ID | Status | Priority | Requirement | Acceptance Scenario |
| --- | --- | --- | --- | --- |
| PLG-001 | Implemented | Must | A plugin shall answer `--manifest` by emitting manifest JSON to stdout and exiting successfully. | Manifest loading shells out to the executable with `--manifest`. |
| PLG-002 | Implemented | Must | Manifest fields shall include `contract_version`, `name`, `mode`, `hooks`, `matchers`, and `requires`; optional fields may include `optional`, `payload_conditions`, `timeout_ms`, `long_running`, `response_time`, `sandbox`, and `description`. | `sc_hooks_core::manifest::Manifest` and `validate_manifest()` define the current schema. |
| PLG-003 | Implemented | Must | Manifest validation rules shall support `non_empty`, `dir_exists`, `file_exists`, `path_resolves`, `positive_int`, and `one_of:<values>`. | Validation rule parsing accepts those rule names. |
| PLG-004 | Implemented | Must | Before spawning a plugin, the host shall build stdin JSON from declared metadata fields only, always include `hook`, and include `payload` only when a payload exists. | `build_plugin_input()` filters metadata and omits absent payloads. |
| PLG-006 | Implemented | Must | Plugin runtime output shall be a `HookResult` JSON object whose `action` is `proceed`, `block`, or `error`. | Dispatch parses `HookResult` from stdout. |
| PLG-009 | Implemented | Must | Async-mode plugins shall not block the chain. A runtime `action=block` from an async plugin is a protocol error that disables the plugin. | Async block returns trigger `async_block` handling in dispatch. |
| PLG-011 | Implemented | Must | The host shall reject plugins whose `contract_version` is greater than the host contract version. | Manifest compatibility is enforced during manifest validation. |
| PLG-012 | Implemented | Must | If no hook payload exists, the plugin input shall omit `payload` rather than sending `null` or `{}`. | `build_plugin_input()` only inserts `payload` when present. |
| PLG-013 | Implemented | Must | The public plugin contract shall be defined in serialized JSON terms, not Rust enum names. Internal enums such as `FieldType`, `ValidationRule`, `DispatchMode`, and `HookAction` are host implementation details. | The contract docs describe string and JSON values, while Rust enums remain internal code structure. |
| PLG-014 | Implemented | Must | Manifest validation rules shall remain string-encoded wire values such as `non_empty` and `one_of:a,b,c`; the internal `ValidationRule` enum is not itself part of the wire contract. | `parse_validation_rule()` consumes raw rule strings rather than deserializing a public enum shape. |
| ERR-004 | Implemented | Should | If plugin stdout contains more than one JSON object, the host shall use the first valid object and record a warning only when the trailing output is another valid JSON object. If trailing output after the first valid object is not valid JSON, the host shall treat it as a protocol error, disable the plugin, and fail the invocation with exit code `2`. | `parse_first_hook_result()` warns on additional valid objects and rejects invalid trailing stdout; dispatch maps that protocol error to plugin disablement and exit code `2`. |

### 4.4 Dispatch, Timeouts, And Session State

| ID | Status | Priority | Requirement | Acceptance Scenario |
| --- | --- | --- | --- | --- |
| DSP-001 | Implemented | Must | Sync handlers shall execute in config order. | Dispatch iterates handlers in resolved-chain order. |
| DSP-002 | Implemented | Must | A sync `action=block` shall short-circuit the chain and return exit code `1`. | Dispatch returns `Blocked` as soon as a sync handler blocks. |
| DSP-004 | Implemented | Must | If all sync handlers proceed, the host shall exit successfully. | `DispatchOutcome::Proceed` maps to success. |
| DSP-006 | Implemented | Must | `--sync` shall run only sync handlers and `--async` shall run only async handlers. | `RunArgs::mode()` drives resolution and dispatch mode filtering. |
| DSP-007 | Implemented | Must | Async `additionalContext` values shall be concatenated with `\\n---\\n`, and async `systemMessage` values shall be concatenated with `\\n`. | Async dispatch writes the aggregated JSON object to stdout. |
| DSP-008 | Implemented | Must | If no handlers match, the host shall exit successfully without emitting an observability event. | Runtime returns early on empty handler chains; the zero-match fast path is tested. |
| TMO-001 | Implemented | Must | Default timeouts shall be `5000ms` for sync handlers and `30000ms` for async handlers unless sync `long_running=true` suppresses the default sync timeout. | `resolve_timeout_ms()` returns those defaults and only suppresses the sync default for valid sync `long_running` handlers. |
| TMO-002 | Implemented | Must | A plugin-declared `timeout_ms` shall override the default timeout, including for sync `long_running` handlers. | `resolve_timeout_ms()` prefers the manifest override. |
| TMO-003 | Implemented | Must | On timeout, the host shall send `SIGTERM`, wait one second, then force-kill if needed. | `terminate_then_kill()` implements TERM then kill. |
| SES-001 | Implemented | Must | Disabled plugin state shall persist in `.sc-hooks/state/session.json`, keyed by session ID. | Session storage tracks disabled plugins per session. |
| SES-002 | Implemented | Must | `SessionEnd` and `sc-hooks audit --reset` shall clear persisted disable state. | Main command handling calls `clear_session()` or `clear_all_sessions()`. |
| TMO-004 | Implemented | Must | The release contract for `long_running` behavior is sync-only: sync handlers with `long_running=true` and no `timeout_ms` run without the default sync timeout; async manifests using `long_running=true` are invalid; SDK runner conveniences remain non-normative authoring helpers. | Manifest validation, audit behavior, timeout resolution, handler discovery, and `long_running_contract` tests all agree on the same contract. |

### 4.5 Metadata And Environment

| ID | Status | Priority | Requirement | Acceptance Scenario |
| --- | --- | --- | --- | --- |
| MTA-001 | Implemented | Must | The host shall discover agent PID, working directory, Git repo root, and current Git branch when available. | `RuntimeMetadata::discover()` populates those fields. |
| MTA-002 | Implemented | Must | The host shall read agent type and session ID from `SC_HOOK_AGENT_TYPE` and `SC_HOOK_SESSION_ID` when present. | `RuntimeMetadata::discover()` copies those env vars into metadata. |
| MTA-004 | Implemented | Must | Before external plugin invocation, the host shall write assembled metadata to a temp file and export `SC_HOOK_TYPE`, `SC_HOOK_EVENT` when present, and `SC_HOOK_METADATA`. | `prepare_for_dispatch()` writes a temp file and `inject_env_vars()` exports those variables. |
| MTA-005 | Implemented | Must | The host shall own the lifecycle of the `SC_HOOK_METADATA` temp file. External plugins may read it as a convenience input, but they do not own or persist it. | `PreparedMetadata` retains a drop guard that deletes the temp file after dispatch scope exits. |

### 4.6 CLI Surface

| ID | Status | Priority | Requirement | Acceptance Scenario |
| --- | --- | --- | --- | --- |
| CLI-001 | Implemented | Must | `sc-hooks run <hook> [event]` shall execute the resolved handler chain for the requested mode. | `run` loads config, resolves handlers, and dispatches them. |
| CLI-002 | Implemented | Must | `sc-hooks audit` shall use static analysis only: config parsing, manifest inspection, matcher validation, metadata satisfiability checks, sandbox validation, and install-plan generation. | Audit does not execute hook logic with live stdin payloads. |
| CLI-003 | Implemented | Should | `sc-hooks fire <hook> [event]` shall run a diagnostic sync dispatch and return a short summary string. | `run_fire()` returns summary text rather than a structured report. |
| CLI-004 | Implemented | Must | `sc-hooks install` shall write `.claude/settings.json` from the current config and plugin manifests. | `write_default_settings()` writes the default settings path. |
| CLI-005 | Implemented | Must | `sc-hooks config`, `handlers`, and `exit-codes` shall expose resolved config, discovered handlers or event taxonomy, and exit-code guidance. | The CLI includes those subcommands. |
| CLI-007 | Implemented | Must | `sc-hooks test <plugin>` and `sc-hooks-test` shall prove the release contract that the docs claim, not only minimal manifest/protocol checks. | `sc-hooks-cli/tests/compliance_host.rs` drives the shared `sc-hooks-test::compliance::run_contract_behavior_suite` through the real CLI binary and directly asserts timeout, invalid output, async misuse, matcher filtering, multi-object stdout warnings, and absent-payload behavior. |

### 4.7 Audit, Logging, And Release Honesty

Retired observability IDs:
- `OBS-003` and `OBS-004` were retired during the move from older ad hoc
  logging drafts to the current `sc-observability` contract; see
  `docs/implementation-gaps.md` for the retirement note tied to `GAP-005` and
  `GAP-007`

| ID | Status | Priority | Requirement | Acceptance Scenario |
| --- | --- | --- | --- | --- |
| AUD-001 | Implemented | Must | Audit shall check handler resolvability, manifest validity, hook declarations, matcher validity, required metadata satisfiability, filesystem validation for `dir_exists` and `file_exists`, sandbox declarations, and install-plan generation. | `audit::run()` emits errors and warnings for those classes. |
| AUD-002 | Implemented | Must | Sandbox warnings shall become errors under `--strict`. | Audit promotes sandbox overruns when strict mode is enabled. |
| AUD-005 | Implemented | Must | Audit shall reject manifests that declare `long_running=true` on async handlers. | `audit::run()` surfaces `AUD-005` when manifest loading hits `ManifestError::AsyncLongRunningUnsupported`. |
| AUD-009 | Implemented | Must | Audit shall reject manifests that declare `long_running=true` without a non-empty description. | `audit::run()` surfaces `AUD-009` when manifest loading hits `ManifestError::MissingLongRunningDescription`. |
| OBS-001 | Implemented | Must | Any invocation that executes at least one handler shall append a structured `LogEvent` JSONL record via `sc-observability`, including `hook`, `matcher`, `mode`, `handlers`, `results`, `total_ms`, and `exit`. | `observability::emit_dispatch_event()` emits service-scoped `LogEvent` records with `matcher = event` or `\"*\"` when no event exists. |
| OBS-002 | Implemented | Must | Current observability output shall use the service-scoped `sc-observability` file-sink layout at `.sc-hooks/observability/sc-hooks/logs/sc-hooks.log.jsonl` unless `SC_HOOKS_ENABLE_FILE_SINK=0` intentionally disables the file sink for an operator/debugging session. | The logger uses `LoggerConfig::default_for(ServiceName::new("sc-hooks"), ".sc-hooks/observability")`, and observability contract tests cover the default file sink path plus the explicit file-sink disable override. |
| OBS-005 | Implemented | Must | Error records shall include the handler name, `error_type`, elapsed time, and `disabled=true` when the plugin is disabled. | `HandlerResultRecord` is serialized into observability event fields for all error outcomes. |
| OBS-006 | Implemented | Must | Structured observability integration shall use the logging-only `sc-observability` crate from the external workspace referenced by `sc-hooks-cli/Cargo.toml` at `../../../sc-observability/...`. | `sc-hooks-cli` depends on `sc-observability` directly and does not use ad hoc in-workspace logger code. |
| OBS-007 | Implemented | Must | `sc-observability` integration shall be owned by `sc-hooks-cli` only. `sc-hooks-core`, `sc-hooks-sdk`, and `sc-hooks-test` shall remain observability-implementation-agnostic. | Logger setup and sink lifecycle live at the CLI/application boundary; lower crates expose typed data and errors instead of owning observability configuration. |
| OBS-008 | Implemented | Must | The initial observability adoption shall not pull in other crates from the sibling `sc-observability` workspace beyond the logging-focused crate and shared types. | `sc-hooks-cli` uses `sc-observability` and `sc-observability-types` only; broader telemetry layers remain out of scope. |
| OBS-009 | Implemented | Should | The CLI may expose environment-flag sink toggles for operator/debugging sessions through `SC_HOOKS_ENABLE_CONSOLE_SINK` and `SC_HOOKS_ENABLE_FILE_SINK`; unrecognized values shall emit a warning to `stderr`, the file sink remains canonical by default, and the console sink emits the contract-tested default summary line only. | Real-dispatch observability tests prove success, block, invalid-json error, and timeout emission with the console sink enabled; docs and logging contract enumerate the accepted env values and the default file-sink posture. |
| DEF-008 | Implemented | Should | Real-dispatch console-sink coverage shall remain proved alongside the file-sink baseline, covering success, block, error, and timeout emission through the actual `sc-hooks-cli` path without weakening the JSONL contract. | `sc-hooks-cli/tests/observability_contract.rs` proves console-sink and file-sink coverage together, while the remaining observability expansion work is carried by `DEF-010` through `DEF-019`. |
| BND-001 | Implemented | Must | The source crates under `plugins/` shall be documented with an explicit maturity level: scaffold/reference or runtime implementation with direct tests, and those classifications shall agree across the control docs. | The source crates under `plugins/` are currently documented as scaffold/reference code in the release posture, and the control docs align on that first-release scope. |
| BND-001a | Implemented | Must | The documented plugin inventory and maturity map shall match the actual source crates in `plugins/`: `agent-session-foundation`, `agent-spawn-gates`, `atm-extension`, `tool-output-gates`, `audit-logger`, `conditional-source`, `event-relay`, `guard-paths`, `identity-state`, `notify`, `policy-enforcer`, `save-context`, and `template-source`. | Architecture and plan docs enumerate the same source-crate set and classify all of them as scaffold/reference for the current release scope. |
| BND-002 | Implemented | Must | Any bundled plugin described as shipped functionality shall have direct behavior tests and runtime installation guidance. Source-only implementation crates may land before install guidance, but they must not be described as preinstalled runtime plugins. | The docs still describe runtime discovery as `.sc-hooks/plugins/` and do not claim any source-owned plugin crate as bundled or preinstalled without matching install guidance. |

### 4.8 Exit Code Taxonomy

| ID | Status | Priority | Requirement | Acceptance Scenario |
| --- | --- | --- | --- | --- |
| EXC-001 | Implemented | Must | Exit code `1` (`BLOCKED`) shall be reserved for sync `action=block` outcomes. | `CliError::Blocked` maps to exit code `1`. |
| EXC-002 | Implemented | Must | Exit code `2` (`PLUGIN_ERROR`) shall cover runtime plugin failures and protocol violations such as invalid JSON, non-zero exit, spawn/write/read failures, and invalid `--payload` JSON parsing. | `CliError::PluginError` maps to exit code `2`. |
| EXC-003 | Implemented | Must | Exit code `3` (`CONFIG_ERROR`) shall cover config read/parse/shape failures. | `CliError::Config` maps to exit code `3`. |
| EXC-004 | Implemented | Must | Exit code `4` (`RESOLUTION_ERROR`) shall cover unresolved handlers and manifest-load or manifest-compatibility failures discovered during handler resolution. | `ResolutionError::UnresolvedHandler` and `ResolutionError::ManifestLoadFailed` both map through `CliError::Resolution` to exit code `4`. |
| EXC-005 | Implemented | Must | Exit code `5` (`VALIDATION_ERROR`) shall be reserved for handler metadata requirement failures after resolution and before plugin execution. | `CliError::Validation` is constructed only from missing or invalid required metadata fields. |
| EXC-006 | Implemented | Must | Exit code `6` (`TIMEOUT`) shall be reserved for synchronous timeout failures that abort the host invocation. | `CliError::Timeout` maps to exit code `6`. |
| EXC-007 | Implemented | Must | Exit code `7` (`AUDIT_FAILURE`) shall cover failed audit results and failed compliance-test runs surfaced as audit-style failures. | `CliError::AuditFailure` maps to exit code `7`. |
| EXC-008 | Implemented | Must | Exit code `10` (`INTERNAL_ERROR`) shall cover panic or host-internal failures not represented by more specific CLI error variants. | Panic handling and `CliError::Internal` both terminate with `10`. |

## 5. Non-Functional Requirements

### 5.1 Testing And Portability

| ID | Status | Priority | Requirement | Acceptance Scenario |
| --- | --- | --- | --- | --- |
| TST-001 | Implemented | Must | Config parsing, resolution, dispatch, metadata, timeout handling, and audit shall be unit or integration testable from Rust. | The workspace includes tests for those components. |
| TST-007 | Implemented | Must | The reusable compliance harness shall cover the same protocol and behavioral guarantees that the release docs promise. | `sc-hooks-test/src/compliance.rs::run_contract_behavior_suite` is exercised directly and through `sc-hooks-cli/tests/compliance_host.rs`, so each release-facing compliance claim points to a real shared harness assertion. |
| TST-008 | Implemented | Should | The Claude hook harness shall detect Claude CLI version bumps by comparing `claude --version` with the `claude_version` recorded in `test-harness/hooks/claude/fixtures/approved/manifest.json`; mismatches shall exit non-zero so maintainers rerun live schema validation before accepting provider-contract changes. | `python3 scripts/verify-claude-hook-api.py` exits `0` when `claude --version` matches the approved manifest's `claude_version`, and exits `1` with a warning when the installed Claude version differs. |
| PRT-001 | Implemented | Must | The workspace shall build and test on Linux and macOS in CI. | `.github/workflows/ci.yml` runs build/test on Ubuntu and macOS. |

## 6. Planned Observability Phase Requirements

Phase-entry amendment rules for the committed observability phase:

- Observability Phase 1 amends `CFG-002` so `[observability]` becomes a supported top-level section alongside the current config sections.
- Observability Phase 1 amends `OBS-001` and `OBS-002` so `off` disables durable structured sinks while direct stderr warnings and failure notices remain visible.
- `DEF-006` is superseded. The project will not restore a `[logging]` section; the committed config surface is `[observability]`.

| ID | Status | Priority | Requirement | Acceptance Scenario |
| --- | --- | --- | --- | --- |
| DEF-010 | Planned | Must | Observability configuration shall support deterministic layering across built-in defaults, global user config at `~/.sc-hooks/config.toml`, repo-local `.sc-hooks/config.toml`, and environment overrides. | The CLI loads both config layers with fixed precedence, the supported keys are documented, and tests prove repo-local overrides without breaking repo-relative plugin policy. |
| DEF-011 | Planned | Must | Observability mode selection shall support `off`, `standard`, and `full`; global config may set defaults for `off` or `standard` only, while enabling `full` remains a repo-local or operator action. | The config contract documents mode semantics, tests prove mode resolution across global/local/env layers, and `full` is rejected when requested only from global scope. |
| DEF-012 | Planned | Must | Full audit output shall default to `.sc-hooks/audit/`, use run-scoped durable files, and allow repo-local relative or absolute path overrides; relative paths resolve from the immutable project root. | The audit writer produces run-scoped files beneath the default root, local overrides resolve deterministically, and path-resolution tests prove project-root-relative behavior. |
| DEF-013 | Planned | Must | Full audit shall support a lean profile for evals and harness runs plus a debug profile for deeper troubleshooting; raw payload capture remains a separate explicit opt-in. | Docs freeze the mandatory fields for both profiles, tests cover profile selection, and payload capture cannot turn on implicitly with `full` alone. |
| DEF-014 | Planned | Must | Full audit shall use strict redaction by default, never rely on the human console sink as a machine contract, and keep the durable audit JSONL files as the canonical machine-readable source for the phase. | Sensitive-field tests prove masking or summarization, audit JSONL remains the canonical machine-readable source, and the human console format is documented as non-contractual. |
| DEF-015 | Planned | Must | Observability, audit, retention, and pruning failures shall never affect hook execution outcomes. | Forced logger-init, emit, and prune failures leave hook exit behavior unchanged while producing the documented degraded signals. |
| DEF-016 | Planned | Must | Production-grade audit mode shall support at least 50 simultaneous agents by sharding durable audit output into run-scoped files, bounding retention, and avoiding a single hot shared file. | Load and soak tests prove 50+ concurrent agents can emit audit records without corruption, unbounded contention, or unbounded disk growth. |
| DEF-017 | Planned | Must | Full audit mode shall record hook invocation attempts even when no handlers match or dispatch fails before handler execution, while `standard` mode keeps the current lower-volume dispatch-log posture. | Integration tests prove zero-match, resolution-failure, and pre-dispatch failure audit records in `full` mode without changing the current `standard` fast path. |
| DEF-019 | Planned | Must | The canonical product, runtime, binary, service, and docs name shall converge on `sc-hooks`, while `hooks` remains a supported convenience CLI alias only. | Control docs, binary naming, and public references converge on `sc-hooks`, and `hooks` is documented as a non-canonical alias. |

## 7. Deferred Items

| ID | Priority | Deferred Behavior | Exit Condition |
| --- | --- | --- | --- |
| DEF-001 | Should | Production-ready bundled plugin behavior beyond scaffold/reference implementations | Real runtime behavior, tests, and installation story exist |
| DEF-002 | Should | A richer diagnostic `fire` report format beyond the current summary string | A stable structured diagnostic output is implemented and tested |
| DEF-003 | Should | Any SDK-level `LongRunning` abstraction beyond the host's current manifest-driven behavior | The SDK, docs, and tests agree on a stable public contract |
| DEF-004 | Should | More granular exit codes for manifest incompatibility vs other resolution failures | The code introduces additional exit-code variants and the CLI/docs are updated together |
| DEF-005 | Should | Builtin handler resolution inside the dispatcher | The product intentionally restores a builtin path and documents how it coexists with plugin resolution |
| DEF-007 | Should | Release-facing support for payload-condition operators beyond the `PLC-002` set (`not_contains`, `gt`, `lt`, `gte`, `lte`) | Requirements, contract docs, and tests are updated together for the expanded operator set |
| DEF-018 | Should | Future global config may define exporter and OTel defaults, but those defaults shall not implicitly enable repo-local `full` audit mode. | Exporter keys are documented separately from the committed observability phase, and tests prove transport defaults do not escalate audit mode by themselves when that later work lands. |

## 8. Post-Release Hook Extension Track

These items are not part of the current release baseline above. They define the
required guardrails for the next hook-extension development track after release
work is complete.

Detailed post-capture runtime design for this track lives in
`docs/phase-bc-hook-runtime-design.md`.

| ID | Status | Priority | Requirement | Acceptance Scenario |
| --- | --- | --- | --- | --- |
| HKR-001 | Deferred | Must | The first hook-extension implementation target shall be the current Claude ATM hook set documented in `docs/hook-api/claude-hook-api.md`; ATM-specific behavior remains isolated in `docs/hook-api/atm-hook-extension.md` and is not itself the generic hook contract. | Hook-extension work cites the Claude API doc as the implementation baseline and keeps ATM-only routing/persistence details in the separate ATM extension doc. |
| HKR-002 | Implemented | Must | No hook implementation code shall be written until the Claude hook schema harness captures and validates the required Claude payloads for the planned hook set. | The Phase 1 harness and follow-up capture passes produced verified Claude fixtures before any new hook runtime crate is authorized. |
| HKR-003 | Implemented | Must | After the Claude schema harness captures real payloads, the plan and hook API docs shall be revised from captured evidence before implementation begins. | `docs/archive/plugin-plan-s9.md` and the hook API docs were revised from captured fixtures, including `resume` and `clear` follow-up evidence. |
| HKR-004 | Deferred | Must | The initial Claude hook implementation scope shall cover only the documented eight-hook ATM baseline: `SessionStart`, `SessionEnd`, `PreToolUse(Bash)`, `PostToolUse(Bash)`, `PreToolUse(Agent)`, `Notification(idle_prompt)`, `PermissionRequest`, and `Stop`. | Hook crates and tests map only to that documented eight-hook set unless requirements are explicitly expanded later; the seven non-`Notification` surfaces are locally captured, `Notification(idle_prompt)` remains documented as wired-but-unresolved in local capture, and the clean runtime design is frozen in `docs/phase-bc-hook-runtime-design.md`. |
| HKR-005 | Deferred | Must | The Claude schema harness shall preserve raw captured fixtures as evidence and shall provide a manual schema-drift detection path that reports required-field removal, type drift, and newly added fields without auto-fixing models. | `test-harness/hooks/run-schema-drift.py` compares current captures to approved fixtures, emits drift output, and retains captured artifacts for review; schema drift remains a manual investigation path rather than a CI gate. |
| HKR-006 | Deferred | Must | Provider-specific docs for Codex, Gemini, and Cursor may be kept in the docs set before implementation, but those providers shall not block the first Claude implementation path. | The first hook-development sequence proceeds on Claude-only capture and implementation even while other provider docs remain present. |
| HKR-007 | Deferred | Must | Cursor Agent shall remain documented as a provider reference during the first development pass, but Cursor harness capture and Cursor-targeting runtime implementation are deferred until a later explicitly approved follow-on sprint. | `docs/hook-api/cursor-agent-hook-api.md` exists, while the first harness and implementation work stays Claude-only. |
| HKR-008 | Implemented | Must | The generic session foundation shall persist one canonical session-state record keyed by `session_id`, `active_pid`, and `ai_root_dir`, where `ai_root_dir` is the immutable working directory established from the root-establishing `SessionStart` for the runtime instance, `ai_current_dir` is chained from each hook payload `cwd`, and downstream consumers receive normalized project-root context regardless of later provider drift. Inbound `CLAUDE_PROJECT_DIR`, when present, is a required equality check against the persisted canonical root rather than a silent fallback. | Hook lifecycle code writes one canonical session record, uses the root-establishing `SessionStart` launch as immutable root for that runtime instance, preserves later `cwd` snapshots separately as current-directory context, never rewrites root identity from later `cwd` drift, emits prominent error-level observability when inbound `CLAUDE_PROJECT_DIR` diverges from the persisted canonical root, and exposes the canonical root back to consumers as normalized project-root context. |
| HKR-009 | Implemented | Must | Canonical hook session-state updates shall use atomic write semantics, shall not rewrite `session.json` when the canonical record is unchanged, and shall emit hook logs on every invocation whether or not state changes. The earlier trait-freeze planning gate is treated as satisfied through the executable-plugin JSON schema contract recorded under `SEAL-001` in `docs/implementation-gaps.md`. | Session-state persistence uses same-directory temp-plus-rename, increments revision only on material change, skips unchanged rewrites, still emits observability/log output for every hook invocation, and the trait-freeze closure is documented through `SEAL-001` in `docs/implementation-gaps.md`. |
| HKR-010 | Deferred | Must | Spawn/tool-gate behavior shall enforce fenced `json` input where required, validate that JSON against the declared schema source, and return exact retryable failure reasons on block. | Invalid fenced JSON is blocked with deterministic retry guidance, schema lookup is explicit, and named-agent/background-agent policy outcomes are tested directly. |
| HKR-011 | Implemented | Must | ATM extension behavior shall enrich the canonical session-state record through extension fields and environment inheritance (`ATM_TEAM`, `ATM_IDENTITY`) without redefining the generic state model. | ATM relay/identity code writes extension fields onto the canonical record, preserves team linkage, documents child identity override behavior, and leaves the generic lifecycle model owned by the foundation crate. |
| HKR-012 | Deferred | Must | The global HTML reporting stack shall be built and QA-approved before any schema-drift or other report-generating sprint depends on it. | `$HOME/.claude/skills/html-report/SKILL.md` and `~/.claude/agents/html-report-generator.md` exist, pass review against `/Users/randlee/Documents/github/synaptic-canvas/docs/claude-code-skills-agents-guidelines-0.4.md`, and produce one valid self-contained HTML report in a tested invocation before Sprint `S9-P3` is considered runnable. |
| HKR-013 | Implemented | Must | ATM relay handling shall preserve distinct raw-request, validated-request, relay-decision, and relay-result stages so validation, routing, and side effects remain separately testable. | `plugins/atm-extension` keeps the four-stage relay pipeline explicit, uses one authoritative `ToolName` type, and covers the typed relay boundary with direct tests. |

Additional verified Claude provider surface outside the current baseline:
- `WorktreeCreate` and `WorktreeRemove` are documented Claude Code hook events,
  but they are not part of the current `sc-hooks` implementation baseline
- if later promoted into scope, they must be treated as top-level provider
  hooks rather than `PreToolUse`-style matcher cases
- `WorktreeCreate` uses a provider-specific success/failure contract:
  - command hooks return the absolute worktree path on stdout
  - stderr carries rejection or failure detail
  - non-zero exit fails worktree creation
  - `HookResult` / decision JSON does not apply
- `WorktreeRemove` receives the provider-returned `worktree_path` and performs
  cleanup side effects; it is not part of the generic `HookResult` decision
  model

## 9. Release Rule

If a behavior is not implemented and not required for release, it must be deferred.

If a behavior is committed to an approved next phase, it must be marked
`Planned` rather than `Deferred`, and the owning phase plus acceptance gate
must be named in `docs/project-plan.md`.

If a behavior is required for release but not yet fully proved, it must appear in `docs/traceability.md` and, when needed for historical planning context, in `docs/archive/`.

## Requirement Amendment Notes

- `DEF-006`
  - prior text: config-driven observability sink routing or a `[logging]` section in `.sc-hooks/config.toml` beyond the current env-flag sink toggles
  - current text: superseded by the planned `[observability]` surface in `DEF-010` through `DEF-017` plus the deferred exporter follow-on in `DEF-018`; the project will not restore `[logging]` as the committed next-phase contract
  - authorizing phase: `SC-LOG-S2`
- `DEF-008`
  - prior text: console-sink dispatch coverage remained an operator-facing follow-up inside the broader observability phase
  - current text: console-sink dispatch coverage is implemented through the real `sc-hooks-cli` path; the remaining observability expansion work is carried by `DEF-010` through `DEF-019`
  - authorizing sprint: `S9-BONUS`
- `TST-008`
  - prior text: required-before-release Claude version-bump detection
  - current text: implemented Claude version-bump detection with direct script and test proof
  - authorizing sprint: `S10-VERSION-BUMP-1`

- `HKR-011`
  - prior text: ATM extension behavior could remain an ATM-owned state model as long as relay behavior was documented consistently
  - current text: ATM extension behavior shall enrich the canonical generic session-state record through extension fields and environment inheritance without redefining the generic lifecycle model
  - authorizing sprint: `S9-HP5`
- `HKR-013`
  - prior text: ATM relay handling could validate and route requests through one combined request type if tests still covered the visible outcomes
  - current text: ATM relay handling shall preserve distinct raw-request, validated-request, relay-decision, and relay-result stages so validation, routing, and side effects remain separately testable
  - authorizing sprint: `S9-HP5`
