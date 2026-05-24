---
id: O.4
title: Codex Runtime Parity
status: planned
branch: feature/pO-s4-codex-runtime-parity
worktree: ../schook-worktrees/feature/pO-s4-codex-runtime-parity
target: integrate/phase-O
---

# Sprint O.4 — Codex Runtime Parity

## Goal

- make approved Codex surfaces run through the same runtime/plugin path as
  Claude
- prove Codex parity for session-state, gate, and ATM-extension behavior

## Hard Dependencies

- `O.3` complete
- current `origin/integrate/phase-O` branch head
- local Codex hooks remain available on this machine

## Exact Targets

- `crates/sc-hooks-cli/src/`
- `plugins/agent-session-foundation/`
- `plugins/agent-spawn-gates/`
- `plugins/tool-output-gates/`
- `plugins/atm-extension/`
- `test-harness/hooks/codex/`

## Deliverables

- Codex runtime support for `SessionStart`
- Codex runtime support for `PreToolUse`
- Codex end-to-end runtime tests on the generic plugin path

## Acceptance Criteria

- Codex `SessionStart` drives canonical session-state updates through the
  generic runtime path
- Codex `PreToolUse` drives the existing gate and ATM extension logic through
  the generic runtime path
- Codex runtime tests pass against the approved fixture baseline

## Out Of Scope

- Codex `notify`, `Stop`, `resume`, `fork`
- Gemini runtime work
- local deployment and cutover

## Required Validation

- `cargo test --workspace`
- `pytest test-harness/hooks/codex/tests/ -q`
- `just test hooks codex`
- `git diff --check`
