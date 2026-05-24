---
id: P.5
title: Codex Missing-Hook Runtime Parity
status: planned
branch: feature/pP-s5-codex-missing-hook-runtime
worktree: ../schook-worktrees/feature/pP-s5-codex-missing-hook-runtime
target: integrate/phase-P
---

# Sprint P.5 — Codex Missing-Hook Runtime Parity

## Goal

- close the remaining Codex runtime parity gap with the live Python-hook
  behavior used on this machine

## Hard Dependencies

- accepted `P.4`

## Exact Targets

- `crates/sc-hooks-core/`
- `crates/sc-hooks-cli/`
- `plugins/agent-session-foundation/`
- `plugins/atm-extension/`
- provider runtime tests
- `docs/requirements.md`
- `docs/traceability.md`

## Deliverables

- Codex runtime support for the retained missing lifecycle surfaces
- end-to-end Codex runtime tests for those surfaces
- requirements/traceability updates closing the Codex parity delta

## Acceptance Criteria

- the retained Codex lifecycle surfaces run through the shared runtime path
- the resulting behavior matches the production Codex hook model this repo is
  replacing
- Codex parity is no longer blocked on `notify`, `Stop`, or `resume`
- any supported-platform limit is documented explicitly instead of being hidden
  behind an implicit Unix-only implementation

## Out Of Scope

- Gemini runtime parity
- packaging/CLI alias work

## Required Validation

- `cargo test --workspace`
- `just test hooks codex`
- `git diff --check`
