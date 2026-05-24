---
id: P.4
title: Lifecycle-Family Normalization Extension
status: planned
branch: feature/pP-s4-lifecycle-normalization-extension
worktree: ../schook-worktrees/feature/pP-s4-lifecycle-normalization-extension
target: integrate/phase-P
---

# Sprint P.4 — Lifecycle-Family Normalization Extension

## Goal

- extend the canonical runtime-normalization model so the newly retained
  lifecycle surfaces can run through one shared seam instead of provider-local
  one-offs

## Hard Dependencies

- accepted `P.3`
- current `ProviderHookNormalizer` boundary from `Phase O`

## Exact Targets

- `crates/sc-hooks-core/`
- `crates/sc-hooks-cli/`
- `docs/architecture.md`
- `docs/requirements.md`
- `docs/traceability.md`
- `boundaries/`

## Deliverables

- extended canonical lifecycle type inventory for the newly retained surfaces
- updated provider-to-canonical compatibility rules
- updated `sc-lint-boundary` records if the normalization seam grows
- architecture and requirement updates that freeze the new lifecycle contract
- one authoritative cross-agent mapping table that records, for each retained
  lifecycle surface:
  - provider hook name
  - canonical hook mapping
  - canonical payload family
  - available variables
  - provider-local fields not promoted into the canonical contract
- explicit removal plan for any direct provider-specific runtime path that
  bypasses `ProviderHookNormalizer`

## Acceptance Criteria

- the newly retained Codex/Gemini lifecycle surfaces have one documented and
  tested canonical runtime path
- no provider bypass path is introduced around the existing normalization seam
- the compatibility table and boundary docs match the landed code
- the cross-agent mapping table exists and is sufficient to compare Claude,
  Codex, and Gemini parity surface-by-surface
- any existing Codex- or Gemini-specific runtime bypass path is either removed
  in this sprint or called out as a blocking defect

## Out Of Scope

- provider-specific runtime parity wiring
- packaging/CLI alias work
- broader error-surface cleanup

## Required Validation

- `cargo check --workspace`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `just lint sc-boundary`
- `git diff --check`
