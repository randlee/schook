---
id: P.7
title: Error, Boundary, And Portability Ruling Closeout
status: complete
branch: feature/pP-s7-ruling-closeout
worktree: ../schook-worktrees/feature/pP-s7-ruling-closeout
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
  - `RULING-NEEDED-HRN-005`
  - `RULING-NEEDED-COW-003`
- portability rulings for any Unix-only behavior introduced by `P.5` or `P.6`
  that is not already documented in those sprints

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

## Sprint QA Checklist

- Which requirement IDs or gap IDs changed status?
- What code was removed early rather than left in parallel?
- Which files/crates were the owned write scope for the sprint?
- What validation commands and direct tests proved the new contract?
- What follow-on work is blocked or unblocked by this sprint?

## Sprint QA Checklist Answers

- Requirement and gap status changes:
  - `TMO-003` now records the real platform split: Unix sends `SIGTERM`, then
    force-kills after the grace window; non-Unix uses platform-native
    termination and the same bounded timeout contract.
  - `RULING-NEEDED-ECR-001` and `RULING-NEEDED-ECR-002` are now explicitly
    deferred past `Phase P` with recorded owner and release-track rationale.
  - `RULING-NEEDED-HRN-005` and `RULING-NEEDED-COW-003` are now closed with
    explicit accepted posture.
- Code removed early:
  - none; this sprint was a ruling and portability closeout, not a runtime
    rewrite.
- Owned write scope:
  - `docs/requirements.md`
  - `docs/architecture.md`
  - `docs/implementation-gaps.md`
  - `docs/traceability.md`
  - `docs/cross-platform-guidelines.md`
  - `docs/plan-phase-P.md`
  - `docs/phase-P/sprint-P7.md`
- Validation that passed:
  - `cargo check --workspace`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test --workspace`
  - `just lint sc-portability`
  - `git diff --check`
- Follow-on status:
  - `P.7` closes the active ruling set named in the Phase P plan and leaves
    only the explicitly separate release-operator follow-ons such as
    `PRR-009` and `LOGR-QA-004`.
