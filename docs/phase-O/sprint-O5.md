---
id: O.5
title: Gemini Runtime Parity
status: planned
branch: feature/pO-s5-gemini-runtime-parity
worktree: ../schook-worktrees/feature/pO-s5-gemini-runtime-parity
target: integrate/phase-O
---

# Sprint O.5 — Gemini Runtime Parity

## Goal

- make approved Gemini surfaces run through the same runtime/plugin path as
  Claude
- prove Gemini parity for session-state, gate, and ATM-extension behavior

## Hard Dependencies

- `O.3` complete
- current `origin/integrate/phase-O` branch head
- local Gemini hooks remain available on this machine

## Exact Targets

- `crates/sc-hooks-cli/src/`
- `plugins/agent-session-foundation/`
- `plugins/agent-spawn-gates/`
- `plugins/tool-output-gates/`
- `plugins/atm-extension/`
- `test-harness/hooks/gemini/`

## Deliverables

- Gemini runtime support for `SessionStart`
- Gemini runtime support for `SessionEnd`, `BeforeAgent`, `BeforeTool`, and
  `AfterTool`
- Gemini end-to-end runtime tests on the generic plugin path

## Acceptance Criteria

- Gemini approved surfaces drive canonical runtime behavior through the generic
  plugin path
- Gemini runtime tests pass against the approved fixture baseline
- provider-local Gemini fields stay outside the canonical runtime contract

## Out Of Scope

- Gemini `AfterAgent`
- Codex deferred surfaces
- local deployment and cutover

## Required Validation

- `cargo test --workspace`
- `pytest test-harness/hooks/gemini/tests/ -q`
- `just test hooks gemini`
- `git diff --check`
