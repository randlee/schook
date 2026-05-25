---
id: Q.6
title: Cursor Agent Hook Harness
status: planned
branch: feature/pQ-s6-cursor-harness
worktree: ../schook-worktrees/feature/pQ-s6-cursor-harness
target: integrate/phase-Q
---

# Sprint Q.6 — Cursor Agent Hook Harness

## Purpose

- expand the permanent provider harness to cover `Cursor Agent`
- reuse the existing `cursor-agent` naming boundary instead of creating a
  second competing provider tree

## Entry Criteria

- accepted `Q.2` smoke infrastructure

## Exact Targets

- `test-harness/hooks/cursor-agent/`
- `test_harness/hooks/cursor-agent/`
- `docs/requirements.md`
- `docs/traceability.md`

## Deliverables

- approved Cursor fixtures
- Cursor provider models
- Cursor harness tests
- control-doc updates that record the new maintained harness scope

## Acceptance Criteria

- Cursor Agent is represented as a maintained harness provider, not only a
  deferred doc reference
- requirement and traceability updates land with the harness expansion

## Required Validation

- `cargo test --workspace`
- `git diff --check`
