---
id: P.8
title: CLI Alias And Retry-Coverage Closeout
status: planned
branch: feature/pP-s8-cli-and-retry-closeout
worktree: ../schook-worktrees/feature/pP-s8-cli-and-retry-closeout
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
  - `LOGR-QA-004`

## Acceptance Criteria

- `PRR-009` and `LOGR-QA-004` are closed or explicitly carried with a bounded
  release note
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
