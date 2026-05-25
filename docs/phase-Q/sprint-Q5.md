---
id: Q.5
title: Gemini Smoke Tests
status: planned
branch: feature/pQ-s5-gemini-smoke
worktree: ../schook-worktrees/feature/pQ-s5-gemini-smoke
target: integrate/phase-Q
---

# Sprint Q.5 — Gemini Smoke Tests

## Purpose

- implement executable smoke coverage for the Gemini runtime path through the
  new `just smoke` surface

## Entry Criteria

- accepted `Q.2` smoke infrastructure

## Exact Targets

- Gemini smoke scripts and docs under the chosen smoke ownership path

## Deliverables

- end-to-end Gemini smoke scenarios covering retained live Gemini behavior
- smoke result record for Gemini on the accepted baseline

## Acceptance Criteria

- `just smoke` includes the Gemini path explicitly
- Gemini smoke proves install, dispatch, lifecycle, and logging behavior

## Required Validation

- `cargo test --workspace`
- `git diff --check`
