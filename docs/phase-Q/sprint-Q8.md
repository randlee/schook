---
id: Q.8
title: opencode Hook Harness
status: planned
branch: feature/pQ-s8-opencode-harness
worktree: ../schook-worktrees/feature/pQ-s8-opencode-harness
target: integrate/phase-Q
---

# Sprint Q.8 — opencode Hook Harness

## Goal

- add `opencode` as a maintained harness provider with approved fixtures,
  harness tests, and control-doc ownership
- land the missing control-doc ownership for opencode provider scope

## Hard Dependencies

- accepted `Q.6` Cursor harness output

## Entry Criteria

- accepted `Q.6` Cursor harness output

## Exact Targets

- `test-harness/hooks/opencode/fixtures/`
- `test-harness/hooks/opencode/hooks/`
- `test-harness/hooks/opencode/schema/`
- `test-harness/hooks/opencode/tests/test_harness_structure.py`
- `test-harness/hooks/opencode/tests/test_fixture_validation.py`
- `docs/requirements.md`
- `docs/traceability.md`

## Deliverables

- approved opencode fixtures
- opencode harness tests
- control-doc updates that introduce and record the opencode provider scope

## Required Work

- capture and approve retained opencode fixtures
- land the opencode harness-side hook scripts, schema files, structure checks,
  and fixture-validation tests
- update only the `HKR-018` requirement and matching traceability row
- extend the Q.6-owned `test-harness/hooks/README.md` with the opencode entry
  without reopening ownership of the full README

## Required Contract Samples

Required opencode harness layout:

```text
test-harness/hooks/opencode/
  fixtures/
  hooks/
  schema/
  tests/
```

## Acceptance Criteria

- opencode is represented as a maintained harness provider
- the phase does not rely on undocumented implied opencode scope
- the shared harness README is extended incrementally from the `Q.6` baseline
  rather than being claimed as a competing write-scope owner

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
- What previously undocumented scope is now explicit?
- Which files or docs are the owned write scope for the sprint?
- What validation proves opencode is a maintained harness provider now?
- What runtime work remains explicitly out of scope?
