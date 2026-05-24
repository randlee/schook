---
id: O.7
title: Local Deployment And Cutover
status: planned
branch: feature/pO-s7-local-cutover
worktree: ../schook-worktrees/feature/pO-s7-local-cutover
target: integrate/phase-O
---

# Sprint O.7 — Local Deployment And Cutover

## Goal

- deploy the normalized `sc-hooks` runtime for the supported agents on this
  computer
- finish with a rollback-safe local cutover path

## Hard Dependencies

- `O.6` complete
- current `origin/integrate/phase-O` branch head
- local access to the provider runtime config paths on this machine

## Exact Targets

- `crates/sc-hooks-cli/src/install.rs`
- `justfile`
- `README.md`
- `USAGE.md`
- local provider runtime config paths documented by the sprint

## Deliverables

- one local install/cutover path for Claude, Codex, and Gemini on this machine
- rollback instructions
- machine-local smoke-test record for the supported providers

## Acceptance Criteria

- the documented local cutover path installs the normalized runtime for Claude,
  Codex, and Gemini on this machine
- rollback steps are documented and tested once
- machine-local smoke tests prove the supported providers are using the
  normalized runtime path

## Out Of Scope

- Cursor runtime work
- deferred `Phase N` surfaces
- cross-machine fleet rollout beyond this computer

## Required Validation

- `cargo test --workspace`
- `just test hooks claude`
- `just test hooks codex`
- `just test hooks gemini`
- `git diff --check`
