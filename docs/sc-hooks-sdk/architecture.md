# sc-hooks-sdk Architecture

## 1. Purpose

This document describes the `sc-hooks-sdk` crate boundary.

## 2. Ownership Boundary

`sc-hooks-sdk` owns:

- Rust-first helper APIs for plugin authors
- manifest builder and parser helpers
- condition-evaluation helpers
- runner helpers and result helpers

`sc-hooks-sdk` does not own:

- the release-defining executable contract
- host dispatch semantics
- sink setup or observability ownership

## 3. SDK ADRs

| ID | Decision | Notes |
| --- | --- | --- |
| `ADR-SHK-SDK-001` | The SDK remains a convenience layer, not the normative contract surface. | If SDK behavior diverges from the host contract, the host contract wins. |
| `ADR-SHK-SDK-002` | Runner helpers may smooth authoring ergonomics, but they must not hide missing required runtime fields in the real host path. | This keeps CLI/runtime semantics authoritative. |
