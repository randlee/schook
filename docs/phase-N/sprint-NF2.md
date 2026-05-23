---
id: NF2
title: Harness Validation And Gemini Import Gate
status: planned
branch: feature/pN-fix-s2-harness-validation-gate
worktree: ../schook-worktrees/feature/pN-fix-s2-harness-validation-gate
target: integrate/phase-N
---

# Sprint NF2 — Harness Validation And Gemini Import Gate

## Goal

- restore real Codex and Gemini fixture-validation proof on the integrated
  Phase N baseline
- fix the Gemini module import blocker in the same harness pass
- close the remaining Claude capture-path sanitization requirement while the
  harness promotion path is open

## Hard Dependencies

- `NF1` merged to `integrate/phase-N`
- `docs/phase-N/plan-remediation.md`

## Exact Targets

- `test-harness/hooks/codex/tests/`
- `test-harness/hooks/gemini/tests/`
- `test-harness/hooks/gemini/hooks/session_start.py`
- `test-harness/hooks/claude/hooks/_capture_common.py`
- `test_harness/hooks/`

## Deliverables

Every listed deliverable is expected to land at a production-ready level for
the scope this sprint claims. If that cannot be done cleanly in one sprint, the
sprint must be split before implementation begins. No deliverable may be
silently dropped or partially deferred.

- `BP-HARNESS-NEW-001` fixed with real fixture-validation test modules for
  Codex and Gemini
- `GEMINI-MODULE-IMPORT` fixed in the Gemini `session_start.py` import path
- `BP-HARNESS-NEW-008` closed by enforcing or explicitly documenting cwd/PWD
  sanitization in the approved-fixture promotion path

## Required Work

- add concrete `pytest` modules under `test-harness/hooks/codex/tests/` and
  `test-harness/hooks/gemini/tests/` that validate approved fixtures against
  the provider models
- prove pytest collects and executes the new files on the integrated branch
- fix the Gemini `session_start.py` module import failure without adding
  provider-specific ad hoc path hacks
- make the Claude promotion path explicitly sanitize cwd/PWD before approved
  fixture promotion, or document and test the exact sanctioned promotion rule

## Explicit Code Samples

```python
def test_approved_payload_fixtures_validate_against_models(...) -> None: ...
```

```python
def test_capture_scripts_write_raw_payload_and_env_files(...) -> None: ...
```

The end state must be real provider-specific fixture/model tests, not only
empty package scaffolding.

## This Sprint Does Not Close

- `SCHOOK-QA-PN-002`
- `PN-PRR-001`
- `BP-CORE-005`
- `BP-CLI-001`
- `BP-CLI-003`
- `BP-HARNESS-NEW-005`

## Acceptance Criteria

- pytest collects non-empty Codex and Gemini test modules from the harness
  provider directories
- the new tests validate approved fixtures against the existing provider-model
  contracts rather than only asserting file presence
- the Gemini `session_start.py` path no longer fails with
  `ModuleNotFoundError`
- the Claude approved-fixture promotion path has one authoritative sanitization
  rule for cwd/PWD and that rule is either enforced in code or locked down in
  test-backed promotion documentation

## Required Validation

- `pytest test-harness/hooks/codex/tests/ -q`
- `pytest test-harness/hooks/gemini/tests/ -q`
- `pytest test-harness/hooks/ -q`
- `cargo test --workspace`
- `git diff --check`
