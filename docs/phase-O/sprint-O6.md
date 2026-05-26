---
id: O.6
title: Cross-Provider Plugin Parity And End-To-End Validation
status: complete
branch: feature/pO-s6-cross-provider-plugin-parity
worktree: ../schook-worktrees/feature/pO-s6-cross-provider-plugin-parity
target: integrate/phase-O
---

# Sprint O.6 — Cross-Provider Plugin Parity And End-To-End Validation

## Goal

- prove that Claude, Codex, and Gemini all drive the same generic plugin stack
  on their approved surfaces
- freeze the runtime normalization boundary with end-to-end and observability
  proof

## Hard Dependencies

- `O.4` complete
- `O.5` complete
- current `origin/integrate/phase-O` branch head

## Exact Targets

- `crates/sc-hooks-cli/src/`
- `crates/sc-hooks-cli/tests/`
- `docs/requirements.md`
- `docs/architecture.md`
- `docs/traceability.md`

## Deliverables

- cross-provider parity tests for Claude, Codex, and Gemini approved surfaces
- observability proof for the approved cross-provider runtime path
- control-doc updates for the normalized runtime boundary
- final cross-provider consolidation of the `HKR-006` and `HKR-010`
  requirement/traceability rows after the per-provider updates from `O.4` and
  `O.5`

## Acceptance Criteria

- Claude, Codex, and Gemini approved surfaces all execute through the same
  generic plugin stack
- observability and error semantics remain stable across approved providers
- `docs/requirements.md` updates `HKR-006` and `HKR-010` to describe only the
  provider runtime behavior actually proved in `Phase O`, while `HKR-007`
  remains deferred
- `docs/traceability.md` updates the `HKR-006` and `HKR-010` rows to cite the
  final cross-provider runtime tests and the stable `just test hooks <provider>`
  commands
- `docs/architecture.md` updates section `1.1 Stable Product ADR IDs`,
  section `2. Current System Boundary`, and the runtime-boundary section added
  by `O.3` so they match only the behavior proved in `O.3` through `O.6`

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
