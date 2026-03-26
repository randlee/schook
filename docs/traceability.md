# sc-hooks Traceability

This table maps the most important documented requirements to current implementation, tests, or explicit gaps.

| Requirement ID | Status | Primary implementation | Primary tests | Gap |
| --- | --- | --- | --- | --- |
| CFG-001 | implemented | `sc-hooks-cli/src/config.rs` | `sc-hooks-cli/src/config.rs` tests | |
| CFG-002 | implemented | `sc-hooks-cli/src/config.rs` | `sc-hooks-cli/src/config.rs` tests | |
| CFG-003 | implemented | `sc-hooks-cli/src/config.rs`, `sc-hooks-cli/src/dispatch.rs` | dispatch tests, config tests | |
| CFG-004 | implemented | `sc-hooks-cli/src/config.rs` | `sc-hooks-cli/src/config.rs` tests | |
| CFG-008 | implemented | `sc-hooks-cli/src/config.rs`, `sc-hooks-cli/src/audit.rs` | config tests, audit tests | |
| RES-001 | implemented | `sc-hooks-cli/src/resolution.rs` | resolution tests | |
| RES-002 | implemented | `sc-hooks-cli/src/resolution.rs`, `sc-hooks-cli/src/handlers.rs` | resolution tests | GAP-004 |
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
| TMO-004 | gap | `sc-hooks-cli/src/timeout.rs`, `sc-hooks-cli/src/audit.rs`, `sc-hooks-sdk/src/manifest.rs` | timeout tests only prove host side; Sprint 1 retired stale `LongRunning`/`AsyncContextSource` traits but did not yet close the end-to-end contract | GAP-002 |
| SES-001 | implemented | `sc-hooks-cli/src/session.rs` | session tests | |
| SES-002 | implemented | `sc-hooks-cli/src/main.rs`, `sc-hooks-cli/src/session.rs` | session tests | |
| MTA-001 | implemented | `sc-hooks-cli/src/metadata.rs` | metadata tests | |
| MTA-002 | implemented | `sc-hooks-cli/src/metadata.rs` | metadata tests, shim tests | |
| MTA-004 | implemented | `sc-hooks-cli/src/metadata.rs` | metadata tests | |
| MTA-005 | implemented | `sc-hooks-cli/src/metadata.rs` | `prepares_env_and_cleans_metadata_file_after_drop` | |
| CLI-001 | implemented | `sc-hooks-cli/src/main.rs` | command-specific tests across modules | |
| CLI-002 | implemented | `sc-hooks-cli/src/audit.rs` | audit tests | |
| CLI-003 | implemented | `sc-hooks-cli/src/fire.rs` | fire tests | |
| CLI-004 | implemented | `sc-hooks-cli/src/install.rs` | install tests | GAP-004 |
| CLI-005 | implemented | `sc-hooks-cli/src/main.rs`, `sc-hooks-cli/src/handlers.rs`, `sc-hooks-core/src/exit_codes.rs` | config tests, handlers tests, exit-code table tests | |
| CLI-007 | gap | `sc-hooks-test/src/compliance.rs`, `sc-hooks-cli/src/testing.rs`, `sc-hooks-cli/src/main.rs` | `sc-hooks-test` owns the compliance engine; `sc-hooks-cli/src/testing.rs` is now a thin presentation wrapper over the shared checks | GAP-001 |
| AUD-001 | implemented | `sc-hooks-cli/src/audit.rs` | audit tests | |
| AUD-002 | implemented | `sc-hooks-cli/src/audit.rs` | audit tests | |
| OBS-001 | implemented | `sc-hooks-cli/src/observability.rs`, `sc-hooks-cli/src/dispatch.rs` | observability tests, dispatch tests | |
| OBS-002 | implemented | `sc-hooks-cli/src/observability.rs` | observability tests, dispatch tests | |
| BND-001 | implemented | `plugins/*/src/main.rs` | source inspection only | GAP-003 |
| BND-001a | implemented | `plugins/*/Cargo.toml`, README, architecture docs | source inventory inspection | |
| BND-002 | gap | `plugins/*` | no direct behavior tests | GAP-003 |
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
| TST-007 | gap | `sc-hooks-test/src/compliance.rs`, `sc-hooks-cli/src/testing.rs` | shared harness currently proves manifest, contract, matcher, timeout-shape, and minimal JSON behavior; direct absent-payload and other protocol branches remain Sprint 2 work | GAP-001 |
| PRT-001 | implemented | `.github/workflows/ci.yml` | CI workflow | |

## Resolved Gap Acknowledgments

| Gap | Status | Primary implementation | Primary tests or checks |
| --- | --- | --- | --- |
| GAP-005 | resolved | `sc-hooks-cli/src/observability.rs`, `sc-hooks-cli/src/dispatch.rs` | observability tests, dispatch tests, logging/observability contract docs |
| GAP-007 | resolved | `sc-hooks-cli/Cargo.toml`, `sc-hooks-cli/src/observability.rs` | dependency inspection, observability tests, architecture/requirements alignment |
