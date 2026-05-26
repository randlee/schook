---
id: Q.8
title: opencode Hook Harness
status: completed
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
- `test-harness/hooks/opencode/fixtures/approved/manifest.json`
- `test-harness/hooks/opencode/captures/raw/`
- `test-harness/hooks/opencode/hooks/`
- `test-harness/hooks/opencode/models/`
- `test-harness/hooks/opencode/prompts/`
- `test-harness/hooks/opencode/reports/`
- `test-harness/hooks/opencode/schema/`
- `test-harness/hooks/opencode/scripts/`
- `test-harness/hooks/opencode/tests/test_harness_structure.py`
- `test-harness/hooks/opencode/tests/test_fixture_validation.py`
- `test_harness/hooks/opencode/`
- `test_harness/hooks/opencode/models/`
- `test-harness/hooks/README.md` (`Q.6` owns the file; `Q.8` appends the
  opencode entry only)
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
- create the matching `test_harness/hooks/opencode/` Python package root that
  `Q.9` extends with payload models and registry files
- update only the `HKR-018` requirement and matching traceability row
- extend the Q.6-owned `test-harness/hooks/README.md` with the opencode entry
  without reopening ownership of the full README

## Required Contract Samples

Required opencode harness layout:

```text
test-harness/hooks/opencode/
  captures/raw/
  fixtures/
  hooks/
  models/
  prompts/
  reports/
  schema/
  scripts/
  tests/

test_harness/hooks/opencode/
  models/
```

## Acceptance Criteria

- opencode is represented as a maintained harness provider
- `test-harness/hooks/opencode/fixtures/approved/manifest.json` exists and
  retains at least one approved opencode event
- the phase does not rely on undocumented implied opencode scope
- the paired `test_harness/hooks/opencode/` package root exists before `Q.9`
  begins model closure work
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

## Sprint QA Checklist Answers

- Which requirement IDs or gap IDs changed status?
  No requirement or gap row changes state in `Q.8`; `HKR-018` remains
  `Planned` because `Q.8` closes the harness/package layer while `Q.9` still
  owns the provider-local model/doc closure.
- What previously undocumented scope is now explicit?
  `opencode` is now explicit as a maintained harness-only provider with the
  retained `session.idle` approved-reference surface, a non-empty approved
  manifest, provider tests, and a paired Python-package root.
- Which files or docs are the owned write scope for the sprint?
  `pyproject.toml`, `docs/traceability.md`, `docs/phase-Q/sprint-Q8.md`,
  `test-harness/hooks/README.md`, and the `test-harness/hooks/opencode/` plus
  `test_harness/hooks/opencode/` harness/package roots. The `Q.6` entry
  criterion was already satisfied before `Q.8` began: `Q.6` landed on
  `feature/pQ-s6-cursor-harness` and PR `#162`, which is the accepted Cursor
  harness baseline this sprint extends.
- What validation proves opencode is a maintained harness provider now?
  `pytest test-harness/hooks/opencode/tests/ -q` proves the approved manifest,
  required harness layout, and paired package root exist, while
  `cargo test --workspace` confirms the repo-wide baseline still passes.
- What runtime work remains explicitly out of scope?
  opencode runtime normalization, plugin parity, and machine cutover all
  remain deferred beyond `Q.8`.
