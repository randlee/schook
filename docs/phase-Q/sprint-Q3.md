---
id: Q.3
title: Claude Smoke Tests
status: planned
branch: feature/pQ-s3-claude-smoke
worktree: ../schook-worktrees/feature/pQ-s3-claude-smoke
target: integrate/phase-Q
---

# Sprint Q.3 — Claude Smoke Tests

## Purpose

- implement executable smoke coverage for the Claude runtime path through the
  new `just smoke` surface

## Entry Criteria

- accepted `Q.2` smoke infrastructure

## Exact Targets

- Claude smoke scripts and docs under the chosen smoke ownership path

## Deliverables

- end-to-end Claude smoke scenarios covering the live hook dispatch path
- smoke result record for Claude on the accepted baseline

## Acceptance Criteria

- `just smoke` includes the Claude path explicitly
- Claude smoke can prove install, dispatch, plugin-chain, and logging behavior

## Required Validation

- `cargo test --workspace`
- `git diff --check`
