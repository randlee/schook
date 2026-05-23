---
id: NF4
title: Core Condition Error Semantics
status: planned
branch: feature/pN-fix-s4-core-condition-semantics
worktree: ../schook-worktrees/feature/pN-fix-s4-core-condition-semantics
target: integrate/phase-N
---

# Sprint NF4 — Core Condition Error Semantics

## Goal

- correct the operator-error classification in `sc-hooks-core`
- close the linked small core follow-up while the same module is under review

## Hard Dependencies

- `integrate/phase-N` merged baseline at `df2794f`
- `docs/phase-N/plan-remediation.md`

## Exact Targets

- `crates/sc-hooks-core/src/conditions.rs`
- any directly coupled tests for `ConditionError`

## Deliverables

Every listed deliverable is expected to land at a production-ready level for
the scope this sprint claims. If that cannot be done cleanly in one sprint, the
sprint must be split before implementation begins. No deliverable may be
silently dropped or partially deferred.

- `BP-CORE-005` fixed by returning `ConditionError::UnsupportedOperator`
  instead of `InvalidValue` for the cited unsupported-operator paths
- `BP-CORE-006` closed if it remains only a small local clarification in the
  same module

## Required Work

- replace the wrong catch-all error variant in the cited operator-dispatch
  paths
- confirm the surrounding tests prove the distinction between unsupported
  operator and invalid condition value
- add or update tests so the corrected variant is directly asserted

## Explicit Code Samples

```rust
match operator {
    /* supported operators */
    _ => Err(ConditionError::UnsupportedOperator { ... }),
}
```

## This Sprint Does Not Close

- `BP-CLI-001`
- `BP-CLI-003`
- `BP-HARNESS-NEW-005`

## Acceptance Criteria

- the cited `conditions.rs` paths no longer map unsupported operators to
  `InvalidValue`
- test coverage directly proves the new `UnsupportedOperator` path
- no unrelated condition-evaluation behavior changes are introduced

## Required Validation

- `cargo test -p sc-hooks-core`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `git diff --check`
