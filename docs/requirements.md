# sc-hooks Requirements

## 1. Purpose

This document defines the release-facing behavior for `sc-hooks` as it exists today or must exist before release. It is intentionally narrower than the prior draft: if the code does not implement a behavior and the release does not require it, the behavior is deferred or tracked as a gap.

## 2. Status Model

| Status | Meaning |
| --- | --- |
| `Implemented` | Backed by current code and direct tests, or by code plus obvious mechanical proof |
| `Required Before Release` | Intended release behavior that is not yet proved cleanly enough by code, tests, or contracts |
| `Deferred` | Not part of the current release baseline |

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
- shipped runtime plugin behavior from the source crates under `plugins/`; every current crate there remains scaffold/reference only
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
| OBS-002 | Implemented | Must | Current observability output shall use the service-scoped `sc-observability` file-sink layout at `.sc-hooks/observability/sc-hooks/logs/sc-hooks.log.jsonl`. | The logger uses `LoggerConfig::default_for(ServiceName::new("sc-hooks"), ".sc-hooks/observability")`. |
| OBS-005 | Implemented | Must | Error records shall include the handler name, `error_type`, elapsed time, and `disabled=true` when the plugin is disabled. | `HandlerResultRecord` is serialized into observability event fields for all error outcomes. |
| OBS-006 | Implemented | Must | Structured observability integration shall use the logging-only `sc-observability` crate from the external workspace referenced by `sc-hooks-cli/Cargo.toml` at `../../../sc-observability/...`. | `sc-hooks-cli` depends on `sc-observability` directly and does not use ad hoc in-workspace logger code. |
| OBS-007 | Implemented | Must | `sc-observability` integration shall be owned by `sc-hooks-cli` only. `sc-hooks-core`, `sc-hooks-sdk`, and `sc-hooks-test` shall remain observability-implementation-agnostic. | Logger setup and sink lifecycle live at the CLI/application boundary; lower crates expose typed data and errors instead of owning observability configuration. |
| OBS-008 | Implemented | Must | The initial observability adoption shall not pull in other crates from the sibling `sc-observability` workspace beyond the logging-focused crate and shared types. | `sc-hooks-cli` uses `sc-observability` and `sc-observability-types` only; broader telemetry layers remain out of scope. |
| BND-001 | Implemented | Must | The source crates under `plugins/` shall be documented as reference or scaffold implementations unless and until they ship real behavior and tests. | The plugin crates currently read stdin and return `{\"action\":\"proceed\"}`. |
| BND-001a | Implemented | Must | The documented reference/scaffold inventory shall match the actual source crates in `plugins/`: `audit-logger`, `conditional-source`, `event-relay`, `guard-paths`, `identity-state`, `notify`, `policy-enforcer`, `save-context`, and `template-source`. | The README and architecture docs enumerate the same set of source crates present in the repository. |
| BND-002 | Implemented | Must | Any bundled plugin described as shipped functionality shall have direct behavior tests and runtime installation guidance. The current release baseline describes no `plugins/` source crate as shipped functionality. | README, architecture, and plugin Cargo metadata all mark the current source crates as scaffold/reference only, so no shipped-plugin claim exists without matching install guidance and direct tests. |

### 4.8 Exit Code Taxonomy

| ID | Status | Priority | Requirement | Acceptance Scenario |
| --- | --- | --- | --- | --- |
| EXC-001 | Implemented | Must | Exit code `1` (`BLOCKED`) shall be reserved for sync `action=block` outcomes. | `CliError::Blocked` maps to exit code `1`. |
| EXC-002 | Implemented | Must | Exit code `2` (`PLUGIN_ERROR`) shall cover runtime plugin failures and protocol violations such as invalid JSON, non-zero exit, spawn/write/read failures, and invalid `--payload` JSON parsing. | `CliError::PluginError` maps to exit code `2`. |
| EXC-003 | Implemented | Must | Exit code `3` (`CONFIG_ERROR`) shall cover config read/parse/shape failures. | `CliError::Config` maps to exit code `3`. |
| EXC-004 | Implemented | Must | Exit code `4` (`RESOLUTION_ERROR`) shall cover unresolved handlers and manifest-load or manifest-compatibility failures discovered during handler resolution. | `ResolutionError::UnresolvedHandler` and `ResolutionError::ManifestLoad` both map through `CliError::Resolution` to exit code `4`. |
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
| PRT-001 | Implemented | Must | The workspace shall build and test on Linux and macOS in CI. | `.github/workflows/ci.yml` runs build/test on Ubuntu and macOS. |

## 6. Deferred Items

| ID | Priority | Deferred Behavior | Exit Condition |
| --- | --- | --- | --- |
| DEF-001 | Should | Production-ready bundled plugin behavior beyond scaffold/reference implementations | Real runtime behavior, tests, and installation story exist |
| DEF-002 | Should | A richer diagnostic `fire` report format beyond the current summary string | A stable structured diagnostic output is implemented and tested |
| DEF-003 | Should | Any SDK-level `LongRunning` abstraction beyond the host's current manifest-driven behavior | The SDK, docs, and tests agree on a stable public contract |
| DEF-004 | Should | More granular exit codes for manifest incompatibility vs other resolution failures | The code introduces additional exit-code variants and the CLI/docs are updated together |
| DEF-005 | Should | Builtin handler resolution inside the dispatcher | The product intentionally restores a builtin path and documents how it coexists with plugin resolution |
| DEF-006 | Should | Config-driven observability sink routing or a `[logging]` section in `.sc-hooks/config.toml` | The CLI reintroduces sink configuration and the contract docs are updated with the supported keys and semantics |
| DEF-007 | Should | Release-facing support for payload-condition operators beyond the `PLC-002` set (`not_contains`, `gt`, `lt`, `gte`, `lte`) | Requirements, contract docs, and tests are updated together for the expanded operator set |

## 7. Post-Release Hook Extension Track

These items are not part of the current release baseline above. They define the
required guardrails for the next hook-extension development track after release
work is complete.

| ID | Status | Priority | Requirement | Acceptance Scenario |
| --- | --- | --- | --- | --- |
| HKR-001 | Deferred | Must | The first hook-extension implementation target shall be the current Claude ATM hook set documented in `docs/hook-api/claude-hook-api.md`; ATM-specific behavior remains isolated in `docs/hook-api/atm-hook-extension.md` and is not itself the generic hook contract. | Hook-extension work cites the Claude API doc as the implementation baseline and keeps ATM-only routing/persistence details in the separate ATM extension doc. |
| HKR-002 | Deferred | Must | No hook implementation code shall be written until the Claude hook schema harness captures and validates the required Claude payloads for the planned hook set. | The first hook-development sprint produces captured Claude fixtures and validation models before any hook runtime crate is added. |
| HKR-003 | Deferred | Must | After the Claude schema harness captures real payloads, the plan and hook API docs shall be revised from captured evidence before implementation begins. | `docs/plugin-plan-s9.md` and the hook API docs are updated from captured fixtures before the first hook crate lands. |
| HKR-004 | Deferred | Must | The initial Claude hook implementation scope shall cover only the currently verified eight-hook ATM baseline: `SessionStart`, `SessionEnd`, `PreToolUse(Bash)`, `PostToolUse(Bash)`, `PreToolUse(Task)`, `Notification(idle_prompt)`, `PermissionRequest`, and `Stop`. | Hook crates and tests map only to that verified eight-hook set unless requirements are explicitly expanded later. |
| HKR-005 | Deferred | Must | The Claude schema harness shall fail CI on required-field removal or type drift and shall preserve raw captured fixtures as evidence. | Hook-schema CI fails on breaking Claude payload drift and retains captured fixture artifacts for review. |
| HKR-006 | Deferred | Must | Provider-specific docs for Codex, Gemini, and Cursor may be kept in the docs set before implementation, but those providers shall not block the first Claude implementation path. | The first hook-development sequence proceeds on Claude-only capture and implementation even while other provider docs remain present. |
| HKR-007 | Deferred | Must | Cursor Agent shall remain documented as a provider reference during the first development pass, but Cursor harness capture and Cursor-targeting runtime implementation are deferred until a later explicitly approved follow-on sprint. | `docs/hook-api/cursor-agent-hook-api.md` exists, while the first harness and implementation work stays Claude-only. |

## 8. Release Rule

If a behavior is not implemented and not required for release, it must be deferred.

If a behavior is required for release but not yet fully proved, it must appear in `docs/implementation-gaps.md` and `docs/traceability.md`.
