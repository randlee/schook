---
id: O.1
title: Codebase Hygiene And Harness Layout Parity
status: planned
branch: feature/pO-s1-codebase-hygiene
worktree: ../schook-worktrees/feature/pO-s1-codebase-hygiene
target: integrate/phase-O
---

# Sprint O.1 — Codebase Hygiene And Harness Layout Parity

## Goal

- close the pre-existing hygiene and layout issues that should not bleed into
  runtime-normalization work
- close `SEAL-001` before the runtime adapter line starts

## Hard Dependencies

- current `origin/integrate/phase-N` accepted baseline
- current `docs/implementation-gaps.md` `SEAL-001` note

## Exact Targets

- `crates/sc-hooks-core/src/session.rs`
- `crates/sc-hooks-core/src/context.rs`
- `crates/sc-hooks-sdk/src/traits.rs`
- `crates/sc-hooks-cli/src/resolution.rs`
- `test_harness/hooks/gemini/tests/__init__.py`
- `pyproject.toml`
- `docs/implementation-gaps.md`

## Deliverables

- `test_harness/hooks/gemini/tests/__init__.py` plus the matching
  `pyproject.toml` package entry
- `active_pid` validation hardening in `sc-hooks-core/src/session.rs`
- `HookContext.event` lifetime fix in `sc-hooks-core/src/context.rs`
- `HandlerRejected.reason` condition-path propagation in
  `sc-hooks-cli/src/resolution.rs`
- sealed `ManifestProvider`, `SyncHandler`, and `AsyncHandler` traits plus the
  replacing `SEAL-001` closure note in `docs/implementation-gaps.md`

## Acceptance Criteria

- `test_harness/hooks/gemini/tests/__init__.py` exists and the Gemini test
  package is wired the same way as Codex in `pyproject.toml`
- `active_pid` no longer deserializes through `#[serde(default)]` and zero is
  rejected by record validation
- `HookContext.event` no longer forces unnecessary `'static` allocation
- condition errors in `resolution.rs` are captured into
  `HandlerRejected.reason`
- `ManifestProvider`, `SyncHandler`, and `AsyncHandler` are sealed and
  `docs/implementation-gaps.md` replaces the prior `SEAL-001` closure note
  with the new sealed-trait outcome

## Out Of Scope

- runtime normalization
- Codex runtime parity
- Gemini runtime parity
- local deployment and cutover

## Required Validation

- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `pytest test-harness/hooks/gemini/tests/ -q`
- `git diff --check`
