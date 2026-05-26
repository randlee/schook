---
id: Q.6
title: Cursor Agent Hook Harness
status: completed
branch: feature/pQ-s6-cursor-harness
worktree: ../schook-worktrees/feature/pQ-s6-cursor-harness
target: integrate/phase-Q
---

# Sprint Q.6 — Cursor Agent Hook Harness

## Goal

- expand the permanent provider harness to cover `Cursor Agent`
- reuse the existing `cursor-agent` naming boundary instead of creating a
  second competing provider tree

## Hard Dependencies

- accepted `Q.2` smoke infrastructure

## Entry Criteria

- accepted `Q.2` smoke infrastructure

## Exact Targets

- `test-harness/hooks/cursor-agent/fixtures/`
- `test-harness/hooks/cursor-agent/fixtures/approved/manifest.json`
- `test-harness/hooks/cursor-agent/captures/raw/`
- `test-harness/hooks/cursor-agent/hooks/`
- `test-harness/hooks/cursor-agent/models/`
- `test-harness/hooks/cursor-agent/prompts/`
- `test-harness/hooks/cursor-agent/reports/`
- `test-harness/hooks/cursor-agent/schema/`
- `test-harness/hooks/cursor-agent/schema/README.md`
- `test-harness/hooks/cursor-agent/scripts/`
- `test-harness/hooks/cursor-agent/tests/test_harness_structure.py`
- `test-harness/hooks/cursor-agent/tests/test_fixture_validation.py`
- `test_harness/hooks/cursor_agent/`
- `test_harness/hooks/cursor_agent/models/`
- `test-harness/hooks/README.md`
- `docs/requirements.md`
- `docs/traceability.md`

## Deliverables

- approved Cursor fixtures
- Cursor harness tests
- control-doc updates that record the new maintained harness scope

## Required Work

- capture and approve retained Cursor fixtures under the existing
  `cursor-agent` evidence tree
- land the Cursor harness-side hook scripts, schema files, structure checks,
  and fixture-validation tests
- create the matching `test_harness/hooks/cursor_agent/` Python package root
  that `Q.7` extends with payload models and registry files
- update only the `HKR-007` requirement and matching traceability row
- establish the shared harness README baseline that later provider-harness
  sprints must extend rather than rewrite

## Required Contract Samples

Required Cursor harness layout:

```text
test-harness/hooks/cursor-agent/
  captures/raw/
  fixtures/
  hooks/
  models/
  prompts/
  reports/
  schema/
  scripts/
  tests/

test_harness/hooks/cursor_agent/
  models/
```

## Acceptance Criteria

- Cursor Agent is represented as a maintained harness provider, not only a
  deferred doc reference
- `test-harness/hooks/cursor-agent/fixtures/approved/manifest.json` exists and
  retains at least one approved Cursor event
- requirement and traceability updates land with the harness expansion
- the sprint reuses the existing `cursor-agent` naming boundary everywhere; no
  parallel `cursor/` provider tree is introduced
- the paired `test_harness/hooks/cursor_agent/` package root exists before
  `Q.7` begins model closure work
- `test-harness/hooks/README.md` carries the shared harness-contract update
  for Cursor and becomes the baseline that `Q.8` extends later

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
- What competing path or naming choice was rejected early?
- Which files or docs are the owned write scope for the sprint?
- What validation proves Cursor is a maintained harness provider now?
- What runtime work remains explicitly out of scope?

## Sprint QA Checklist Answers

- Which requirement IDs or gap IDs changed status?
  `HKR-007` moved from `Planned` to `Partially Implemented` in
  `docs/requirements.md` and `docs/traceability.md`; no gap row changed state.
- What competing path or naming choice was rejected early?
  A parallel `cursor/` provider tree was rejected; the sprint keeps the
  existing `cursor-agent` evidence path paired with the `cursor_agent` Python
  package path only.
- Which files or docs are the owned write scope for the sprint?
  `pyproject.toml`, `docs/requirements.md`, `docs/traceability.md`,
  `docs/phase-Q/sprint-Q6.md`, `test-harness/hooks/README.md`, and the
  `test-harness/hooks/cursor-agent/` plus `test_harness/hooks/cursor_agent/`
  harness/package roots.
- What validation proves Cursor is a maintained harness provider now?
  `pytest test-harness/hooks/cursor-agent/tests/ -q` proves the approved
  manifest, required harness layout, and paired package root all exist, while
  `cargo test --workspace` confirms the repo-wide baseline still passes.
- What runtime work remains explicitly out of scope?
  Cursor runtime normalization, plugin parity, and machine cutover all remain
  deferred beyond `Q.6`.
