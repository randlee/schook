# sc-hooks Traceability

This table maps the most important documented requirements to current implementation, tests, or explicit gaps.

| Requirement ID | Status | Primary implementation | Primary tests | Gap |
| --- | --- | --- | --- | --- |
| CFG-001 | implemented | `sc-hooks-cli/src/config.rs` | `sc-hooks-cli/src/config.rs` tests plus `sc-hooks-cli/tests/runtime_layout_example.rs` proving the checked default `.sc-hooks/config.toml` layout | |
| CFG-002 | implemented | `sc-hooks-cli/src/config.rs` | `parses_observability_section_with_local_only_fields`, `rejects_unknown_observability_field`, and `layered_config_applies_built_in_global_local_and_env_precedence` | |
| CFG-003 | implemented | `sc-hooks-cli/src/config.rs`, `sc-hooks-cli/src/dispatch.rs` | dispatch tests, config tests | |
| CFG-004 | implemented | `sc-hooks-cli/src/config.rs` | `sc-hooks-cli/src/config.rs` tests | |
| CFG-008 | implemented | `sc-hooks-cli/src/config.rs`, `sc-hooks-cli/src/audit.rs` | config tests, audit tests | |
| RES-001 | implemented | `sc-hooks-cli/src/resolution.rs` | resolution tests | |
| RES-002 | implemented | `sc-hooks-cli/src/resolution.rs`, `sc-hooks-cli/src/handlers.rs` | resolution tests plus `sc-hooks-cli/tests/runtime_layout_example.rs` using `examples/runtime-layout/.sc-hooks/` | |
| RES-003 | implemented | `sc-hooks-cli/src/resolution.rs`, `sc-hooks-cli/src/audit.rs` | resolution tests plus audit tests covering unresolved handler failures in runtime and audit | |
| MTR-001 | implemented | `sc-hooks-cli/src/install.rs` | install tests | |
| PLC-001 | implemented | `sc-hooks-sdk/src/conditions.rs`, `sc-hooks-cli/src/resolution.rs` | condition tests, resolution tests | |
| PLC-002 | implemented | `sc-hooks-sdk/src/conditions.rs` | condition tests | DEF-007 |
| PLG-001 | implemented | `sc-hooks-sdk/src/manifest.rs` | manifest tests | |
| PLG-002 | implemented | `sc-hooks-core/src/manifest.rs`, `sc-hooks-sdk/src/manifest.rs` | manifest tests | |
| PLG-003 | implemented | `sc-hooks-core/src/validation.rs`, `sc-hooks-sdk/src/manifest.rs` | manifest tests | |
| PLG-004 | implemented | `sc-hooks-sdk/src/manifest.rs` | manifest tests | |
| PLG-006 | implemented | `sc-hooks-core/src/results.rs`, `sc-hooks-cli/src/dispatch.rs` | dispatch tests | |
| PLG-009 | implemented | `sc-hooks-cli/src/dispatch.rs`, `sc-hooks-cli/src/audit.rs` | dispatch tests, audit tests | |
| PLG-011 | implemented | `sc-hooks-sdk/src/manifest.rs` | manifest tests | |
| PLG-012 | implemented | `sc-hooks-sdk/src/manifest.rs` | manifest tests | |
| PLG-013 | implemented | `sc-hooks-core/src/validation.rs`, `sc-hooks-core/src/dispatch.rs`, `sc-hooks-core/src/results.rs` | protocol contract is documented from current serialized values | |
| PLG-014 | implemented | `sc-hooks-core/src/validation.rs` | validation parsing tests by manifest/condition coverage | |
| ERR-004 | implemented | `sc-hooks-cli/src/dispatch.rs` | dispatch tests covering additional-valid-object warning and invalid-trailing-stdout failure | |
| DSP-001 | implemented | `sc-hooks-cli/src/dispatch.rs` | dispatch tests | |
| DSP-002 | implemented | `sc-hooks-cli/src/dispatch.rs` | dispatch tests | |
| DSP-004 | implemented | `sc-hooks-cli/src/dispatch.rs`, `sc-hooks-cli/src/main.rs` | dispatch tests | |
| DSP-006 | implemented | `sc-hooks-cli/src/main.rs`, `sc-hooks-cli/src/dispatch.rs` | dispatch tests | |
| DSP-007 | implemented | `sc-hooks-cli/src/dispatch.rs` | dispatch tests | |
| DSP-008 | implemented | `sc-hooks-cli/src/main.rs`, `sc-hooks-cli/src/fire.rs`, `sc-hooks-cli/src/observability.rs` | fire tests, dispatch tests, `off_mode_suppresses_durable_observability_output`, and `full_mode_zero_match_writes_audit_record` | |
| TMO-001 | implemented | `sc-hooks-cli/src/timeout.rs` | timeout tests | |
| TMO-002 | implemented | `sc-hooks-cli/src/timeout.rs`, `sc-hooks-cli/src/dispatch.rs` | timeout tests | |
| TMO-003 | implemented | `sc-hooks-cli/src/timeout.rs` | timeout tests | |
| TMO-004 | implemented | `sc-hooks-cli/src/timeout.rs`, `sc-hooks-cli/src/audit.rs`, `sc-hooks-cli/src/handlers.rs`, `sc-hooks-sdk/src/manifest.rs` | timeout tests, audit tests, manifest validation tests, and `sc-hooks-cli/tests/long_running_contract.rs` agree on the sync-only `long_running` contract | |
| SES-001 | implemented | `sc-hooks-cli/src/session.rs` | session tests | |
| SES-002 | implemented | `sc-hooks-cli/src/main.rs`, `sc-hooks-cli/src/session.rs` | session tests | |
| MTA-001 | implemented | `sc-hooks-cli/src/metadata.rs` | metadata tests | |
| MTA-002 | implemented | `sc-hooks-cli/src/metadata.rs` | metadata tests, shim tests | |
| MTA-004 | implemented | `sc-hooks-cli/src/metadata.rs` | metadata tests | |
| MTA-005 | implemented | `sc-hooks-cli/src/metadata.rs` | `prepares_env_and_cleans_metadata_file_after_drop` | |
| CLI-001 | implemented | `sc-hooks-cli/src/main.rs` | command-specific tests across modules | |
| CLI-002 | implemented | `sc-hooks-cli/src/audit.rs` | audit tests | |
| CLI-003 | implemented | `sc-hooks-cli/src/fire.rs` | fire tests | |
| CLI-004 | implemented | `sc-hooks-cli/src/install.rs` | install tests plus `sc-hooks-cli/tests/runtime_layout_example.rs` proving the checked `.sc-hooks/` layout | |
| CLI-005 | implemented | `sc-hooks-cli/src/main.rs`, `sc-hooks-cli/src/handlers.rs`, `sc-hooks-core/src/exit_codes.rs` | config tests, handlers tests, exit-code table tests | |
| CLI-007 | implemented | `sc-hooks-test/src/compliance.rs`, `sc-hooks-cli/src/testing.rs`, `sc-hooks-cli/src/main.rs` | `sc-hooks-cli/tests/compliance_host.rs` proves `sc-hooks-test::compliance::run_contract_behavior_suite` through the real CLI binary, while `testing.rs` remains the thin presentation wrapper | |
| AUD-001 | implemented | `sc-hooks-cli/src/audit.rs` | audit tests | |
| AUD-002 | implemented | `sc-hooks-cli/src/audit.rs` | audit tests | |
| AUD-005 | implemented | `sc-hooks-cli/src/audit.rs`, `sc-hooks-sdk/src/manifest.rs` | `sc-hooks-cli/src/audit.rs` test `audit_rejects_async_long_running_manifest` plus `sc-hooks-sdk/src/manifest.rs` test `rejects_async_long_running_manifest` | |
| AUD-009 | implemented | `sc-hooks-cli/src/audit.rs`, `sc-hooks-sdk/src/manifest.rs` | `sc-hooks-cli/src/audit.rs` test `audit_rejects_long_running_without_description` plus `sc-hooks-sdk/src/manifest.rs` test `rejects_long_running_manifest_without_description` | |
| OBS-001 | implemented | `sc-hooks-cli/src/observability.rs`, `sc-hooks-cli/src/dispatch.rs`, `sc-hooks-cli/src/main.rs` | observability tests, dispatch tests, and integration tests proving degraded stderr signals for pre-dispatch failures | |
| OBS-002 | implemented | `sc-hooks-cli/src/observability.rs`, `sc-hooks-cli/src/config.rs` | observability tests, dispatch tests, and integration tests covering `mode = "off"` sink suppression | |
| OBS-005 | implemented | `sc-hooks-cli/src/observability.rs`, `sc-hooks-cli/src/dispatch.rs` | observability tests plus dispatch error-path tests covering `HandlerResultRecord` fields `handler_name`, `error_type`, elapsed time, and `disabled=true` | |
| BND-001 | implemented | `plugins/*/src/main.rs`, `plugins/agent-session-foundation/tests/session_foundation.rs`, `plugins/atm-extension/tests/atm_extension.rs`, `plugins/agent-spawn-gates/src/lib.rs`, `plugins/tool-output-gates/src/lib.rs` | behavior tests plus source inspection | |
| BND-001a | implemented | `plugins/*/Cargo.toml`, README, architecture docs | source inventory inspection | |
| BND-002 | implemented | `plugins/*/Cargo.toml`, README, architecture docs | release-facing docs and plugin metadata agree that no current `plugins/` source crate is shipped runtime functionality, so no unsupported shipped-plugin claim remains | |
| HKR-004 | deferred | docs-only Claude hook runtime planning (`docs/requirements.md`, `docs/archive/plugin-plan-s9.md`, `docs/phase-bc-hook-runtime-design.md`) | documentation inspection plus captured-fixture inventory for the seven locally reproduced non-`Notification` hook surfaces | |
| OBS-006 | implemented | `sc-hooks-cli/Cargo.toml`, `sc-hooks-cli/src/observability.rs` | build/test dependency integration plus observability tests | |
| OBS-007 | implemented | `sc-hooks-cli/src/observability.rs`, `sc-hooks-core/Cargo.toml`, `plugins/agent-session-foundation/Cargo.toml` | observability tests plus code inspection confirming logger config and sink routing now live only at the CLI boundary | |
| OBS-008 | implemented | `sc-hooks-cli/Cargo.toml` | dependency inspection | |
| OBS-009 | implemented | `sc-hooks-cli/src/observability.rs`, `sc-hooks-cli/tests/observability_contract.rs`, `docs/observability-contract.md`, `docs/logging-contract.md` | real dispatch-path observability tests plus contract/logging docs covering env-flag sink toggles | |
| DEF-008 | implemented | `sc-hooks-cli/tests/observability_contract.rs`, `docs/observability-contract.md`, `docs/logging-contract.md` | real dispatch-path observability tests prove both the JSONL file sink and the default console sink for success, block, invalid-json error, and timeout outcomes | |
| DEF-009 | implemented | `crates/sc-hooks-cli/src/dispatch.rs`, `crates/sc-hooks-cli/src/observability.rs`, `crates/sc-hooks-cli/tests/observability_contract.rs` | `standard_mode_emit_failure_is_non_blocking` and `full_mode_append_failure_is_non_blocking` prove forced emit and audit-append failures fall back to stderr without changing hook exits | |
| DEF-010 | implemented | `sc-hooks-cli/src/config.rs`, `docs/requirements.md`, `docs/architecture.md` | `layered_config_applies_built_in_global_local_and_env_precedence`, `default_global_config_path_uses_userprofile_when_home_is_missing`, and `rejects_unknown_observability_field` prove built-in < global < local < env precedence and the supported key surface | |
| DEF-011 | implemented | `sc-hooks-cli/src/config.rs`, `sc-hooks-cli/src/observability.rs`, `docs/requirements.md`, `docs/architecture.md` | `rejects_full_mode_from_global_config_alone`, `off_mode_returns_before_logger_initialization`, and `off_mode_suppresses_durable_observability_output` prove mode resolution and durable-sink suppression semantics | |
| DEF-012 | implemented | `sc-hooks-cli/src/observability.rs`, `docs/observability-contract.md` | `full_mode_writes_run_scoped_audit_files_for_success_dispatch` and `full_mode_path_override_writes_pre_dispatch_failure_record` prove run-scoped file layout and repo-local path override behavior | |
| DEF-013 | implemented | `sc-hooks-cli/src/observability.rs`, `sc-hooks-cli/src/config.rs`, `docs/observability-contract.md` | `full_mode_debug_profile_emits_machine_readable_strict_debug_fields`, `full_mode_debug_profile_permissive_still_requires_payload_capture_opt_in`, and `full_mode_debug_profile_payload_capture_is_bounded_when_enabled` prove debug-profile field emission and payload gating | |
| DEF-014 | implemented | `sc-hooks-cli/src/observability.rs`, `docs/observability-contract.md`, `docs/logging-contract.md` | `full_mode_debug_profile_emits_machine_readable_strict_debug_fields` and `full_mode_debug_profile_permissive_still_requires_payload_capture_opt_in` prove strict redaction defaults, explicit payload-capture gating, and audit JSONL as the machine-readable contract | |
| DEF-015 | implemented | `crates/sc-hooks-cli/src/dispatch.rs`, `crates/sc-hooks-cli/src/observability.rs`, `crates/sc-hooks-cli/tests/observability_contract.rs` | `standard_mode_logger_init_failure_is_non_blocking`, `standard_mode_emit_failure_is_non_blocking`, `full_mode_append_failure_is_non_blocking`, and `full_mode_prune_failure_is_non_blocking` prove degraded paths stay non-blocking | |
| DEF-016 | implemented | `sc-hooks-cli/tests/observability_contract.rs`, `docs/phase-observability-plan.md`, `docs/project-plan.md`, `docs/observability-soak-runbook.md` | `full_mode_concurrent_agents_shard_runs_without_corruption` proves 64 concurrent agents share one audit root without corruption, while the runbook defines the longer soak path | |
| DEF-017 | implemented | `sc-hooks-cli/src/main.rs`, `sc-hooks-cli/src/dispatch.rs`, `sc-hooks-cli/src/observability.rs` | `full_mode_zero_match_writes_audit_record`, `full_mode_path_override_writes_pre_dispatch_failure_record`, and `resolution_failure_emits_standard_degraded_signal` prove full-mode attempt accounting while standard keeps the lower-volume path | |
| DEF-018 | deferred | `docs/requirements.md`, `docs/phase-observability-plan.md` | kept out of the committed phase acceptance gate; future exporter defaults must not escalate to local `full` | observability follow-on |
| DEF-019 | implemented | `docs/requirements.md`, `docs/architecture.md`, `docs/project-plan.md` | naming cleanup landed in `SC-LOG-S1`, with `sc-hooks` frozen as canonical and `hooks` documented as alias-only language | |
| EXC-001 | implemented | `sc-hooks-cli/src/errors.rs`, `sc-hooks-core/src/exit_codes.rs` | exit-code table tests | |
| EXC-002 | implemented | `sc-hooks-cli/src/errors.rs`, `sc-hooks-core/src/exit_codes.rs` | exit-code table tests | |
| EXC-003 | implemented | `sc-hooks-cli/src/errors.rs`, `sc-hooks-core/src/exit_codes.rs` | exit-code table tests | |
| EXC-004 | implemented | `sc-hooks-cli/src/errors.rs`, `sc-hooks-cli/src/resolution.rs`, `sc-hooks-sdk/src/manifest.rs` | exit-code table tests, resolution tests | GAP-006 |
| EXC-005 | implemented | `sc-hooks-cli/src/errors.rs`, `sc-hooks-cli/src/dispatch.rs` | dispatch validation-path tests | |
| EXC-006 | implemented | `sc-hooks-cli/src/errors.rs`, `sc-hooks-cli/src/timeout.rs` | timeout tests | |
| EXC-007 | implemented | `sc-hooks-cli/src/errors.rs`, `sc-hooks-core/src/exit_codes.rs` | exit-code table tests | |
| EXC-008 | implemented | `sc-hooks-cli/src/errors.rs`, `sc-hooks-core/src/exit_codes.rs` | exit-code table tests | |
| TST-001 | implemented | workspace modules | distributed unit/integration tests | |
| TST-007 | implemented | `sc-hooks-test/src/compliance.rs`, `sc-hooks-cli/tests/compliance_host.rs` | shared `run_contract_behavior_suite` covers timeout, invalid stdout, multi-object warnings, async misuse, matcher filtering, and absent-payload behavior | |
| TST-008 | implemented | `scripts/verify-claude-hook-api.py`, `test-harness/hooks/claude/fixtures/approved/manifest.json` | `test-harness/hooks/claude/tests/test_version_bump_detector.py` plus direct `claude --version` comparison against the approved manifest | |
| PRT-001 | implemented | `.github/workflows/ci.yml` | CI workflow | |
| HKR-002 | implemented | `test-harness/hooks/claude/captures/raw/`, `test-harness/hooks/claude/tests/` | `test_fixture_validation.py`, `test_harness_structure.py` — harness structure and capture script contracts verified | |
| HKR-003 | implemented | `docs/archive/plugin-plan-s9.md`, `docs/hook-api/claude-hook-api.md` | `test-harness/run-schema-drift.py` drift detection; plan and hook API docs were revised from captured fixtures including `resume` and `clear` evidence | |
| HKR-008 | implemented | `sc-hooks-core/src/session.rs`, `plugins/agent-session-foundation/src/lib.rs`, `sc-hooks-core/src/storage.rs` | `sc-hooks-core/src/session.rs:429-534`, `plugins/agent-session-foundation` unit tests, and storage tests covering canonical-record validation, immutable root persistence, current-dir drift handling, and provider-root equality enforcement | |
| HKR-009 | implemented | `plugins/agent-session-foundation/src/lib.rs` | `plugins/agent-session-foundation` unit tests covering atomic-write temp-plus-rename, skip-on-unchanged, and per-invocation observability emission | |
| HKR-011 | implemented | `plugins/atm-extension/src/lib.rs` | `plugins/atm-extension` tests covering extension-field enrichment, team linkage, and child identity override behavior | |
| HKR-013 | implemented | `plugins/atm-extension/src/lib.rs` | `plugins/atm-extension` tests covering the four-stage relay pipeline, `ToolName` typed boundary, and relay-decision side-effect separation | |

## Resolved Gap Acknowledgments

| Gap | Status | Primary implementation | Primary tests or checks |
| --- | --- | --- | --- |
| GAP-001 | resolved | `sc-hooks-test/src/compliance.rs`, `sc-hooks-cli/tests/compliance_host.rs` | shared host-dispatch contract suite plus CLI delegation through `sc-hooks-cli/src/testing.rs` |
| GAP-002 | resolved | `sc-hooks-sdk/src/manifest.rs`, `sc-hooks-cli/src/timeout.rs`, `sc-hooks-cli/src/handlers.rs`, `sc-hooks-cli/src/audit.rs` | manifest validation tests, timeout tests, audit tests, handler discovery tests, and `sc-hooks-cli/tests/long_running_contract.rs` |
| GAP-003 | resolved | README, `docs/architecture.md`, `docs/requirements.md`, `plugins/*/Cargo.toml` | release-facing docs and plugin metadata consistently keep the legacy plugin crates scaffold/reference-only while separately classifying the Sprint 9 runtime crates |
| GAP-004 | resolved | `examples/runtime-layout/.sc-hooks/`, `examples/runtime-layout/README.md` | `sc-hooks-cli/tests/runtime_layout_example.rs` plus the checked example tree |
| GAP-005 | resolved | `sc-hooks-cli/src/observability.rs`, `sc-hooks-cli/src/dispatch.rs` | observability tests, dispatch tests, logging/observability contract docs |
| GAP-007 | resolved | `sc-hooks-cli/Cargo.toml`, `sc-hooks-cli/src/observability.rs` | dependency inspection, observability tests, architecture/requirements alignment |
| GAP-010 | resolved | `sc-hooks-cli/tests/observability_contract.rs`, `docs/implementation-gaps.md`, `docs/project-plan.md` | real dispatch-path observability tests plus the implementation-gap and project-plan follow-up notes agree on the file-sink baseline and the now-complete console-sink expansion |

## Requirement Amendment Notes

- `BND-001`
  - prior text: source-only plugin crates could remain described as non-runtime code “unless and until” a later phase promoted them
  - current text: every source crate under `plugins/` must be documented with an explicit maturity level of either scaffold/reference or runtime implementation with direct tests
  - authorizing sprint: `S9-BONUS`
- `HKR-008`
  - prior text: env-var availability of `CLAUDE_PROJECT_DIR` in hook process context was unverified; implementation of the canonical session-state record keyed by `ai_root_dir` was specified but not capture-backed
  - current text: `CLAUDE_PROJECT_DIR` is confirmed as a hook-only env injection (present in hook process env, absent in the launch shell); `SessionStart(source="startup")` is the capture-backed surface for establishing immutable root; later `cwd` values may drift; the canonical session-state model now enforces immutable-root persistence, root-equality checks, and normalized consumer output
  - authorizing sprints: `S9-ENV-CAPTURE`, `S10-R1`
- `BND-001a`
  - prior text: the documented `plugins/` source inventory listed nine crates and treated later additions as outside the branch baseline
  - current text: the documented `plugins/` source inventory lists all thirteen source crates in the branch and distinguishes the four non-scaffold runtime crates from the nine scaffold/reference crates
  - authorizing sprint: `S9-HP5`
- `OBS-009`
  - prior text: env-flag sink toggles were documented implementation details, not a named release-facing observability requirement
  - current text: env-flag sink toggles are promoted into the release-facing observability contract as `OBS-009`, with file sink canonical by default and console sink documented as the operator/debugging surface
  - authorizing sprint: `S9-BONUS`
- `CFG-002`
  - prior text: the top-level host config surface recognized `[meta]`, `[context]`, `[hooks]`, and `[sandbox]` only
  - current text: the top-level host config surface also recognizes `[observability]`, with the detailed field surface frozen in the observability contract
  - authorizing sprint: `SC-LOG-S2`
- `DEF-006`
  - prior text: observability config might return later as a separate `[logging]` section once the next planning pass defined the right surface
  - current text: superseded; the committed config surface is `[observability]`, and the project will not restore `[logging]` as a parallel persisted contract
  - authorizing sprint: `SC-LOG-S2`
- `DEF-010`
  - prior text: layered global/local/env observability config was part of the planned observability phase only
  - current text: layered observability config is implemented with built-in < global < local < env precedence
  - authorizing sprint: `SC-LOG-S2`
- `DEF-011`
  - prior text: `off | standard | full` mode resolution was part of the planned observability phase only
  - current text: mode resolution is implemented, and `full` remains invalid when it comes from global config alone
  - authorizing sprint: `SC-LOG-S2`
- `HKR-011`
  - prior text: ATM extension behavior could remain an ATM-owned state model as long as relay behavior was documented consistently
  - current text: ATM extension behavior shall enrich the canonical generic session-state record through extension fields and environment inheritance without redefining the generic lifecycle model
  - authorizing sprint: `S9-HP5`
- `HKR-013`
  - prior text: ATM relay handling could validate and route requests through one combined request type if tests still covered the visible outcomes
  - current text: ATM relay handling shall preserve distinct raw-request, validated-request, relay-decision, and relay-result stages so validation, routing, and side effects remain separately testable
  - authorizing sprint: `S9-HP5`
