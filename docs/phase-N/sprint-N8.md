---
id: N.8
title: Codex API Document And Models Verification
status: completed
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
- `N.7` is parallel, not sequential; the Gemini API-doc/model scope is disjoint
  from this Codex API-doc/model scope
- current `origin/integrate/phase-N` branch head

## Exact Targets

- `docs/hook-api/codex-hook-api.md`
- `test_harness/hooks/codex/models/`
- Codex fixture-validation tests where model assertions live

## Deliverables

- `docs/hook-api/codex-hook-api.md`
- `test_harness/hooks/codex/models/payloads.py`
- `test-harness/hooks/codex/tests/`

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
