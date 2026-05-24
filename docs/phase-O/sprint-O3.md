---
id: O.3
title: Runtime Normalization Foundation
status: planned
branch: feature/pO-s3-runtime-normalization-foundation
worktree: ../schook-worktrees/feature/pO-s3-runtime-normalization-foundation
target: integrate/phase-O
---

# Sprint O.3 — Runtime Normalization Foundation

## Goal

- implement the provider-to-canonical normalization layer for approved Codex
  and Gemini surfaces only
- require one sealed normalization trait boundary so provider-specific parsing
  cannot bypass the canonical runtime path

## Hard Dependencies

- `O.2` complete
- current `origin/integrate/phase-N` accepted baseline
- approved provider fixtures, models, and hook API docs from `Phase N`
- `CDR-B` merged or otherwise present on the execution baseline
- `sc-lint` boundary enforcement active in this repo

## Exact Targets

- `crates/sc-hooks-core/src/`
- `crates/sc-hooks-cli/src/`
- `crates/sc-hooks-cli/tests/`
- `test-harness/hooks/codex/`
- `test-harness/hooks/gemini/`

## Deliverables

- runtime normalization code for approved Codex and Gemini surfaces
- one sealed normalization trait boundary
- fixture-backed normalization tests
- one documented normalization boundary for provider-specific vs canonical data

## Acceptance Criteria

- provider raw payload handling enters the runtime only through one sealed
  normalization trait boundary
- approved Codex fixtures normalize into canonical runtime hook/event data
- approved Gemini fixtures normalize into canonical runtime hook/event data
- deferred `Phase N` surfaces are not implemented or implied as supported
- normalization tests cite approved fixtures and pass
- boundary lint fails if provider-specific parsing bypasses the normalization
  trait boundary or if external impls are introduced

## Out Of Scope

- Codex `notify`, `Stop`, `resume`, `fork`
- Gemini `AfterAgent`
- local deployment and cutover

## Required Validation

- `cargo test --workspace`
- `just lint sc-boundary`
- `pytest test-harness/hooks/codex/tests/ -q`
- `pytest test-harness/hooks/gemini/tests/ -q`
- `git diff --check`
