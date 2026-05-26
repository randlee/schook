---
id: Q.9
title: opencode API Doc And Pydantic Models
status: completed
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
- `docs/requirements.md`
- `docs/traceability.md`
- `test_harness/hooks/opencode/models/payloads.py`
- `test_harness/hooks/opencode/models/registry.py`
- `test-harness/hooks/opencode/tests/test_payload_models.py`

## Deliverables

- opencode API doc
- Pydantic payload models for retained opencode surfaces

## Required Work

- update the opencode API doc to match the retained approved fixtures from
  `Q.8`
- treat the `Q.8` approved manifest as non-empty required input; `Q.9` does
  not close if the model suite would pass vacuously with zero retained events
- add one payload model class and one registry entry for every retained
  surface in the `Q.8` approved fixture manifest
- add payload-model tests that prove those models parse the approved fixtures

## Required Contract Samples

Required opencode model closure shape:

```text
surface_source:
  test-harness/hooks/opencode/fixtures/approved/manifest.json
model_rule:
  one payload model class per retained event in the approved manifest
registry_rule:
  test_harness/hooks/opencode/models/registry.py maps each retained event to
  its payload model
```

## Acceptance Criteria

- opencode API doc matches the retained harness-backed provider surfaces
- provider models and tests cover the retained surfaces explicitly
- the number of opencode payload model classes equals the retained event count
  in `test-harness/hooks/opencode/fixtures/approved/manifest.json`

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

## Sprint QA Checklist Answers

- Which requirement IDs or gap IDs changed status?
  `HKR-018` moves to `Implemented` in `docs/requirements.md` and
  `docs/traceability.md`; no gap row changes state in `Q.9`.
- What retained opencode surfaces are now explicit rather than implied?
  The retained `session.idle` surface is now explicit across the approved
  manifest, API doc, payload model, registry entry, and payload-model tests.
- Which files or docs are the owned write scope for the sprint?
  `docs/hook-api/opencode-agent-hook-api.md`, `docs/requirements.md`,
  `docs/traceability.md`, `test_harness/hooks/opencode/models/payloads.py`,
  `test_harness/hooks/opencode/models/registry.py`, and
  `test-harness/hooks/opencode/tests/test_payload_models.py`.
- What validation proves the API doc and models match the retained fixtures?
  `pytest test-harness/hooks/opencode/tests/ -q` proves the retained manifest
  fixture parses through the opencode payload model/registry path, and
  `cargo test --workspace` confirms the repo baseline remains green.
- What runtime work remains explicitly out of scope?
  opencode runtime normalization, plugin parity, and machine cutover all
  remain deferred beyond `Q.9`.
