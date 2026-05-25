---
id: Q.4
title: Codex Smoke Tests
status: planned
branch: feature/pQ-s4-codex-smoke
worktree: ../schook-worktrees/feature/pQ-s4-codex-smoke
target: integrate/phase-Q
---

# Sprint Q.4 — Codex Smoke Tests

## Purpose

- implement executable smoke coverage for the Codex runtime path through the
  new `just smoke` surface

## Entry Criteria

- accepted `Q.2` smoke infrastructure

## Exact Targets

- Codex smoke scripts and docs under the chosen smoke ownership path

## Deliverables

- end-to-end Codex smoke scenarios covering retained live Codex behavior
- smoke result record for Codex on the accepted baseline

## Acceptance Criteria

- `just smoke` includes the Codex path explicitly
- Codex smoke proves install, dispatch, lifecycle, and logging behavior

## Required Validation

- `cargo test --workspace`
- `git diff --check`
