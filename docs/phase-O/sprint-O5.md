---
id: O.5
title: Gemini Runtime Parity
status: complete
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
- `crates/sc-hooks-core/src/`
- `crates/sc-hooks-cli/tests/`
- `.sc-hooks/plugins/`
- `docs/requirements.md`
- `docs/traceability.md`
- `test-harness/hooks/gemini/`

## Deliverables

- Gemini runtime support for `SessionStart`
- Gemini runtime support for `SessionEnd`, `BeforeAgent`, `BeforeTool`, and
  `AfterTool`
- runtime plugin-path proof through `.sc-hooks/plugins/` for the Gemini
  generic gate and ATM behaviors this sprint closes
- Gemini end-to-end runtime tests on the generic plugin path
- Gemini-specific requirement/traceability updates for the runtime behavior
  closed in this sprint

## Acceptance Criteria

- Gemini `SessionStart` drives canonical session-state creation and root/current
  directory updates through the generic runtime path
- Gemini `SessionEnd` drives the canonical terminal-session path and session
  cleanup behavior through the generic runtime path
- Gemini `BeforeAgent` drives the generic agent-spawn gate path, including any
  approved ATM-extension metadata enrichment already proved for Claude:
  canonical session/team identity inheritance. The isolated BeforeAgent ATM
  enrichment proof is covered by the cross-provider end-to-end validation
  carried forward into `O.6`.
- Gemini `BeforeTool` drives the generic tool-gate path through the same
  provider-agnostic plugin stack
- Gemini `AfterTool` drives the generic post-tool path, including any approved
  tool-output gate behavior already proved for Claude
- when a Gemini retryable normalization or gate failure occurs, the surfaced
  operator-visible stderr message or observability event includes the
  `recovery_hint` string from `RetryableGateInput`
- Gemini runtime tests pass against the approved fixture baseline
- provider-local Gemini fields stay outside the canonical runtime contract
- `docs/requirements.md` and `docs/traceability.md` update the Gemini portion
  of `HKR-006` and `HKR-010` for the behavior actually closed in `O.5`

## Out Of Scope

- Gemini `AfterAgent`
- Codex deferred surfaces
- local deployment and cutover

## Required Validation

- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `pytest test-harness/hooks/gemini/tests/ -q`
- `just test hooks gemini`
- `git diff --check`
