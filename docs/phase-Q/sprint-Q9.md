---
id: Q.9
title: opencode API Doc And Pydantic Models
status: planned
branch: feature/pQ-s9-opencode-doc-models
worktree: ../schook-worktrees/feature/pQ-s9-opencode-doc-models
target: integrate/phase-Q
---

# Sprint Q.9 — opencode API Doc And Pydantic Models

## Goal

- create the opencode provider API document
- add or tighten Pydantic models for retained opencode surfaces

## Hard Dependencies

- accepted `Q.8` opencode harness output

## Entry Criteria

- accepted `Q.8` opencode harness output

## Exact Targets

- `docs/hook-api/opencode-agent-hook-api.md`
- `test_harness/hooks/opencode/models/payloads.py`
- `test_harness/hooks/opencode/models/registry.py`
- `test-harness/hooks/opencode/tests/test_payload_models.py`

## Deliverables

- opencode API doc
- Pydantic payload models for retained opencode surfaces

## Required Contract Samples

Required opencode model entrypoint shape:

```python
from test_harness.hooks.opencode.models.payloads import ...
```

## Acceptance Criteria

- opencode API doc matches the retained harness-backed provider surfaces
- provider models and tests cover the retained surfaces explicitly

## Out Of Scope

- opencode runtime normalization
- opencode plugin parity
- opencode machine cutover

## Required Validation

- `pytest test-harness/hooks/opencode/tests/ -q`
- `cargo test --workspace`
- `git diff --check`

## Sprint QA Checklist

- Which requirement IDs or gap IDs changed status?
- What retained opencode surfaces are now explicit rather than implied?
- Which files or docs are the owned write scope for the sprint?
- What validation proves the API doc and models match the retained fixtures?
- What runtime work remains explicitly out of scope?
