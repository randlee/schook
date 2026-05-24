---
id: P.6
title: Gemini Missing-Hook Runtime Parity
status: planned
branch: feature/pP-s6-gemini-missing-hook-runtime
worktree: ../schook-worktrees/feature/pP-s6-gemini-missing-hook-runtime
target: integrate/phase-P
---

# Sprint P.6 — Gemini Missing-Hook Runtime Parity

## Goal

- close the remaining Gemini lifecycle parity gap by carrying `AfterAgent`
  through the shared runtime/plugin path

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

- Gemini runtime support for `AfterAgent`
- end-to-end Gemini runtime tests for `AfterAgent`
- requirements/traceability updates closing the Gemini parity delta

## Acceptance Criteria

- `AfterAgent` runs through the shared runtime path
- the resulting behavior matches the production Gemini hook model this repo is
  replacing
- Gemini parity is no longer blocked on `AfterAgent`
- any supported-platform limit is documented explicitly instead of being hidden
  behind an implicit Unix-only implementation

## Out Of Scope

- Codex runtime parity
- packaging/CLI alias work

## Required Validation

- `cargo test --workspace`
- `just test hooks gemini`
- `git diff --check`
