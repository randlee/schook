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
- `crates/sc-hooks-core/src/`
- `crates/sc-hooks-cli/tests/`
- `.sc-hooks/plugins/`
- `docs/requirements.md`
- `docs/traceability.md`
- `test-harness/hooks/codex/`

## Deliverables

- Codex runtime support for `SessionStart`
- Codex runtime support for `PreToolUse`
- runtime plugin-path proof through `.sc-hooks/plugins/` for the Codex generic
  gate and ATM behaviors this sprint closes
- Codex end-to-end runtime tests on the generic plugin path
- Codex-specific requirement/traceability updates for the runtime behavior
  closed in this sprint

## Acceptance Criteria

- Codex `SessionStart` drives canonical session-state updates through the
  generic runtime path
- Codex `PreToolUse` drives the existing gate and ATM extension logic through
  the generic runtime path
- the required Codex ATM-extension behavior is limited to the same metadata
  enrichment already proved for Claude: canonical session/team identity
  inheritance plus retryable gate failures on blocked tool execution
- Codex runtime tests pass against the approved fixture baseline
- `docs/requirements.md` and `docs/traceability.md` update the Codex portion
  of `HKR-006` and `HKR-010` for the behavior actually closed in `O.4`

## Out Of Scope

- Codex `notify`, `Stop`, `resume`, `fork`
- Gemini runtime work
- local deployment and cutover

## Required Validation

- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `pytest test-harness/hooks/codex/tests/ -q`
- `just test hooks codex`
- `git diff --check`
