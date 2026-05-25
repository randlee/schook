---
id: Q.3
title: Claude Smoke Tests
status: planned
branch: feature/pQ-s3-claude-smoke
worktree: ../schook-worktrees/feature/pQ-s3-claude-smoke
target: integrate/phase-Q
---

# Sprint Q.3 — Claude Smoke Tests

## Goal

- implement executable smoke coverage for the Claude runtime path through the
  new `just smoke` surface

## Hard Dependencies

- accepted `Q.2` smoke infrastructure

## Entry Criteria

- accepted `Q.2` smoke infrastructure

## Exact Targets

- `.just/smoke/claude.py`
- `.just/smoke/fixtures/claude/`
- `docs/phase-Q/smoke-claude.md`

## Deliverables

- end-to-end Claude smoke scenarios covering the live hook dispatch path
- smoke result record for Claude on the accepted baseline

## Required Work

- add the Claude-specific smoke module under `.just/smoke/`
- add the offline replay or dry-run assets required by the Q.2-owned CI smoke
  model under `.just/smoke/fixtures/claude/`
- record the accepted-baseline Claude smoke result in
  `docs/phase-Q/smoke-claude.md`

## CI Execution Model

- `Q.3` does not change the Q.2 smoke surface contract
- generic CI continues to run `just smoke all ci` without a live Claude CLI
- `Q.3` adds the Claude replay or dry-run assets consumed by that CI-owned
  offline gate
- `Q.3` separately records one live Claude accepted-baseline smoke result in
  `docs/phase-Q/smoke-claude.md`

## Required Contract Samples

Required Claude smoke coverage:

- `SessionStart`
- one executable tool path through the shared plugin chain
- one logging/observability proof on the accepted baseline

## Acceptance Criteria

- `just smoke` includes the Claude path explicitly
- Claude smoke can prove install, dispatch, plugin-chain, and logging behavior
- the CI-owned offline smoke path for Claude is explicit and does not depend
  on a live Claude CLI being present on generic CI runners

## Out Of Scope

- Codex smoke
- Gemini smoke
- new Claude runtime-surface expansion
- schema-harness changes

## Required Validation

- `just smoke claude live`
- `cargo test --workspace`
- `git diff --check`

## Sprint QA Checklist

- Which requirement IDs or gap IDs changed status?
- What smoke coverage was added and what was intentionally not added?
- Which files or docs are the owned write scope for the sprint?
- What validation proves the Claude live runtime path end to end?
- What follow-on work is still separate for Codex and Gemini?
