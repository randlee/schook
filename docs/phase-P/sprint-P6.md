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
- Gemini handler implementation available through the shared Rust runtime path,
  not through a side-channel provider-specific path

## Acceptance Criteria

- `AfterAgent` runs through the shared runtime path
- the resulting behavior matches the production Gemini hook model this repo is
  replacing
- Gemini parity is no longer blocked on `AfterAgent`
- `plugins/agent-session-foundation/` is updated as needed for Gemini
  `AfterAgent` and its changed behavior is covered by runtime tests
- `plugins/atm-extension/` is updated as needed for Gemini `AfterAgent` and
  its changed behavior is covered by runtime tests
- any supported-platform limit is documented explicitly instead of being hidden
  behind an implicit Unix-only implementation
- there is no direct Gemini-specific runtime bypass around
  `ProviderHookNormalizer`; if one existed on branch entry, it is removed

## Out Of Scope

- Codex runtime parity
- packaging/CLI alias work

## Required Validation

- `cargo test --workspace`
- `just lint sc-portability`
- `just test hooks gemini`
- `git diff --check`
