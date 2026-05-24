---
id: O.4
title: Cross-Provider Plugin Parity And End-To-End Validation
status: planned
branch: feature/pO-s4-cross-provider-plugin-parity
worktree: ../schook-worktrees/feature/pO-s4-cross-provider-plugin-parity
target: integrate/phase-O
---

# Sprint O.4 — Cross-Provider Plugin Parity And End-To-End Validation

## Goal

- prove that Claude, Codex, and Gemini all drive the same generic plugin stack
  on their approved surfaces
- freeze the runtime normalization boundary with end-to-end and observability
  proof

## Hard Dependencies

- `O.2` complete
- `O.3` complete
- current `origin/integrate/phase-O` branch head

## Exact Targets

- `crates/sc-hooks-cli/src/`
- `crates/sc-hooks-cli/tests/`
- `plugins/`
- `docs/requirements.md`
- `docs/architecture.md`
- `docs/traceability.md`

## Deliverables

- cross-provider parity tests for Claude, Codex, and Gemini approved surfaces
- observability proof for the approved cross-provider runtime path
- control-doc updates for the normalized runtime boundary

## Acceptance Criteria

- Claude, Codex, and Gemini approved surfaces all execute through the same
  generic plugin stack
- observability and error semantics remain stable across approved providers
- control docs describe only the runtime behavior actually proved in this phase

## Out Of Scope

- local cutover
- deferred `Phase N` surfaces
- Cursor runtime work

## Required Validation

- `cargo check --workspace`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `pytest test-harness/hooks/ -q`
- `just test hooks claude`
- `just test hooks codex`
- `just test hooks gemini`
- `git diff --check`
