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

- verify the existing Gemini harness against current local Gemini behavior
- tighten only what is drifted, weak, or externally inconsistent with the
  Claude baseline
- keep Gemini externally identical to Claude from the harness user's
  perspective

## Hard Dependencies

- `N.5` complete
- current `origin/integrate/phase-N` branch head
- local Gemini execution path remains available on this machine

## Exact Targets

- `test-harness/hooks/gemini/`
- `test_harness/hooks/gemini/`
- shared harness files only where Gemini must align outward behavior to Claude

## Deliverables

- verified Gemini harness capture path
- verified Gemini approved fixtures
- verified Gemini fixture-validation tests

## Acceptance Criteria

- Gemini harness is confirmed runnable
- Gemini approved fixtures are confirmed against local evidence
- Gemini tests validate schema-bearing fixtures
- Gemini presents the same external harness contract Claude does

## Out Of Scope

- Codex API doc and models
- Gemini API doc and models
- `just` integration

## Required Validation

- `pytest test-harness/hooks/gemini/tests/ -q`
- `pytest test-harness/hooks/ -q`
- `git diff --check`
