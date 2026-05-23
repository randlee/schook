---
id: N.7
title: Gemini Harness Verification
status: planned
branch: feature/pN-s7-gemini-harness-verify
worktree: ../schook-worktrees/feature/pN-s7-gemini-harness-verify
target: integrate/phase-N
---

# Sprint N.7 — Gemini Harness Verification

## Goal

- leave the Gemini harness deliverables current against local Gemini evidence
- keep Gemini externally identical to the Claude harness contract

## Hard Dependencies

- `N.5` complete
- current `origin/integrate/phase-N` branch head
- local Gemini execution path remains available on this machine

## Exact Targets

- `test-harness/hooks/gemini/`
- `test_harness/hooks/gemini/`
- shared harness files only where Gemini must align outward behavior to Claude

## Deliverables

- `test-harness/hooks/gemini/captures/raw/`
- `test-harness/hooks/gemini/fixtures/approved/`
- `test-harness/hooks/gemini/tests/`

## Acceptance Criteria

- `test-harness/hooks/gemini/captures/raw/` is the current raw-capture path
  for Gemini evidence
- `test-harness/hooks/gemini/fixtures/approved/` is the approved Gemini
  fixture set used by the tests
- `test-harness/hooks/gemini/tests/` validates the approved Gemini fixtures
- Gemini keeps the same external harness contract Claude exposes

## Out Of Scope

- Codex API doc and models
- Gemini API doc and models
- `just` integration

## Required Validation

- `pytest test-harness/hooks/gemini/tests/ -q`
- `pytest test-harness/hooks/ -q`
- `git diff --check`
