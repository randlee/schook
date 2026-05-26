---
id: Q.4
title: Codex Smoke Tests
status: planned
branch: feature/pQ-s4-codex-smoke
worktree: ../schook-worktrees/feature/pQ-s4-codex-smoke
target: integrate/phase-Q
---

# Sprint Q.4 — Codex Smoke Tests

## Goal

- implement executable smoke coverage for the Codex runtime path through the
  new `just smoke` surface

## Hard Dependencies

- accepted `Q.2` smoke infrastructure

## Entry Criteria

- accepted `Q.2` smoke infrastructure

## Exact Targets

- `.just/smoke/codex.py`
- `.just/smoke/fixtures/codex/`
- `docs/phase-Q/smoke-codex.md`

## Deliverables

- end-to-end Codex smoke scenarios covering retained live Codex behavior
- smoke result record for Codex on the accepted baseline

## Required Work

- add the Codex-specific smoke module under `.just/smoke/`
- add the offline replay or dry-run assets required by the Q.2-owned CI smoke
  model under `.just/smoke/fixtures/codex/`
- record the accepted-baseline Codex smoke result in
  `docs/phase-Q/smoke-codex.md`

## CI Execution Model

- `Q.4` does not change the Q.2 smoke surface contract
- generic CI continues to run `just smoke all ci` without a live Codex CLI
- `Q.4` adds the Codex replay or dry-run assets consumed by that CI-owned
  offline gate
- `Q.4` separately records one live Codex accepted-baseline smoke result in
  `docs/phase-Q/smoke-codex.md`

## Required Contract Samples

Required Codex smoke coverage:

- `SessionStart`
- `PreToolUse`
- retained live idle/notify behavior through the supported runtime path
- one logging/observability proof on the accepted baseline

## Acceptance Criteria

- `just smoke` includes the Codex path explicitly
- Codex smoke proves install, dispatch, lifecycle, and logging behavior
- the CI-owned offline smoke path for Codex is explicit and does not depend
  on a live Codex CLI being present on generic CI runners

## Out Of Scope

- Claude smoke
- Gemini smoke
- new Codex runtime-surface expansion
- Codex harness/doc-model expansion

## Required Validation

- `just smoke codex live`
- `cargo test --workspace`
- `git diff --check`

## Sprint QA Checklist

- Which requirement IDs or gap IDs changed status?
- What smoke coverage was added and what was intentionally not added?
- Which files or docs are the owned write scope for the sprint?
- What validation proves the Codex live runtime path end to end?
- What follow-on work remains separate from smoke coverage?
