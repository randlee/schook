# Phase P Plan

## Goal

Finish the missing provider-hook parity that Phase `O` did not cover, starting
with harness support for the deferred Codex and Gemini hook surfaces and then
bringing those surfaces through runtime normalization and parity on the shared
`sc-hooks` path.

Phase `P` also closes the remaining active implementation-gap items that were
left intentionally out of `Phase O`.

Normalization in `Phase P` is not just fixture capture. It must produce:

- the permanent harness evidence for the newly retained provider surfaces
- the Rust runtime handler implementation that makes those surfaces available
  through `sc-hooks`
- one authoritative cross-agent mapping table showing:
  - provider hook name
  - canonical hook mapping
  - canonical payload family
  - provider-local fields retained outside the canonical contract
  - exact variables available from each provider surface

Planned artifact:

- `docs/phase-P/canonical-hook-mapping.md`

That table is the parity contract for `schook`. It exists so Claude, Codex, and
Gemini can be compared directly and so provider parity gaps stay explicit.

## Baseline

- planning branch: `docs/phase-P-planning`
- integration branch: `integrate/phase-P`
- prerequisite baseline: `integrate/phase-O`
- prerequisite status: `Phase O` is the runtime-normalization baseline for
  Claude plus the first approved Codex/Gemini surfaces

Phase `P` exists because runtime parity with the currently used Python-hook
behavior is still incomplete after `Phase O`.

Missing provider surfaces to add first:

- Codex:
  - `notify`
  - `Stop`
  - `resume`
  - `fork` if `P.1` harness evidence confirms it is exercisable and `P.1`
    retains it as supported
- Gemini:
  - `AfterAgent`

Still out of scope for `Phase P` unless explicitly approved later:

- Cursor runtime work

## Integration Branch

Accepted `Phase P` sprint outputs merge into:

- `integrate/phase-P`

Rules:

- sprint branches do not write accepted rows directly into
  `docs/phase-P/readiness.md`
- the integration author updates `docs/phase-P/readiness.md` only after the
  accepted sprint output is merged into `integrate/phase-P`

## Phase Entry Criteria

`Phase P` execution may begin only when:

- `Phase O` remains accepted on `integrate/phase-O`
- `just test hooks claude`, `just test hooks codex`, and
  `just test hooks gemini` remain green on the baseline
- the missing-hook expansion is treated as required production-parity work,
  not optional follow-on scope
- the existing `ProviderHookNormalizer` seam remains the only allowed path into
  generic runtime dispatch for provider-adapted surfaces

## Pre-Sprint Kickoff Checklist

Before any `Phase P` sprint starts, the handoff or working notes must record:

- exact requirement IDs and implementation-gap IDs in scope
- the `integrate/phase-O` baseline commit being used for the sprint
- proof that `just test hooks claude`, `just test hooks codex`, and
  `just test hooks gemini` were green on that baseline before new work started
- the single owning implementation path for each behavior in scope, including
  confirmation that no provider-specific bypass around `ProviderHookNormalizer`
  is being introduced
- the tests expected to fail before the sprint and pass after it
- the docs that must change in the same PR as the code
- the files, crates, and docs that define the sprint write scope

## Sprint Sequence

### P.1 Codex Missing-Hook Harness Expansion

Purpose:

- add the missing Codex hook surfaces to the permanent harness
- capture or explicitly classify the real support posture for `notify`,
  `Stop`, `resume`, and `fork`
- update the Codex hook API doc, fixtures, models, and tests to match the
  actual supported Codex hook surface
- close the Codex portion of the shared harness-contract expansion under
  `HKR-017`, including matching `docs/traceability.md` updates

Execution branch:
- `feature/pP-s1-codex-missing-hook-harness`

Execution worktree:
- `../schook-worktrees/feature/pP-s1-codex-missing-hook-harness`

### P.2 Gemini Missing-Hook Harness Expansion

Purpose:

- add the missing Gemini `AfterAgent` surface to the permanent harness as a
  first-class verified surface
- update the Gemini hook API doc, fixtures, models, and tests to match the
  actual supported Gemini hook surface
- close the Gemini portion of the shared harness-contract expansion under
  `HKR-017`, including matching `docs/traceability.md` updates

Execution branch:
- `feature/pP-s2-gemini-missing-hook-harness`

Execution worktree:
- `../schook-worktrees/feature/pP-s2-gemini-missing-hook-harness`

### P.3 `sc-lint` Suite Adoption And Cross-Platform Gate

Purpose:

- adopt the full in-repo `sc-lint` lint pattern that is already used in
  `../sc-lint` and follows the curated `just lint` style used in `../atm-core`
- wire the available `sc-lint` lint surfaces into this repo, including
  `sc-boundary` and `sc-portability`
- make cross-platform portability a hard gate before new runtime parity work
  lands so Phase P does not drift into Unix-only implementation

Execution branch:
- `feature/pP-s3-sc-lint-suite-adoption`

Execution worktree:
- `../schook-worktrees/feature/pP-s3-sc-lint-suite-adoption`

### P.4 Lifecycle-Family Normalization Extension

Purpose:

- extend the runtime-normalization contract to cover the newly approved
  post-response / turn-complete lifecycle family
- define the canonical treatment for Claude `Stop`, Codex `notify` /
  `Stop` / `resume`, and Gemini `AfterAgent` where semantics truly align
- lock the new compatibility rules before provider runtime work starts
- produce the authoritative cross-agent mapping table for the retained
  lifecycle surfaces so parity is checked against one shared artifact
- land the same-PR architecture authorization required by `ADR-SHK-009` before
  any retained lifecycle surface is treated as part of the canonical typed
  inventory

Execution precondition:

- `P.4` starts only after `P.1`, `P.2`, and `P.3` are all accepted, so the
  canonical mapping rows are derived from accepted harness evidence plus the
  accepted portability/boundary gate rather than speculative provider fields

Execution branch:
- `feature/pP-s4-canonical-hook-mapping`

Execution worktree:
- `../schook-worktrees/feature/pP-s4-canonical-hook-mapping`

### P.5 Codex Missing-Hook Runtime Parity

Purpose:

- make the newly added Codex surfaces run through the shared runtime/plugin
  path
- close the remaining Codex parity gap with the live Python-hook behavior used
  on this machine
- eliminate any Codex-specific dispatch path that bypasses the sealed
  normalization trait seam

Execution branch:
- `feature/pP-s5-codex-missing-hook-runtime`

Execution worktree:
- `../schook-worktrees/feature/pP-s5-codex-missing-hook-runtime`

### P.6 Gemini Missing-Hook Runtime Parity

Purpose:

- make Gemini `AfterAgent` run through the shared runtime/plugin path
- close the remaining Gemini lifecycle parity gap with the live Python-hook
  behavior used on this machine
- eliminate any Gemini-specific dispatch path that bypasses the sealed
  normalization trait seam

Execution branch:
- `feature/pP-s6-gemini-missing-hook-runtime`

Execution worktree:
- `../schook-worktrees/feature/pP-s6-gemini-missing-hook-runtime`

### P.7 Error, Boundary, And Portability Ruling Closeout

Purpose:

- resolve the remaining active error-surface, portability, and boundary
  rulings left open in `docs/implementation-gaps.md`
- close the explicit post-`Phase O` design decisions while the new hook
  surfaces are fresh in hand

Execution branch:
- `feature/pP-s7-error-and-boundary-rulings`

Execution worktree:
- `../schook-worktrees/feature/pP-s7-error-and-boundary-rulings`

### P.8 CLI Alias And Retry-Coverage Closeout

Purpose:

- close the remaining packaging/CLI/testing cleanup items that were kept out of
  the runtime-parity path
- finish the active implementation-gap ledger for the current release track

Execution branch:
- `feature/pP-s8-cli-and-retry-closeout`

Execution worktree:
- `../schook-worktrees/feature/pP-s8-cli-and-retry-closeout`

## Sprint Artifact Summary

- `P.1`:
  - `test-harness/hooks/codex/`
  - `test_harness/hooks/codex/`
  - `docs/hook-api/codex-hook-api.md`
  - Codex findings/checklist docs as needed
- `P.2`:
  - `test-harness/hooks/gemini/`
  - `test_harness/hooks/gemini/`
  - `docs/hook-api/gemini-hook-api.md`
- `P.3`:
  - `justfile`
  - `.just/`
  - `docs/sc-lint-boundary.md`
  - `docs/cross-platform-guidelines.md`
  - repo-local wrappers for the available `sc-lint` surfaces from `../sc-lint`
  - `fmt`, `clippy`, `modules`, `deny`, `shear`, `version`, `manifests`,
    `spell`, `pytests`, `sc-boundary`, and `sc-portability` under the
    `../atm-core` `just lint` pattern
- `P.4`:
  - `crates/sc-hooks-core/`
  - `crates/sc-hooks-cli/`
  - `docs/architecture.md`
  - `docs/requirements.md`
  - `docs/traceability.md`
  - `boundaries/`
  - `docs/phase-P/canonical-hook-mapping.md`
- `P.5`:
  - Codex runtime path updates in `crates/` and `plugins/`
  - Codex runtime parity tests
- `P.6`:
  - Gemini runtime path updates in `crates/` and `plugins/`
  - Gemini runtime parity tests
- `P.7`:
  - `docs/implementation-gaps.md`
  - core/sdk/cli boundary, portability, and error surfaces
- `P.8`:
  - install/packaging alias work for `hooks`
  - retry-path coverage in `sc-hooks-core` / `sc-hooks-test`
  - release/operator docs
