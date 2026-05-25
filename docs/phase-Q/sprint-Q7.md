---
id: Q.7
title: Cursor Agent API Doc And Pydantic Models
status: planned
branch: feature/pQ-s7-cursor-doc-models
worktree: ../schook-worktrees/feature/pQ-s7-cursor-doc-models
target: integrate/phase-Q
---

# Sprint Q.7 — Cursor Agent API Doc And Pydantic Models

## Goal

- promote the Cursor Agent API document from planning reference to
  harness-backed provider artifact
- add or tighten Pydantic models for retained Cursor surfaces

## Hard Dependencies

- accepted `Q.6` Cursor harness output

## Entry Criteria

- accepted `Q.6` Cursor harness output

## Exact Targets

- `docs/hook-api/cursor-agent-hook-api.md`
- `test_harness/hooks/cursor_agent/models/payloads.py`
- `test_harness/hooks/cursor_agent/models/registry.py`
- `test-harness/hooks/cursor-agent/tests/test_payload_models.py`

## Deliverables

- updated Cursor Agent API doc
- Pydantic payload models for retained Cursor surfaces

## Required Contract Samples

Required Cursor model entrypoint shape:

```python
from test_harness.hooks.cursor_agent.models.payloads import ...
```

## Acceptance Criteria

- Cursor API doc matches the retained harness-backed provider surfaces
- provider models and tests cover the retained surfaces explicitly

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
- What retained Cursor surfaces are now explicit rather than implied?
- Which files or docs are the owned write scope for the sprint?
- What validation proves the API doc and models match the retained fixtures?
- What runtime work remains explicitly out of scope?
