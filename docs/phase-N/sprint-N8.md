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

- verify the Codex API doc and Codex models against the approved Codex
  fixtures
- tighten only real drift, missing proof, or layout inconsistency

## Hard Dependencies

- `N.6` complete
- current `origin/integrate/phase-N` branch head

## Exact Targets

- `docs/hook-api/codex-hook-api.md`
- `test_harness/hooks/codex/models/`
- Codex fixture-validation tests where model assertions live

## Deliverables

- verified `docs/hook-api/codex-hook-api.md`
- verified Codex Pydantic models
- verified Codex model-validation tests

## Required Work

- verify the Codex doc directly against approved Codex fixtures
- verify model/test coverage against those same approved fixtures
- correct only real drift, missing fields, or layout inconsistency
- keep model locations and validation entrypoints aligned with the same shared
  provider harness contract

## This Sprint Does Not Close

- Gemini API doc and models
- `just` integration

## Acceptance Criteria

- `docs/hook-api/codex-hook-api.md` matches the approved Codex fixtures
- Codex Pydantic models validate the approved Codex fixtures
- Codex model/test layout follows the shared provider pattern

## Required Validation

- `pytest test-harness/hooks/codex/tests/ -q`
- `pytest test-harness/hooks/ -q`
- `git diff --check`
