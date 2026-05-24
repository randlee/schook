---
id: P.1
title: Codex Missing-Hook Harness Expansion
status: planned
branch: feature/pP-s1-codex-missing-hook-harness
worktree: ../schook-worktrees/feature/pP-s1-codex-missing-hook-harness
target: integrate/phase-P
---

# Sprint P.1 — Codex Missing-Hook Harness Expansion

## Goal

- add the remaining Codex lifecycle surfaces to the permanent harness
- turn the current deferred/missing Codex hook set into explicit verified
  evidence, approved fixtures, and tests
- close the Codex-side `HKR-017` harness-contract expansion with matching
  `docs/requirements.md` and `docs/traceability.md` updates

## Hard Dependencies

- accepted `integrate/phase-O` baseline
- current Codex harness under `test-harness/hooks/codex/`
- current Codex API doc in `docs/hook-api/codex-hook-api.md`

## Exact Targets

- `test-harness/hooks/codex/`
- `test_harness/hooks/codex/`
- `docs/hook-api/codex-hook-api.md`
- `docs/requirements.md`
- `docs/traceability.md`

## Deliverables

- harness support for Codex `notify`
- harness support for Codex `Stop`
- harness support for Codex `resume`
- harness support for Codex `fork` if the surface is still worth carrying; if
  not, a documented disposition with evidence
- approved Codex fixtures, models, and tests for every retained surface
- updated `docs/hook-api/codex-hook-api.md` that matches the real harness state
- updated `docs/requirements.md` rows for retained or deferred Codex lifecycle
  surfaces
- updated `docs/traceability.md` rows for retained or deferred Codex lifecycle
  surfaces
- `HKR-017` amendment text and traceability evidence updated for the retained
  Codex missing-hook harness surface

## Acceptance Criteria

- every missing Codex hook surface is accounted for as one of:
  - approved and fixture-backed
  - confirmed unsupported / non-exercisable with explicit evidence
  - explicitly deferred again with a justified ruling
- `docs/hook-api/codex-hook-api.md` matches the retained Codex surfaces
- `docs/requirements.md` records the retained or deferred Codex lifecycle
  surfaces with no stale pre-Phase-P wording
- `docs/traceability.md` records the retained or deferred Codex lifecycle
  surfaces with no stale pre-Phase-P wording
- the retained Codex missing-hook harness work is recorded as `HKR-017`
  progress, with requirement and traceability updates landing in the same PR
- Codex fixture/model/test coverage is green for the retained surfaces

## Out Of Scope

- Codex runtime parity implementation
- Gemini hook work
- lifecycle normalization changes

## Required Validation

- `pytest test-harness/hooks/codex/tests/ -q`
- `just test hooks codex`
- `git diff --check`

## Sprint QA Checklist

- Which requirement IDs or gap IDs changed status?
- What code was removed early rather than left in parallel?
- Which files/crates were the owned write scope for the sprint?
- What validation commands and direct tests proved the new contract?
- What follow-on work is blocked or unblocked by this sprint?
