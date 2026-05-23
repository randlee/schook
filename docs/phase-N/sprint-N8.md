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

## Production-Ready Expectation

Every listed deliverable is expected to land at a production-ready level for
the verification scope this sprint claims.

## Signature Sample

```python
def validate_codex_hook_payload(payload: Any) -> CodexHookPayload
def validate_codex_env_snapshot(payload: Any) -> CodexEnvSnapshot
def validate_codex_fixture_manifest(payload: Any) -> CodexFixtureManifest
```

## Acceptance Criteria

- the three listed deliverables are each either:
  - confirmed current from the approved Codex fixtures
  - or updated to match the approved Codex fixtures
- Codex model/test layout still follows the shared provider pattern

## Out Of Scope

- Gemini API doc and models
- `just` integration

## Required Validation

- `pytest test-harness/hooks/codex/tests/ -q`
- `pytest test-harness/hooks/ -q`
- `git diff --check`
