# sc-hooks-core Requirements

## 1. Purpose

This document defines the `sc-hooks-core` crate ownership boundary.

It owns shared typed models and invariant enforcement that the rest of the
workspace depends on, without redefining the top-level product contract.

## 2. Crate Requirements

| ID | Status | Requirement | Implements product IDs | Primary files |
| --- | --- | --- | --- | --- |
| `REQ-SHK-CORE-001` | Implemented | The core crate shall own the shared manifest, hook-result, event, matcher, validation, and dispatch-mode types used across the workspace. | `REQ-SHK-PLG-002`, `REQ-SHK-PLG-003`, `REQ-SHK-PLG-006`, `REQ-SHK-PLG-013`, `REQ-SHK-PLG-014`, `REQ-SHK-PLC-*` | `sc-hooks-core/src/manifest.rs`, `sc-hooks-core/src/events.rs`, `sc-hooks-core/src/validation.rs`, `sc-hooks-core/src/dispatch.rs` |
| `REQ-SHK-CORE-002` | Implemented | The core crate shall own the shared exit-code table and error-shape primitives used by the CLI boundary. | `REQ-SHK-EXC-*` | `sc-hooks-core/src/exit_codes.rs`, `sc-hooks-core/src/errors.rs` |
| `REQ-SHK-CORE-003` | Deferred | The core crate shall own canonical session/root/state types and the invariants on those types, while leaving storage policy and logging ownership to higher layers. Deferred pending `HKR-008` / `HKR-012` — see `docs/requirements.md`. | `REQ-SHK-HKR-008`, `REQ-SHK-HKR-012` | `sc-hooks-core/src/session.rs`, `sc-hooks-core/src/storage.rs` |
| `REQ-SHK-CORE-004` | Deferred | The core crate shall remain the future home for any finalized typed hook-plugin trait surface that supersedes raw `serde_json::Value` passthrough. | `REQ-SHK-HKR-009` | `sc-hooks-core/src/context.rs`, `sc-hooks-core/src/results.rs` |

## 3. Ownership Notes

- `sc-hooks-core` owns typed models and invariant checks.
- `sc-hooks-core` does not own process spawning, sink initialization, or CLI
  parsing.
