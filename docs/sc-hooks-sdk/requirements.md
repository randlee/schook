# sc-hooks-sdk Requirements

## 1. Purpose

This document defines the `sc-hooks-sdk` ownership boundary.

The SDK is an authoring convenience layer for Rust plugin authors. It does not
define the release contract on its own.

## 2. Crate Requirements

| ID | Status | Requirement | Implements product IDs | Primary files |
| --- | --- | --- | --- | --- |
| `REQ-SHK-SDK-001` | Implemented | The SDK shall provide Rust helpers for manifest parsing/building, condition helpers, runner wiring, and result construction. | `REQ-SHK-PLG-002`, `REQ-SHK-PLG-003`, `REQ-SHK-PLC-002`, `REQ-SHK-DEF-003` | `sc-hooks-sdk/src/manifest.rs`, `sc-hooks-sdk/src/conditions.rs`, `sc-hooks-sdk/src/runner.rs`, `sc-hooks-sdk/src/result.rs` |
| `REQ-SHK-SDK-002` | Implemented | SDK conveniences shall not redefine or silently broaden the executable host contract documented in the product docs. | `REQ-SHK-PLG-013`, `REQ-SHK-PLG-014`, `REQ-SHK-TMO-004` | `sc-hooks-sdk/src/runner.rs`, `sc-hooks-sdk/src/lib.rs` |
| `REQ-SHK-SDK-003` | Deferred | Any richer typed plugin-trait surface beyond the current helper layer must be frozen explicitly before it becomes part of the hook-runtime baseline. | `REQ-SHK-HKR-009` | `sc-hooks-sdk/src/traits.rs` |
| `REQ-SHK-SDK-004` | Deferred | Any future SDK helper that consumes audit metadata shall remain a convenience layer and shall not own layered config, sink wiring, or exporter behavior. | `REQ-SHK-DEF-013`, `REQ-SHK-DEF-014` | future SDK helpers only if explicitly approved |
