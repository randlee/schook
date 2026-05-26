---
id: P.8
title: CLI Alias And Retry-Coverage Closeout
status: complete
branch: feature/pP-s8-observability-qa
worktree: ../schook-worktrees/feature/pP-s8-observability-qa
target: integrate/phase-P
---

# Sprint P.8 — CLI Alias And Retry-Coverage Closeout

## Goal

- close the remaining operator-facing packaging/CLI/testing gaps after runtime
  parity and boundary decisions are settled

## Hard Dependencies

- accepted `P.7`

## Exact Targets

- install/packaging path for the `hooks` alias
- `docs/implementation-gaps.md`
- `README.md`
- `USAGE.md`
- `PUBLISHING.md`
- retry coverage under `crates/sc-hooks-core/` and `crates/sc-hooks-test/`

## Deliverables

- `hooks` CLI alias implemented or explicitly downgraded from supported release
  language
- exhausted retry-path coverage for the shared spawn helper
- final closeout updates for:
  - `PRR-009`
  - `LOGR-QA-004`, because `P.8` intentionally owns the exhausted-retry-path
    test closure for the shared spawn helper rather than leaving that signed-off
    conditional as a future touch-only follow-up

## Acceptance Criteria

- `PRR-009` and `LOGR-QA-004` are closed or explicitly carried with a bounded
  release note; `LOGR-QA-004` is not treated as implicitly reopened unless this
  sprint actually lands the named exhausted-retry-path coverage
- shared spawn-helper retry behavior is production-verified with explicit test
  coverage for the retained retry paths, not only shaped in code
- operator docs match the actual install/alias behavior

## Out Of Scope

- new provider runtime surfaces
- Cursor runtime work

## Required Validation

- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `git diff --check`

## Sprint QA Checklist

- Which requirement IDs or gap IDs changed status?
- What code was removed early rather than left in parallel?
- Which files/crates were the owned write scope for the sprint?
- What validation commands and direct tests proved the new contract?
- What follow-on work is blocked or unblocked by this sprint?

## Sprint QA Checklist Answers

- Requirement and gap status changes:
  - `DEF-019` now points to the actual alias install behavior rather than
    alias-only naming language with no install mechanism.
  - `PRR-009` is closed in `P.8`.
  - `LOGR-QA-004` is closed in `P.8`.
- Code removed early:
  - none; `P.8` closed packaging/install and retry-coverage gaps without
    removing a parallel runtime path.
- Owned write scope:
  - `crates/sc-hooks-core/src/process.rs`
  - `crates/sc-hooks-cli/src/install.rs`
  - `docs/implementation-gaps.md`
  - `docs/requirements.md`
  - `docs/traceability.md`
  - `docs/plan-phase-P.md`
  - `docs/phase-P/sprint-P8.md`
  - `README.md`
  - `USAGE.md`
  - `PUBLISHING.md`
- Validation that passed:
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test --workspace`
  - `git diff --check`
- Follow-on status:
  - `P.8` closes the remaining active Phase P operator-facing gap items on this
    branch; any future alias expansion into additional release channels belongs
    to release automation follow-on work, not the missing-hook parity track.
