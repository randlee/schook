---
id: O.2
title: Runtime Normalization Foundation
status: planned
branch: feature/pO-s2-runtime-normalization-foundation
worktree: ../schook-worktrees/feature/pO-s2-runtime-normalization-foundation
target: integrate/phase-O
---

# Sprint O.2 — Runtime Normalization Foundation

## Goal

- implement the provider-to-canonical runtime normalization layer for approved
  Codex and Gemini surfaces only
- keep provider-specific parsing isolated from the generic runtime/plugin path

## Hard Dependencies

- `O.1` complete
- current `origin/integrate/phase-N` accepted baseline
- approved provider fixtures, models, and hook API docs from `Phase N`
- `CDR-B` merged or otherwise present on the execution baseline

## Exact Targets

- `crates/sc-hooks-core/src/`
- `crates/sc-hooks-cli/src/`
- `crates/sc-hooks-cli/tests/`
- `test-harness/hooks/codex/`
- `test-harness/hooks/gemini/`

## Deliverables

- runtime normalization code for approved Codex and Gemini surfaces
- fixture-backed normalization tests
- one documented normalization boundary for provider-specific vs canonical data

## Acceptance Criteria

- approved Codex fixtures normalize into canonical runtime hook/event data
- approved Gemini fixtures normalize into canonical runtime hook/event data
- deferred `Phase N` surfaces are not implemented or implied as supported
- normalization tests cite approved fixtures and pass

## Out Of Scope

- Codex `notify`, `Stop`, `resume`, `fork`
- Gemini `AfterAgent`
- local deployment and cutover

## Required Validation

- `cargo test --workspace`
- `pytest test-harness/hooks/codex/tests/ -q`
- `pytest test-harness/hooks/gemini/tests/ -q`
- `git diff --check`
