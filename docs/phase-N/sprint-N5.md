---
id: N.5
title: Claude Harness And Baseline Verification
status: planned
branch: feature/pN-s5-claude-harness-verify
worktree: ../schook-worktrees/feature/pN-s5-claude-harness-verify
target: integrate/phase-N
---

# Sprint N.5 — Claude Harness And Baseline Verification

## Goal

- verify the existing Claude harness, API doc, models, fixtures, and tests
  against current local/global Claude behavior
- refresh only what has drifted
- confirm the shared external harness contract that Codex and Gemini must match

## Hard Dependencies

- current `origin/integrate/phase-N` branch head
- local Claude hooks remain available on this machine

## Exact Targets

- `docs/hook-api/claude-hook-api.md`
- `test-harness/hooks/claude/`
- `test_harness/hooks/claude/`
- `test-harness/hooks/README.md`

## Deliverables

- verified Claude API doc
- verified Claude models
- verified Claude approved fixtures and tests
- written shared-harness baseline for the later provider sprints

## Acceptance Criteria

- Claude doc, models, fixtures, and tests are confirmed current or updated
- the repo has one explicit Claude-derived external harness contract for later
  provider sprints
- no Claude work is left ambiguous between "already complete" and "needs
  refresh"

## Out Of Scope

- Codex deliverables
- Gemini deliverables
- `just` integration

## Required Validation

- `pytest test-harness/hooks/claude/tests/ -q`
- `pytest test-harness/hooks/ -q`
- `git diff --check`
