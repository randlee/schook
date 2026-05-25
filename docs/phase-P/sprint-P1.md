---
id: P.1
title: Codex Missing-Hook Harness Expansion
status: complete
branch: feature/pP-s1-harness-expansion
worktree: ../schook-worktrees/feature/pP-s1-harness-expansion
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

## Sprint QA Checklist Answers

- Requirement IDs changed status:
  - `HKR-017` closed its Codex-side retained harness expansion in `P.1`
- Code removed early rather than left in parallel:
  - none; this sprint expanded the retained Codex harness surface without
    introducing an alternate runtime path
- Owned write scope:
  - `crates/sc-hooks-test/`
  - `test-harness/hooks/codex/`
  - `docs/requirements.md`
  - `docs/traceability.md`
- Validation commands and direct tests that proved the new contract:
  - `pytest test-harness/hooks/codex/tests/ -q`
  - `just test hooks codex`
  - `cargo test --workspace`
  - `git diff --check`
- Follow-on work blocked or unblocked by this sprint:
  - unblocks `P.2` Gemini-side harness closure on the same shared contract
  - contributes accepted Codex retained-surface evidence required by `P.4`

### Integration QA Questions

- AC traceability complete?
  - yes; `HKR-017` Codex-side closure is reflected in `docs/requirements.md`
    and `docs/traceability.md`, and the retained Codex harness evidence is
    linked from the sprint references
- No new `requirements.md` entries without `HKR-*` IDs?
  - yes; this sprint only updated existing `HKR-017` contract text and did
    not introduce a new unnamed requirement
- No new `implementation-gaps.md` entries without `RULING-NEEDED-*` IDs?
  - yes; `P.1` did not add any new implementation-gap entry
- All amendment notes in standalone three-field block format?
  - yes; the `HKR-017` Codex-side amendment is recorded as its own
    prior/current/authorizing-sprint block
- No `plan-phase-P.md` branch mismatches introduced?
  - yes; `P.1` still points to `feature/pP-s1-harness-expansion` in the plan
