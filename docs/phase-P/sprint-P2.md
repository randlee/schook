---
id: P.2
title: Gemini Missing-Hook Harness Expansion
status: complete
branch: feature/pP-s2-gemini-afteragent
worktree: ../schook-worktrees/feature/pP-s2-gemini-afteragent
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

## Sprint QA Checklist Answers

- Requirement IDs changed status:
  - `HKR-017` closed its Gemini-side retained harness expansion in `P.2`
- Code removed early rather than left in parallel:
  - none; this sprint promoted `AfterAgent` into the maintained Gemini harness
    contract without adding a second runtime path
- Owned write scope:
  - `test-harness/hooks/gemini/`
  - `docs/hook-api/gemini-hook-api.md`
  - `docs/requirements.md`
  - `docs/traceability.md`
- Validation commands and direct tests that proved the new contract:
  - `pytest test-harness/hooks/gemini/tests/ -q`
  - `just test hooks gemini`
  - `cargo test --workspace`
  - `git diff --check`
- Follow-on work blocked or unblocked by this sprint:
  - closes the Gemini harness side of `HKR-017`
  - unblocks `P.4` lifecycle-family normalization from accepted Gemini
    `AfterAgent` evidence

### Integration QA Questions

- AC traceability complete?
  - yes; `HKR-017` Gemini-side closure is reflected in `docs/requirements.md`
    and `docs/traceability.md`, and the retained Gemini harness evidence is
    linked from the sprint references
- No new `requirements.md` entries without `HKR-*` IDs?
  - yes; this sprint only updated existing `HKR-017` contract text and did
    not introduce a new unnamed requirement
- No new `implementation-gaps.md` entries without `RULING-NEEDED-*` IDs?
  - yes; `P.2` did not add any new implementation-gap entry
- All amendment notes in standalone three-field block format?
  - yes; the `HKR-017` Gemini-side amendment is recorded as its own
    prior/current/authorizing-sprint block
- No `plan-phase-P.md` branch mismatches introduced?
  - yes; `P.2` still points to `feature/pP-s2-gemini-afteragent` in the plan
