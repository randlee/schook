---
id: P.2
title: Gemini Missing-Hook Harness Expansion
status: planned
branch: feature/pP-s2-gemini-missing-hook-harness
worktree: ../schook-worktrees/feature/pP-s2-gemini-missing-hook-harness
target: integrate/phase-P
---

# Sprint P.2 — Gemini Missing-Hook Harness Expansion

## Goal

- add Gemini `AfterAgent` to the permanent harness as a fully verified retained
  surface
- close the Gemini-side `HKR-017` harness-contract expansion with matching
  `docs/requirements.md` and `docs/traceability.md` updates

## Hard Dependencies

- accepted `integrate/phase-O` baseline
- current Gemini harness under `test-harness/hooks/gemini/`
- current Gemini API doc in `docs/hook-api/gemini-hook-api.md`

## Exact Targets

- `test-harness/hooks/gemini/`
- `test_harness/hooks/gemini/`
- `docs/hook-api/gemini-hook-api.md`
- `docs/requirements.md`
- `docs/traceability.md`

## Deliverables

- approved `AfterAgent` Gemini fixtures
- Gemini model coverage for `AfterAgent`
- Gemini tests that exercise the `AfterAgent` harness path
- updated Gemini API doc and findings docs that treat `AfterAgent` as a real
  supported surface
- updated `docs/requirements.md` rows for the retained Gemini `AfterAgent`
  surface
- updated `docs/traceability.md` rows for the retained Gemini `AfterAgent`
  surface
- `HKR-017` amendment text and traceability evidence updated for the retained
  Gemini `AfterAgent` harness surface

## Acceptance Criteria

- `AfterAgent` is no longer just historical evidence; it is present in the
  maintained Gemini fixture/model/test surface
- `docs/hook-api/gemini-hook-api.md` matches the retained Gemini surface set
- `docs/requirements.md` records the retained Gemini `AfterAgent` surface with
  no stale pre-Phase-P wording
- `docs/traceability.md` records the retained Gemini `AfterAgent` surface with
  no stale pre-Phase-P wording
- the retained Gemini `AfterAgent` harness work is recorded as `HKR-017`
  progress, with requirement and traceability updates landing in the same PR
- Gemini fixture/model/test coverage is green including `AfterAgent`

## Out Of Scope

- Gemini runtime parity implementation
- Codex hook work
- lifecycle normalization changes

## Required Validation

- `pytest test-harness/hooks/gemini/tests/ -q`
- `just test hooks gemini`
- `git diff --check`

## Sprint QA Checklist

- Which requirement IDs or gap IDs changed status?
- What code was removed early rather than left in parallel?
- Which files/crates were the owned write scope for the sprint?
- What validation commands and direct tests proved the new contract?
- What follow-on work is blocked or unblocked by this sprint?
