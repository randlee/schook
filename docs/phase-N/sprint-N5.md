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

- current `docs/hook-api/claude-hook-api.md`
- current Claude Pydantic models
- current Claude approved fixtures/tests
- one written shared-harness baseline for later provider sprints

## Acceptance Criteria

- the four listed deliverables are each either:
  - confirmed current from local evidence
  - or updated to match local evidence
- the later provider sprints can cite one explicit Claude-derived external
  harness contract

## Out Of Scope

- Codex deliverables
- Gemini deliverables
- `just` integration

## Required Validation

- `pytest test-harness/hooks/claude/tests/ -q`
- `pytest test-harness/hooks/ -q`
- `git diff --check`
