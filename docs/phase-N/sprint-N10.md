---
id: N.10
title: Harness `just` Integration
status: planned
branch: feature/pN-s10-harness-just-integration
worktree: ../schook-worktrees/feature/pN-s10-harness-just-integration
target: integrate/phase-N
---

# Sprint N.10 — Harness `just` Integration

## Goal

- expose one stable `just` entrypoint per provider for the permanent harness
- keep harness execution easy and repeatable on this repo

## Hard Dependencies

- `N.8` complete
- `N.9` complete
- existing local skill/template support for wiring `just`

## Exact Targets

- `justfile`
- any local support files needed for the new `just` targets
- harness docs only where the `just` entrypoints must be documented

## Deliverables

- `just test hooks claude`
- `just test hooks codex`
- `just test hooks gemini`
- docs explaining what each target runs

## Acceptance Criteria

- `just test hooks claude` exercises the Claude harness
- `just test hooks codex` exercises the Codex harness
- `just test hooks gemini` exercises the Gemini harness
- the targets are documented and reproducible

## Out Of Scope

- runtime hook implementation

## Required Validation

- `just test hooks claude`
- `just test hooks codex`
- `just test hooks gemini`
- `git diff --check`
