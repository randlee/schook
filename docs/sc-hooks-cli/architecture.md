# sc-hooks-cli Architecture

## 1. Purpose

This document describes the `sc-hooks-cli` crate boundary and the architectural
decisions that stay local to the host binary.

## 2. Ownership Boundary

`sc-hooks-cli` owns:

- CLI parsing in `sc-hooks-cli/src/main.rs`
- config loading and normalization in `sc-hooks-cli/src/config.rs`
- handler resolution and matcher filtering in `sc-hooks-cli/src/resolution.rs`
- metadata discovery and environment injection in `sc-hooks-cli/src/metadata.rs`
- process spawn, stdout/stderr parsing, timeout handling, and session-disable
  policy in `sc-hooks-cli/src/dispatch.rs`, `sc-hooks-cli/src/timeout.rs`, and
  `sc-hooks-cli/src/session.rs`
- static audit behavior in `sc-hooks-cli/src/audit.rs`
- install output generation in `sc-hooks-cli/src/install.rs`
- observability sink setup and event emission in `sc-hooks-cli/src/observability.rs`

`sc-hooks-cli` does not own:

- the serialized wire contract types and shared validation rules
- reusable Rust authoring helpers for external plugin authors
- reusable compliance harness fixtures

## 3. CLI ADRs

| ID | Decision | Notes |
| --- | --- | --- |
| `ADR-SHK-CLI-001` | The host remains a process-based dispatcher, not an in-process plugin runtime. | Child-process execution, stdin/stdout JSON, and timeout enforcement stay in `sc-hooks-cli`. |
| `ADR-SHK-CLI-002` | The CLI crate is the only logging boundary. | Lower crates expose typed data and errors; logger setup and sink ownership stay here. |
| `ADR-SHK-CLI-003` | Audit remains static analysis, not simulated hook execution. | Audit checks config, manifest, metadata satisfiability, and install surfaces without executing live hook logic. |

## 4. Planned Observability Phase Boundary

The upcoming observability phase keeps these future responsibilities inside
`sc-hooks-cli`:

- layered merge of built-in defaults, `~/.sc-hooks/config.toml`,
  `.sc-hooks/config.toml`, and environment overrides
- observability-mode resolution for `off`, `standard`, and `full`
- audit sink orchestration, run-scoped file layout, retention pruning, and
  degraded-path handling
- any future machine-readable stream or exporter wiring at the CLI boundary

The phase does not move sink ownership into lower crates.
It also does not redefine the existing `sc-hooks audit` command, which remains
the static-analysis command surface unless a later CLI plan says otherwise.
