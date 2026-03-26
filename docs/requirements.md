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
- hook routing to builtins and external plugins
- manifest loading and metadata filtering
- sync/async dispatch with timeouts and per-session disable state
- Claude settings generation from matchers
- audit, diagnostic fire, compliance-test entry points, and exit-code reporting
- JSONL dispatch logging

Current release scope does not include:
- production-ready bundled plugin behavior from the `plugins/` directory
- a stable end-to-end `LongRunning` SDK surface beyond the manifest fields the host already enforces
- a compliance harness that proves every behavior described in older drafts

## 4. Functional Requirements

### 4.1 Configuration

| ID | Status | Priority | Requirement | Acceptance Scenario |
| --- | --- | --- | --- | --- |
| CFG-001 | Implemented | Must | The host shall load its default config from `.sc-hooks/config.toml` relative to the current repository. | `sc-hooks config` reads the default path through `load_default_config()`. |
| CFG-002 | Implemented | Must | The config shall recognize exactly `[meta]`, `[context]`, `[hooks]`, `[logging]`, and `[sandbox]`; only `[meta]` and `[hooks]` are required. | Unknown top-level sections fail parsing. |
| CFG-003 | Implemented | Must | `[hooks]` shall map hook names to ordered handler arrays. | Resolution and dispatch preserve config order. |
| CFG-004 | Implemented | Must | `[context] team = "<name>"` shall map to `metadata.team.name`; other context keys remain top-level metadata fields. | `map_context_to_metadata()` applies the special-case mapping only for `team`. |
| CFG-006 | Implemented | Must | `[logging]` shall configure the hook log path and the recorded log level label. | `LoggingConfig` supports `hook_log` and `level`; dispatch records carry the chosen level. |
| CFG-008 | Implemented | Should | `[sandbox]` shall allow per-plugin network and path overrides for audit validation. | `SandboxConfig` exposes `allow_network` and `allow_paths`. |

### 4.2 Resolution And Matching

| ID | Status | Priority | Requirement | Acceptance Scenario |
| --- | --- | --- | --- | --- |
| RES-001 | Implemented | Must | Handler resolution shall prefer builtins over external plugins. | `log` resolves as a builtin even if a same-named plugin executable exists. |
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
| ERR-004 | Implemented | Should | If a plugin writes multiple JSON objects to stdout, the host shall use the first one and record a warning. | `parse_first_hook_result()` returns a warning when more than one object is present. |

### 4.4 Dispatch, Timeouts, And Session State

| ID | Status | Priority | Requirement | Acceptance Scenario |
| --- | --- | --- | --- | --- |
| DSP-001 | Implemented | Must | Sync handlers shall execute in config order. | Dispatch iterates handlers in resolved-chain order. |
| DSP-002 | Implemented | Must | A sync `action=block` shall short-circuit the chain and return exit code `1`. | Dispatch returns `Blocked` as soon as a sync handler blocks. |
| DSP-004 | Implemented | Must | If all sync handlers proceed, the host shall exit successfully. | `DispatchOutcome::Proceed` maps to success. |
| DSP-006 | Implemented | Must | `--sync` shall run only sync handlers and `--async` shall run only async handlers. | `RunArgs::mode()` drives resolution and dispatch mode filtering. |
| DSP-007 | Implemented | Must | Async `additionalContext` values shall be concatenated with `\\n---\\n`, and async `systemMessage` values shall be concatenated with `\\n`. | Async dispatch writes the aggregated JSON object to stdout. |
| DSP-008 | Implemented | Must | If no handlers match, the host shall exit successfully without writing a dispatch log entry. | Runtime returns early on empty handler chains; the zero-match fast path is tested. |
| TMO-001 | Implemented | Must | Default timeouts shall be `5000ms` for sync handlers and `30000ms` for async handlers. | `resolve_timeout_ms()` returns those defaults. |
| TMO-002 | Implemented | Must | A plugin-declared `timeout_ms` shall override the default timeout. | `resolve_timeout_ms()` prefers the manifest override. |
| TMO-003 | Implemented | Must | On timeout, the host shall send `SIGTERM`, wait one second, then force-kill if needed. | `terminate_then_kill()` implements TERM then kill. |
| SES-001 | Implemented | Must | Disabled plugin state shall persist in `.sc-hooks/state/session.json`, keyed by session ID. | Session storage tracks disabled plugins per session. |
| SES-002 | Implemented | Must | `SessionEnd` and `sc-hooks audit --reset` shall clear persisted disable state. | Main command handling calls `clear_session()` or `clear_all_sessions()`. |
| TMO-004 | Required Before Release | Must | The release contract for `long_running` behavior shall be documented and tested end to end across host, SDK, and audit behavior. | The SDK surface, architecture doc, and tests all agree on the same contract. |

### 4.5 Metadata And Environment

| ID | Status | Priority | Requirement | Acceptance Scenario |
| --- | --- | --- | --- | --- |
| MTA-001 | Implemented | Must | The host shall discover agent PID, working directory, Git repo root, and current Git branch when available. | `RuntimeMetadata::discover()` populates those fields. |
| MTA-002 | Implemented | Must | The host shall read agent type and session ID from `SC_HOOK_AGENT_TYPE` and `SC_HOOK_SESSION_ID` when present. | `RuntimeMetadata::discover()` copies those env vars into metadata. |
| MTA-004 | Implemented | Must | Before external plugin invocation, the host shall write assembled metadata to a temp file and export `SC_HOOK_TYPE`, `SC_HOOK_EVENT` when present, and `SC_HOOK_METADATA`. | `prepare_for_dispatch()` writes a temp file and `inject_env_vars()` exports those variables. |

### 4.6 CLI Surface

| ID | Status | Priority | Requirement | Acceptance Scenario |
| --- | --- | --- | --- | --- |
| CLI-001 | Implemented | Must | `sc-hooks run <hook> [event]` shall execute the resolved handler chain for the requested mode. | `run` loads config, resolves handlers, and dispatches them. |
| CLI-002 | Implemented | Must | `sc-hooks audit` shall use static analysis only: config parsing, manifest inspection, matcher validation, metadata satisfiability checks, sandbox validation, and install-plan generation. | Audit does not execute hook logic with live stdin payloads. |
| CLI-003 | Implemented | Should | `sc-hooks fire <hook> [event]` shall run a diagnostic sync dispatch and return a short summary string. | `run_fire()` returns summary text rather than a structured report. |
| CLI-004 | Implemented | Must | `sc-hooks install` shall write `.claude/settings.json` from the current config and plugin manifests. | `write_default_settings()` writes the default settings path. |
| CLI-005 | Implemented | Must | `sc-hooks config`, `handlers`, and `exit-codes` shall expose resolved config, discovered handlers or event taxonomy, and exit-code guidance. | The CLI includes those subcommands. |
| CLI-007 | Required Before Release | Must | `sc-hooks test <plugin>` and `sc-hooks-test` shall prove the release contract that the docs claim, not only minimal manifest/protocol checks. | The compliance suite covers timeout, invalid output, async misuse, matcher validity, and absent-payload behavior with direct assertions. |

### 4.7 Audit, Logging, And Release Honesty

| ID | Status | Priority | Requirement | Acceptance Scenario |
| --- | --- | --- | --- | --- |
| AUD-001 | Implemented | Must | Audit shall check handler resolvability, manifest validity, hook declarations, matcher validity, required metadata satisfiability, filesystem validation for `dir_exists` and `file_exists`, sandbox declarations, and install-plan generation. | `audit::run()` emits errors and warnings for those classes. |
| AUD-002 | Implemented | Must | Sandbox warnings shall become errors under `--strict`. | Audit promotes sandbox overruns when strict mode is enabled. |
| OBS-001 | Implemented | Must | Any invocation that executes at least one handler shall append a structured dispatch record to the configured hook log path. | `emit_dispatch_log()` appends `DispatchLogEntry` JSONL records. |
| OBS-002 | Implemented | Must | The builtin `log` handler shall append its own simpler JSONL records to the same configured hook log path. | `builtins::log::write_entry()` writes minimal records. |
| OBS-005 | Implemented | Must | Error records shall include the handler name, `error_type`, elapsed time, and `disabled=true` when the plugin is disabled. | `error_result()` and dispatch logging preserve those fields. |
| BND-001 | Implemented | Must | The source crates under `plugins/` shall be documented as reference or scaffold implementations unless and until they ship real behavior and tests. | The plugin crates currently read stdin and return `{\"action\":\"proceed\"}`. |
| BND-002 | Required Before Release | Must | Any bundled plugin described as shipped functionality shall have direct behavior tests and runtime installation guidance. | The docs, plugin code, and tests all support the same claim. |

## 5. Non-Functional Requirements

### 5.1 Testing And Portability

| ID | Status | Priority | Requirement | Acceptance Scenario |
| --- | --- | --- | --- | --- |
| TST-001 | Implemented | Must | Config parsing, resolution, dispatch, metadata, timeout handling, and audit shall be unit or integration testable from Rust. | The workspace includes tests for those components. |
| TST-007 | Required Before Release | Must | The reusable compliance harness shall cover the same protocol and behavioral guarantees that the release docs promise. | `docs/traceability.md` can point every test-claim row to a real harness assertion or to an explicit gap. |
| PRT-001 | Implemented | Must | The workspace shall build and test on Linux and macOS in CI. | `.github/workflows/ci.yml` runs build/test on Ubuntu and macOS. |

## 6. Deferred Items

| ID | Priority | Deferred Behavior | Exit Condition |
| --- | --- | --- | --- |
| DEF-001 | Should | Production-ready bundled plugin behavior beyond scaffold/reference implementations | Real runtime behavior, tests, and installation story exist |
| DEF-002 | Should | A richer diagnostic `fire` report format beyond the current summary string | A stable structured diagnostic output is implemented and tested |
| DEF-003 | Should | Any SDK-level `LongRunning` abstraction beyond the host's current manifest-driven behavior | The SDK, docs, and tests agree on a stable public contract |

## 7. Release Rule

If a behavior is not implemented and not required for release, it must be deferred.

If a behavior is required for release but not yet fully proved, it must appear in `docs/implementation-gaps.md` and `docs/traceability.md`.
