---
id: Q.6
title: Cursor Agent Hook Harness
status: planned
branch: feature/pQ-s6-cursor-harness
worktree: ../schook-worktrees/feature/pQ-s6-cursor-harness
target: integrate/phase-Q
---

# Sprint Q.6 — Cursor Agent Hook Harness

## Goal

- expand the permanent provider harness to cover `Cursor Agent`
- reuse the existing `cursor-agent` naming boundary instead of creating a
  second competing provider tree

## Hard Dependencies

- accepted `Q.2` smoke infrastructure

## Entry Criteria

- accepted `Q.2` smoke infrastructure

## Exact Targets

- `test-harness/hooks/cursor-agent/fixtures/`
- `test-harness/hooks/cursor-agent/hooks/`
- `test-harness/hooks/cursor-agent/schema/`
- `test-harness/hooks/cursor-agent/tests/test_harness_structure.py`
- `test-harness/hooks/cursor-agent/tests/test_fixture_validation.py`
- `test-harness/hooks/README.md`
- `docs/requirements.md`
- `docs/traceability.md`

## Deliverables

- approved Cursor fixtures
- Cursor harness tests
- control-doc updates that record the new maintained harness scope

## Required Contract Samples

Required Cursor harness layout:

```text
test-harness/hooks/cursor-agent/
  fixtures/
  hooks/
  schema/
  tests/
```

## Acceptance Criteria

- Cursor Agent is represented as a maintained harness provider, not only a
  deferred doc reference
- requirement and traceability updates land with the harness expansion
- the sprint reuses the existing `cursor-agent` naming boundary everywhere; no
  parallel `cursor/` provider tree is introduced

## Out Of Scope

- Cursor runtime normalization
- Cursor plugin parity
- Cursor machine cutover

## Required Validation

- `pytest test-harness/hooks/cursor-agent/tests/ -q`
- `cargo test --workspace`
- `git diff --check`

## Sprint QA Checklist

- Which requirement IDs or gap IDs changed status?
- What competing path or naming choice was rejected early?
- Which files or docs are the owned write scope for the sprint?
- What validation proves Cursor is a maintained harness provider now?
- What runtime work remains explicitly out of scope?
