---
id: N.9
title: Gemini API Document And Models Verification
status: planned
branch: feature/pN-s9-gemini-api-models-verify
worktree: ../schook-worktrees/feature/pN-s9-gemini-api-models-verify
target: integrate/phase-N
---

# Sprint N.9 — Gemini API Document And Models Verification

## Goal

- verify the Gemini API doc and Gemini models against the approved Gemini
  fixtures
- tighten only real drift, missing proof, or layout inconsistency

## Hard Dependencies

- `N.7` complete
- current `origin/integrate/phase-N` branch head

## Exact Targets

- `docs/hook-api/gemini-hook-api.md`
- `test_harness/hooks/gemini/models/`
- Gemini fixture-validation tests where model assertions live

## Deliverables

- verified `docs/hook-api/gemini-hook-api.md`
- verified Gemini Pydantic models
- verified Gemini model-validation tests

## Acceptance Criteria

- `docs/hook-api/gemini-hook-api.md` matches the approved Gemini fixtures
- Gemini Pydantic models validate the approved Gemini fixtures
- Gemini model/test layout follows the shared provider pattern

## Out Of Scope

- `just` integration

## Required Validation

- `pytest test-harness/hooks/gemini/tests/ -q`
- `pytest test-harness/hooks/ -q`
- `git diff --check`
