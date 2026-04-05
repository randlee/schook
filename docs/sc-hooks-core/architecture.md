# sc-hooks-core Architecture

## 1. Purpose

This document records the `sc-hooks-core` architectural boundary.

## 2. Ownership Boundary

`sc-hooks-core` owns:

- manifest and hook-result model types
- hook/event taxonomy types
- shared validation and condition primitives
- exit-code catalog data
- canonical session/root/state types

`sc-hooks-core` does not own:

- CLI flag parsing
- process spawn or timeout behavior
- logger setup or sink routing
- SDK-only authoring conveniences

## 3. Core ADRs

| ID | Decision | Notes |
| --- | --- | --- |
| `ADR-SHK-CORE-001` | The public contract is serialized JSON and documented environment variables, not Rust enum names. | `sc-hooks-core` types are the internal typed model behind that contract. |
| `ADR-SHK-CORE-002` | Session/root/state invariants are expressed through validated types, not by trusting raw payload strings. | The core crate carries the canonical typed model for root, session, and lifecycle state. |

## 4. Planned Observability Phase Boundary

The upcoming observability phase does not move config loading, sink routing, or
export ownership into `sc-hooks-core`.

If the phase introduces additional shared invocation, correlation, or
redaction-friendly metadata types, those types must remain sink-agnostic and
must not make `sc-hooks-core` responsible for logger lifecycle.
