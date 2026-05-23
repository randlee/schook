---
id: N.8
title: Codex API Document And Models Verification
status: planned
branch: feature/pN-s8-codex-api-models-verify
worktree: ../schook-worktrees/feature/pN-s8-codex-api-models-verify
target: integrate/phase-N
---

# Sprint N.8 — Codex API Document And Models Verification

## Goal

- leave the Codex API-doc and model deliverables current against the approved
  Codex fixtures

## Hard Dependencies

- `N.6` complete
- current `origin/integrate/phase-N` branch head

## Exact Targets

- `docs/hook-api/codex-hook-api.md`
- `test_harness/hooks/codex/models/`
- Codex fixture-validation tests where model assertions live

## Deliverables

- `docs/hook-api/codex-hook-api.md`
- `test_harness/hooks/codex/models/payloads.py`
- `test-harness/hooks/codex/tests/`

## Acceptance Criteria

- `docs/hook-api/codex-hook-api.md` matches the approved Codex fixtures
- `test_harness/hooks/codex/models/payloads.py` validates the approved Codex
  fixtures
- `test-harness/hooks/codex/tests/` validates the approved Codex fixtures
- Codex model/test layout follows the shared provider pattern

## Out Of Scope

- Gemini API doc and models
- `just` integration

## Required Validation

- `pytest test-harness/hooks/codex/tests/ -q`
- `pytest test-harness/hooks/ -q`
- `git diff --check`
