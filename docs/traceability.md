# sc-hooks Traceability

This table maps the most important documented requirements to current implementation, tests, or explicit gaps.

| Requirement ID | Status | Primary implementation | Primary tests | Gap |
| --- | --- | --- | --- | --- |
| CFG-001 | implemented | `sc-hooks-cli/src/config.rs` | `sc-hooks-cli/src/config.rs` tests plus `sc-hooks-cli/tests/runtime_layout_example.rs` proving the checked default `.sc-hooks/config.toml` layout | |
| CFG-002 | implemented | `sc-hooks-cli/src/config.rs` | `sc-hooks-cli/src/config.rs` tests | |
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
| DSP-008 | implemented | `sc-hooks-cli/src/main.rs`, `sc-hooks-cli/src/fire.rs` | fire tests, dispatch tests | |
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
| OBS-001 | implemented | `sc-hooks-cli/src/observability.rs`, `sc-hooks-cli/src/dispatch.rs` | observability tests, dispatch tests | |
| OBS-002 | implemented | `sc-hooks-cli/src/observability.rs` | observability tests, dispatch tests | |
| OBS-005 | implemented | `sc-hooks-cli/src/observability.rs`, `sc-hooks-cli/src/dispatch.rs` | observability tests plus dispatch error-path tests covering `HandlerResultRecord` fields `handler_name`, `error_type`, elapsed time, and `disabled=true` | |
| BND-001 | implemented | `plugins/*/src/main.rs` | source inspection only | |
| BND-001a | implemented | `plugins/*/Cargo.toml`, README, architecture docs | source inventory inspection | |
| BND-002 | implemented | `plugins/*/Cargo.toml`, README, architecture docs | release-facing docs and plugin metadata agree that no current `plugins/` source crate is shipped runtime functionality, so no unsupported shipped-plugin claim remains | |
| OBS-006 | implemented | `sc-hooks-cli/Cargo.toml`, `sc-hooks-cli/src/observability.rs` | build/test dependency integration plus observability tests | |
| OBS-007 | implemented | `sc-hooks-cli/src/observability.rs` | observability tests plus code inspection of crate boundaries | |
| OBS-008 | implemented | `sc-hooks-cli/Cargo.toml` | dependency inspection | |
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
| PRT-001 | implemented | `.github/workflows/ci.yml` | CI workflow | |

## Resolved Gap Acknowledgments

| Gap | Status | Primary implementation | Primary tests or checks |
| --- | --- | --- | --- |
| GAP-001 | resolved | `sc-hooks-test/src/compliance.rs`, `sc-hooks-cli/tests/compliance_host.rs` | shared host-dispatch contract suite plus CLI delegation through `sc-hooks-cli/src/testing.rs` |
| GAP-002 | resolved | `sc-hooks-sdk/src/manifest.rs`, `sc-hooks-cli/src/timeout.rs`, `sc-hooks-cli/src/handlers.rs`, `sc-hooks-cli/src/audit.rs` | manifest validation tests, timeout tests, audit tests, handler discovery tests, and `sc-hooks-cli/tests/long_running_contract.rs` |
| GAP-003 | resolved | README, `docs/architecture.md`, `docs/requirements.md`, `plugins/*/Cargo.toml` | release-facing docs and plugin metadata consistently mark every source crate as scaffold/reference only |
| GAP-004 | resolved | `examples/runtime-layout/.sc-hooks/`, `examples/runtime-layout/README.md` | `sc-hooks-cli/tests/runtime_layout_example.rs` plus the checked example tree |
| GAP-005 | resolved | `sc-hooks-cli/src/observability.rs`, `sc-hooks-cli/src/dispatch.rs` | observability tests, dispatch tests, logging/observability contract docs |
| GAP-007 | resolved | `sc-hooks-cli/Cargo.toml`, `sc-hooks-cli/src/observability.rs` | dependency inspection, observability tests, architecture/requirements alignment |
