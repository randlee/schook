# sc-hooks-cli Requirements

## 1. Purpose

This document defines the `sc-hooks-cli` crate ownership boundary.

It does not restate the full product contract from
[docs/requirements.md](../requirements.md). Instead, it names the CLI-layer
responsibilities that satisfy the referenced product requirements.

## 2. Crate Requirements

| ID | Status | Requirement | Implements product IDs | Primary files |
| --- | --- | --- | --- | --- |
| `REQ-SHK-CLI-001` | Implemented | The CLI crate shall own repo-relative config loading, config parsing, and config-to-runtime normalization for the host process. | `REQ-SHK-CFG-*` | `sc-hooks-cli/src/config.rs`, `sc-hooks-cli/src/main.rs` |
| `REQ-SHK-CLI-002` | Implemented | The CLI crate shall own handler resolution, matcher filtering, payload-condition evaluation, and manifest loading for external executable plugins under `.sc-hooks/plugins/`. | `REQ-SHK-RES-*`, `REQ-SHK-MTR-*`, `REQ-SHK-PLC-*`, `REQ-SHK-PLG-001`, `REQ-SHK-PLG-011` | `sc-hooks-cli/src/resolution.rs`, `sc-hooks-cli/src/handlers.rs`, `sc-hooks-cli/src/events.rs` |
| `REQ-SHK-CLI-003` | Implemented | The CLI crate shall own child-process dispatch, timeout enforcement, stderr/stdout parsing, session-disable policy, and CLI exit-code selection. | `REQ-SHK-DSP-*`, `REQ-SHK-TMO-*`, `REQ-SHK-SES-*`, `REQ-SHK-ERR-004`, `REQ-SHK-EXC-*` | `sc-hooks-cli/src/dispatch.rs`, `sc-hooks-cli/src/timeout.rs`, `sc-hooks-cli/src/session.rs`, `sc-hooks-cli/src/errors.rs` |
| `REQ-SHK-CLI-004` | Implemented | The CLI crate shall own runtime metadata discovery, metadata temp-file lifecycle, and environment injection for plugin subprocesses. | `REQ-SHK-MTA-*` | `sc-hooks-cli/src/metadata.rs` |
| `REQ-SHK-CLI-005` | Implemented | The CLI crate shall own static audit execution, install-plan generation, and the retained user-facing command surface. | `REQ-SHK-CLI-001`, `REQ-SHK-CLI-002`, `REQ-SHK-AUD-*` | `sc-hooks-cli/src/audit.rs`, `sc-hooks-cli/src/install.rs`, `sc-hooks-cli/src/main.rs` |
| `REQ-SHK-CLI-006` | Implemented | The CLI crate shall remain the only workspace crate that initializes logging sinks, emits structured `sc-observability` dispatch events, and emits deterministic degraded stderr signals when standard-mode pre-dispatch failures prevent `dispatch.complete`. | `REQ-SHK-OBS-*` | `sc-hooks-cli/src/observability.rs`, `sc-hooks-cli/src/dispatch.rs`, `sc-hooks-cli/src/main.rs` |
| `REQ-SHK-CLI-007` | Implemented | The CLI crate shall provide the real host-dispatch path that the reusable compliance harness exercises. | `REQ-SHK-TST-007` | `sc-hooks-cli/src/testing.rs`, `sc-hooks-cli/tests/compliance_host.rs` |
| `REQ-SHK-CLI-008` | Implemented | The CLI crate shall own layered observability-config loading across built-in defaults, global `~/.sc-hooks/config.toml`, repo-local `.sc-hooks/config.toml`, and environment overrides, including the supported `[observability]` key surface. | `REQ-SHK-DEF-010`, `REQ-SHK-DEF-011` | `sc-hooks-cli/src/config.rs`, `sc-hooks-cli/src/main.rs`, config tests |
| `REQ-SHK-CLI-009` | Implemented | The CLI crate shall own the lean full-audit sink orchestration, run-scoped file layout, and full-mode attempt accounting without affecting hook execution; this runtime audit-trail work remains distinct from the existing static `sc-hooks audit` command. | `REQ-SHK-DEF-012`, `REQ-SHK-DEF-017`, `REQ-SHK-DEF-019` | `sc-hooks-cli/src/observability.rs`, `sc-hooks-cli/src/dispatch.rs`, `sc-hooks-cli/src/main.rs`, `sc-hooks-cli/tests/observability_contract.rs` |
| `REQ-SHK-CLI-010` | Implemented | The CLI crate shall own the debug-profile audit extensions, redaction policy enforcement, and payload-capture gating without introducing a public sink-extension API. | `REQ-SHK-DEF-013`, `REQ-SHK-DEF-014` | `sc-hooks-cli/src/observability.rs`, `sc-hooks-cli/src/config.rs`, `sc-hooks-cli/src/dispatch.rs`, `sc-hooks-cli/tests/observability_contract.rs` |
| `REQ-SHK-CLI-011` | Implemented | The CLI crate shall complete the remaining audit-phase ownership for retention pruning, degraded-path hardening, and load validation without introducing a public sink-extension API. | `REQ-SHK-DEF-015`, `REQ-SHK-DEF-016` | `sc-hooks-cli/src/observability.rs`, `sc-hooks-cli/tests/observability_contract.rs` (`full_mode_prunes_run_directories_by_age_and_count`, `full_mode_concurrent_agents_shard_runs_without_corruption`) |

## 3. Ownership Notes

- `sc-hooks-cli` owns process orchestration and operator-facing command
  behavior.
- `sc-hooks-cli` does not own the serialized contract types; those belong to
  [docs/sc-hooks-core/requirements.md](../sc-hooks-core/requirements.md).
- `sc-hooks-cli` may use `sc-hooks-sdk` conveniences internally, but those
  conveniences do not redefine the host contract.
