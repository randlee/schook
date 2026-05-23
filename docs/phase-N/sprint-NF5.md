---
id: NF5
title: CLI Error Surface And Timeout Invariant Hardening
status: planned
branch: feature/pN-fix-s5-cli-error-hardening
worktree: ../schook-worktrees/feature/pN-fix-s5-cli-error-hardening
target: integrate/phase-N
---

# Sprint NF5 — CLI Error Surface And Timeout Invariant Hardening

## Goal

- harden the CLI error-reporting surface for internal failures
- remove or explicitly guard the timeout invariant that currently relies on a
  bare `expect(...)`

## Hard Dependencies

- `integrate/phase-N` merged baseline at `df2794f`
- `docs/phase-N/plan-remediation.md`

## Exact Targets

- `crates/sc-hooks-cli/src/errors.rs`
- `crates/sc-hooks-cli/src/dispatch.rs`
- `crates/sc-hooks-cli/src/observability.rs`
- any directly coupled CLI tests

## Deliverables

Every listed deliverable is expected to land at a production-ready level for
the scope this sprint claims. If that cannot be done cleanly in one sprint, the
sprint must be split before implementation begins. No deliverable may be
silently dropped or partially deferred.

- `BP-CLI-001` fixed by giving `CliError::Internal` actionable recovery
  guidance
- `BP-CLI-003` fixed by replacing the bare timeout `expect(...)` dependence
  with an explicit guarded error path or a locally enforced invariant
- `BP-CLI-002`, `BP-CLI-004`, and `BP-CLI-005` closed if any remain after
  `NF1`

## Required Work

- define the intended operator-facing recovery text for internal CLI errors
- decide whether the timeout invariant should be made impossible locally or
  downgraded into a structured dispatch error
- keep the invariant enforcement near the establishment site rather than
  relying on cross-module assumptions alone
- close any remaining small CLI consistency items in the same pass

## Explicit Code Samples

```rust
CliError::Internal {
    context: "...".into(),
    recovery: Some("contact maintainer or rerun with ...".into()),
}
```

```rust
let timeout_ms = timeout_ms.ok_or_else(|| DispatchError::MissingTimeoutInvariant { ... })?;
```

## This Sprint Does Not Close

- `BP-HARNESS-NEW-005`

## Acceptance Criteria

- `CliError::Internal` no longer renders as an unrecoverable dead-end without
  operator guidance
- the cited timeout path no longer relies on an unchecked cross-module
  `expect(...)`
- CLI tests cover the adjusted internal-error or missing-timeout behavior

## Required Validation

- `cargo test -p sc-hooks-cli`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `git diff --check`
