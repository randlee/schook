---
id: P.7
title: Error, Boundary, And Portability Ruling Closeout
status: planned
branch: feature/pP-s7-error-and-boundary-rulings
worktree: ../schook-worktrees/feature/pP-s7-error-and-boundary-rulings
target: integrate/phase-P
---

# Sprint P.7 — Error, Boundary, And Portability Ruling Closeout

## Goal

- close the remaining active design-ruling items that still affect the
  core/sdk/cli boundary after the missing-hook runtime work lands

## Hard Dependencies

- accepted `P.6`

## Exact Targets

- `docs/requirements.md`
- `docs/architecture.md`
- `docs/implementation-gaps.md`
- `docs/traceability.md`
- `docs/cross-platform-guidelines.md`
- `crates/sc-hooks-core/`
- `crates/sc-hooks-sdk/`
- `crates/sc-hooks-cli/`
- `crates/sc-hooks-test/`

## Deliverables

- authoritative closure or explicit release-track disposition for:
  - `RULING-NEEDED-ECR-001`
  - `RULING-NEEDED-ECR-002`
  - `RULING-NEEDED-NT-CLI-002`
  - `RULING-NEEDED-HRN-005`
  - `RULING-NEEDED-COW-003`
- any required portability rulings needed to keep the runtime line honest about
  supported-platform behavior

## Acceptance Criteria

- each active ruling above is either closed by code/docs or deliberately
  deferred again with explicit rationale and owner
- control docs agree with the landed boundary/error posture
- no Unix-only behavior is left undocumented behind a generic “runtime parity”
  claim

## Out Of Scope

- new provider runtime surfaces
- `hooks` CLI alias
- exhausted retry-path coverage

## Required Validation

- `cargo check --workspace`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `just lint sc-portability`
- `git diff --check`
