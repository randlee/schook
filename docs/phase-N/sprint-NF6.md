---
id: NF6
title: Codex Pending Record Parse Hardening
status: planned
branch: feature/pN-fix-s6-codex-pending-record-hardening
worktree: ../schook-worktrees/feature/pN-fix-s6-codex-pending-record-hardening
target: integrate/phase-N
---

# Sprint NF6 — Codex Pending Record Parse Hardening

## Goal

- harden persisted Codex debounce-state parsing so malformed records fail with
  structured errors instead of bare `KeyError` behavior

## Hard Dependencies

- `NF1` merged to `integrate/phase-N`
- `docs/phase-N/plan-remediation.md`

## Exact Targets

- `test_harness/hooks/codex/debounce.py`
- any directly coupled Codex debounce tests

## Deliverables

Every listed deliverable is expected to land at a production-ready level for
the scope this sprint claims. If that cannot be done cleanly in one sprint, the
sprint must be split before implementation begins. No deliverable may be
silently dropped or partially deferred.

- `BP-HARNESS-NEW-005` fixed by wrapping missing-key decode failures in a
  structured parse error path

## Required Work

- replace bare dict-key lookup in `PendingRecord.from_json`
- capture the missing-field case as a structured error with enough context to
  identify the broken pending record
- add or update tests for malformed pending-record JSON input

## Explicit Code Samples

```python
try:
    due_at = payload["due_at"]
except KeyError as exc:
    raise PendingRecordDecodeError(field=str(exc), path=record_path) from exc
```

## This Sprint Does Not Close

- any Phase N closure-record reconciliation
- any core or CLI semantics findings outside the Codex debounce parser

## Acceptance Criteria

- malformed pending-record JSON no longer escapes as a bare `KeyError`
- tests assert the structured malformed-record path directly
- the happy-path debounce behavior remains unchanged

## Required Validation

- `pytest test-harness/hooks/codex/tests/ -q`
- `pytest test-harness/hooks/ -q`
- `git diff --check`
