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

- verify the existing Codex harness against current local Codex behavior
- tighten only what is drifted, weak, or externally inconsistent with the
  Claude baseline
- keep Codex externally identical to Claude from the harness user's
  perspective

## Hard Dependencies

- current `origin/integrate/phase-N` branch head
- `N.5` complete
- local Codex global hooks remain available on this machine

## Exact Targets

- `test-harness/hooks/codex/`
- `test_harness/hooks/codex/`
- shared harness files only where Codex must align outward behavior to Claude

## Deliverables

- verified Codex harness capture path
- verified Codex approved fixtures
- verified Codex fixture-validation tests

## Required Work

- verify current Codex capture flows and fixture inventory against local Codex
  runs
- correct only real drift, missing capture proof, or outward inconsistency
- keep internal provider differences internal; preserve Claude-compatible
  controls, layout, validation, and output shape

## This Sprint Does Not Close

- Codex API doc and models
- Gemini deliverables
- `just` integration

## Acceptance Criteria

- Codex harness is confirmed runnable
- Codex approved fixtures are confirmed against local evidence
- Codex tests validate schema-bearing fixtures
- Codex presents the same external harness contract Claude does

## Required Validation

- `pytest test-harness/hooks/codex/tests/ -q`
- `pytest test-harness/hooks/ -q`
- `git diff --check`
