---
id: N.6
title: Codex Harness Verification
status: planned
branch: feature/pN-s6-codex-harness-verify
worktree: ../schook-worktrees/feature/pN-s6-codex-harness-verify
target: integrate/phase-N
---

# Sprint N.6 — Codex Harness Verification

## Goal

- leave the Codex harness deliverables current against local Codex evidence
- keep Codex externally identical to the Claude harness contract

## Hard Dependencies

- current `origin/integrate/phase-N` branch head
- `N.1` complete
- `N.5` complete
- local Codex global hooks remain available on this machine

## Exact Targets

- `test-harness/hooks/codex/`
- `test_harness/hooks/codex/`
- `test-harness/hooks/README.md`

## Deliverables

- `test-harness/hooks/codex/captures/raw/`
- `test-harness/hooks/codex/fixtures/approved/`
- `test-harness/hooks/codex/tests/`

## Acceptance Criteria

- `test-harness/hooks/codex/captures/raw/` is the current raw-capture path for
  Codex evidence
- `test-harness/hooks/codex/fixtures/approved/` is the approved Codex fixture
  set used by the tests
- `test-harness/hooks/codex/tests/` validates the approved Codex fixtures
- Codex keeps the same external harness contract Claude exposes

## Out Of Scope

- Codex API doc and models
- Gemini deliverables
- `just` integration

## Required Validation

- `pytest test-harness/hooks/codex/tests/ -q`
- `pytest test-harness/hooks/ -q`
- `git diff --check`
