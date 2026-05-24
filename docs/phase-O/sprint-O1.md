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
- carry forward the accepted `SEAL-001` closure decision before the runtime
  adapter line starts

## Hard Dependencies

- current `origin/integrate/phase-N` accepted baseline
- accepted `integrate/phase-N` hygiene findings set:
  `PN-008`, `RBP-1`, `RBP-2`, `RBP-4`
- current `docs/implementation-gaps.md` `SEAL-001` note

## Exact Targets

- `crates/sc-hooks-core/src/session.rs`
- `crates/sc-hooks-core/src/context.rs`
- `crates/sc-hooks-cli/src/resolution.rs`
- `test-harness/hooks/gemini/tests/__init__.py`
- `pyproject.toml`
- `docs/implementation-gaps.md`

## Deliverables

- `test-harness/hooks/gemini/tests/__init__.py` plus the matching
  `pyproject.toml` package entry
- `active_pid` validation hardening in `sc-hooks-core/src/session.rs`
- `HookContext.event` lifetime fix in `sc-hooks-core/src/context.rs`
- `HandlerRejected.reason` condition-path propagation in
  `sc-hooks-cli/src/resolution.rs`
- `SEAL-001` closure carried forward in `docs/implementation-gaps.md` with an
  explicit Phase O note that the unsealed-trait decision remains in force and
  is not reopened by `O.1`

## Acceptance Criteria

- `test-harness/hooks/gemini/tests/__init__.py` exists and the Gemini test
  package is wired the same way as Codex in `pyproject.toml` (`PN-008`)
- `active_pid` no longer deserializes through `#[serde(default)]` and zero is
  rejected by record validation (`RBP-1`)
- `HookContext.event` no longer forces unnecessary `'static` allocation
  (`RBP-2`)
- condition errors in `resolution.rs` are captured into
  `HandlerRejected.reason` (`RBP-4`)
- `docs/implementation-gaps.md` states that `SEAL-001` remains closed under
  the accepted unsealed-trait decision and is not reopened in `Phase O`

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
