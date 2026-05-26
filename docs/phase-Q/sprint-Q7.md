---
id: Q.7
title: Cursor Agent API Doc And Pydantic Models
status: completed
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

## Required Work

- update the Cursor Agent API doc to match the retained approved fixtures from
  `Q.6`
- treat the `Q.6` approved manifest as non-empty required input; `Q.7` does
  not close if the model suite would pass vacuously with zero retained events
- add one payload model class and one registry entry for every retained
  surface in the `Q.6` approved fixture manifest
- add payload-model tests that prove those models parse the approved fixtures

## Required Contract Samples

Required Cursor model closure shape:

```text
surface_source:
  test-harness/hooks/cursor-agent/fixtures/approved/manifest.json
model_rule:
  one payload model class per retained event in the approved manifest
registry_rule:
  test_harness/hooks/cursor_agent/models/registry.py maps each retained event
  to its payload model
```

## Acceptance Criteria

- Cursor API doc matches the retained harness-backed provider surfaces
- provider models and tests cover the retained surfaces explicitly
- the number of Cursor payload model classes equals the retained event count
  in `test-harness/hooks/cursor-agent/fixtures/approved/manifest.json`

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

## Sprint QA Checklist Answers

- Which requirement IDs or gap IDs changed status?
  No requirement or gap row changes state in `Q.7`; `HKR-007` remains
  `Partially Implemented` because `Q.7` closes the model/doc layer on top of
  the `Q.6` harness baseline rather than authorizing runtime work.
- What retained Cursor surfaces are now explicit rather than implied?
  The retained `stop` surface is now explicit across the approved manifest, API
  doc, payload model, registry entry, and payload-model tests.
- Which files or docs are the owned write scope for the sprint?
  `docs/hook-api/cursor-agent-hook-api.md`,
  `test_harness/hooks/cursor_agent/models/payloads.py`,
  `test_harness/hooks/cursor_agent/models/registry.py`, and
  `test-harness/hooks/cursor-agent/tests/test_payload_models.py`.
- What validation proves the API doc and models match the retained fixtures?
  `pytest test-harness/hooks/cursor-agent/tests/ -q` proves the retained
  manifest fixture parses through the Cursor payload model/registry path, and
  `cargo test --workspace` and `git diff --check` confirm the repo baseline
  remains green.
- What runtime work remains explicitly out of scope?
  Cursor runtime normalization, plugin parity, and machine cutover all remain
  deferred beyond `Q.7`.
