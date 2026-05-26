---
id: Q.5
title: Gemini Smoke Tests
status: completed
branch: feature/pQ-s5-gemini-smoke
worktree: ../schook-worktrees/feature/pQ-s5-gemini-smoke
target: integrate/phase-Q
---

# Sprint Q.5 — Gemini Smoke Tests

## Goal

- implement executable smoke coverage for the Gemini runtime path through the
  new `just smoke` surface

## Hard Dependencies

- accepted `Q.2` smoke infrastructure

## Entry Criteria

- accepted `Q.2` smoke infrastructure

## Exact Targets

- `.just/smoke/gemini.py`
- `.just/smoke/fixtures/gemini/`
- `docs/phase-Q/smoke-gemini.md`
- `docs/phase-Q/sprint-Q5.md`
- `docs/requirements.md`
- `docs/traceability.md`

## Deliverables

- end-to-end Gemini smoke scenarios covering retained live Gemini behavior
- smoke result record for Gemini on the accepted baseline

## Required Work

- add the Gemini-specific smoke module under `.just/smoke/`
- add the offline replay or dry-run assets required by the Q.2-owned CI smoke
  model under `.just/smoke/fixtures/gemini/`
- record the accepted-baseline Gemini smoke result in
  `docs/phase-Q/smoke-gemini.md`
- update the `HKR-019` traceability row as the single owning smoke sprint
  because `Q.5` is the final Claude/Codex/Gemini smoke closeout

## CI Execution Model

- `Q.5` does not change the Q.2 smoke surface contract
- generic CI continues to run `just smoke all ci` without a live Gemini CLI
- `Q.5` adds the Gemini replay or dry-run assets consumed by that CI-owned
  offline gate
- `Q.5` separately records one live Gemini accepted-baseline smoke result in
  `docs/phase-Q/smoke-gemini.md`

## Required Contract Samples

Required Gemini smoke coverage:

- `SessionStart`
- `BeforeAgent`
- one retained tool or lifecycle surface already approved in the normalized
  Gemini path
- one logging/observability proof on the accepted baseline

## Acceptance Criteria

- `just smoke` includes the Gemini path explicitly
- Gemini smoke proves install, dispatch, lifecycle, and logging behavior
- the CI-owned offline smoke path for Gemini is explicit and does not depend
  on a live Gemini CLI being present on generic CI runners

## Out Of Scope

- Claude smoke
- Codex smoke
- new Gemini runtime-surface expansion
- Gemini harness/doc-model expansion

## Required Validation

- `just smoke gemini live`
- `cargo test --workspace`
- `git diff --check`

## Sprint QA Checklist

- Which requirement IDs or gap IDs changed status?
- What smoke coverage was added and what was intentionally not added?
- Which files or docs are the owned write scope for the sprint?
- What validation proves the Gemini live runtime path end to end?
- What follow-on work remains separate from smoke coverage?
- Does `docs/traceability.md` now name `Q.5` as the single owner that closes
  the `HKR-019` smoke-surface row?

## Sprint QA Checklist Answers

- `HKR-019` gained its final provider-specific smoke closure here, and
  `docs/traceability.md` now names `Q.5` as the single owning sprint that
  closes the last Phase Q smoke row. No Cursor or opencode requirement IDs
  changed in this sprint.
- Added Gemini smoke coverage for install validation, `SessionStart`,
  `BeforeAgent`, the retained `PreToolUse(Agent)` observability proof, and the
  accepted-baseline `SessionEnd` state closure. New Gemini runtime-surface
  expansion and harness/doc-model work remain out of scope.
- Owned write scope:
  - `.just/smoke/gemini.py`
  - `.just/smoke/fixtures/gemini/`
  - `docs/phase-Q/smoke-gemini.md`
  - `docs/phase-Q/sprint-Q5.md`
  - `docs/requirements.md`
  - `docs/traceability.md`
- End-to-end proof comes from `just smoke gemini live` plus
  `cargo test --workspace`, with the live run verifying installed hook wiring,
  the retained runtime dispatch path, the terminal Gemini state record, and
  `dispatch.complete` observability for `SessionStart`, `PreToolUse(Agent)`,
  and `SessionEnd`.
- Remaining follow-on work stays separate from smoke coverage: earlier Claude
  and Codex smoke sprints, any future Gemini runtime-surface expansion, and
  Gemini harness/doc-model work owned outside Phase Q smoke.
- Yes. `docs/traceability.md` now records `Q.5` as the single owner that
  closes the final `HKR-019` smoke-surface row.
