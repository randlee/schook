# sc-hooks Traceability

This table maps the most important documented requirements to current implementation, tests, or explicit gaps.

| Requirement ID | Status | Primary implementation | Primary tests | Gap |
| --- | --- | --- | --- | --- |
| CFG-001 | implemented | `sc-hooks-cli/src/config.rs` | `sc-hooks-cli/src/config.rs` tests | |
| CFG-002 | implemented | `sc-hooks-cli/src/config.rs` | `sc-hooks-cli/src/config.rs` tests | |
| CFG-003 | implemented | `sc-hooks-cli/src/config.rs`, `sc-hooks-cli/src/dispatch.rs` | dispatch tests, config tests | |
| CFG-004 | implemented | `sc-hooks-cli/src/config.rs` | `sc-hooks-cli/src/config.rs` tests | |
| RES-001 | implemented | `sc-hooks-cli/src/resolution.rs` | resolution tests | |
| RES-002 | implemented | `sc-hooks-cli/src/resolution.rs`, `sc-hooks-cli/src/handlers.rs` | resolution tests | GAP-004 |
| MTR-001 | implemented | `sc-hooks-cli/src/install.rs` | install tests | |
| PLC-001 | implemented | `sc-hooks-sdk/src/conditions.rs`, `sc-hooks-cli/src/resolution.rs` | condition tests, resolution tests | |
| PLG-001 | implemented | `sc-hooks-sdk/src/manifest.rs` | manifest tests | |
| PLG-003 | implemented | `sc-hooks-core/src/validation.rs`, `sc-hooks-sdk/src/manifest.rs` | manifest tests | |
| PLG-004 | implemented | `sc-hooks-sdk/src/manifest.rs` | manifest tests | |
| PLG-006 | implemented | `sc-hooks-core/src/results.rs`, `sc-hooks-cli/src/dispatch.rs` | dispatch tests | |
| PLG-009 | implemented | `sc-hooks-cli/src/dispatch.rs`, `sc-hooks-cli/src/audit.rs` | dispatch tests, audit tests | |
| PLG-011 | implemented | `sc-hooks-sdk/src/manifest.rs` | manifest tests | |
| ERR-004 | implemented | `sc-hooks-cli/src/dispatch.rs` | dispatch tests | |
| DSP-001 | implemented | `sc-hooks-cli/src/dispatch.rs` | dispatch tests | |
| DSP-002 | implemented | `sc-hooks-cli/src/dispatch.rs` | dispatch tests | |
| DSP-007 | implemented | `sc-hooks-cli/src/dispatch.rs` | dispatch tests | |
| DSP-008 | implemented | `sc-hooks-cli/src/main.rs`, `sc-hooks-cli/src/fire.rs` | fire tests, dispatch tests | |
| TMO-001 | implemented | `sc-hooks-cli/src/timeout.rs` | timeout tests | |
| TMO-003 | implemented | `sc-hooks-cli/src/timeout.rs` | timeout tests | |
| TMO-004 | gap | `sc-hooks-cli/src/timeout.rs`, `sc-hooks-cli/src/audit.rs`, `sc-hooks-sdk/src/traits.rs` | timeout tests only prove host side | GAP-002 |
| SES-001 | implemented | `sc-hooks-cli/src/session.rs` | session tests | |
| MTA-001 | implemented | `sc-hooks-cli/src/metadata.rs` | metadata tests | |
| MTA-004 | implemented | `sc-hooks-cli/src/metadata.rs` | metadata tests | |
| CLI-001 | implemented | `sc-hooks-cli/src/main.rs` | command-specific tests across modules | |
| CLI-002 | implemented | `sc-hooks-cli/src/audit.rs` | audit tests | |
| CLI-003 | implemented | `sc-hooks-cli/src/fire.rs` | fire tests | |
| CLI-004 | implemented | `sc-hooks-cli/src/install.rs` | install tests | GAP-004 |
| CLI-007 | gap | `sc-hooks-cli/src/testing.rs`, `sc-hooks-test/src/compliance.rs` | minimal compliance tests only | GAP-001 |
| AUD-001 | implemented | `sc-hooks-cli/src/audit.rs` | audit tests | |
| OBS-001 | implemented | `sc-hooks-cli/src/logging.rs`, `sc-hooks-cli/src/dispatch.rs` | logging tests, dispatch tests | |
| OBS-002 | implemented | `sc-hooks-cli/src/builtins/log.rs` | builtin log tests | GAP-005 |
| BND-001 | implemented | `plugins/*/src/main.rs` | source inspection only | GAP-003 |
| BND-002 | gap | `plugins/*` | no direct behavior tests | GAP-003 |
| TST-001 | implemented | workspace modules | distributed unit/integration tests | |
| TST-007 | gap | `sc-hooks-test/src/compliance.rs` | current harness tests | GAP-001 |
| PRT-001 | implemented | `.github/workflows/ci.yml` | CI workflow | |
