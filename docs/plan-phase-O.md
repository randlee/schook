# Phase O Plan

## Goal

Implement runtime hook normalization for the approved Codex and Gemini surfaces
that Phase `N` proved, bring those providers to Claude parity on the shared
runtime plugin path, and finish with one local-machine cutover path for all
supported agents on this computer.

Phase `O` consumes the permanent provider harness as input. It does not rebuild
the harness or reopen schema-proof work except where live drift forces a narrow
fixture refresh.

## Baseline

- planning branch: `docs/phase-O-planning`
- integration branch: `integrate/phase-O`
- prerequisite baseline: `integrate/phase-N` at `89ad9cb`
- prerequisite verdict: `Phase N` closed `PARTIAL_GO`
- develop-branch Phase N planning docs still show only `N.1` through `N.4`
  rows in `docs/phase-N/readiness.md`
- prerequisite provider verification for `Phase O` execution is therefore:
  accepted `N.5` through `N.10` outputs present on the `integrate/phase-N`
  execution baseline, not merely present in the develop-branch planning copy

Approved `Phase N` runtime surfaces:

- Codex:
  - `SessionStart`
  - `PreToolUse`
- Gemini:
  - `SessionStart`
  - `SessionEnd`
  - `BeforeAgent`
  - `BeforeTool`
  - `AfterTool`

Deferred `Phase N` surfaces that remain out of scope for `Phase O`:

- Codex:
  - `notify`
  - `Stop`
  - `resume`
  - `fork`
- Gemini:
  - `AfterAgent`
- Cursor:
  - all runtime work remains deferred

## Integration Branch

Accepted `Phase O` sprint outputs merge into:

- `integrate/phase-O`

Rules:

- sprint branches do not write accepted rows directly into
  `docs/phase-O/readiness.md`
- the integration author updates `docs/phase-O/readiness.md` only after the
  accepted sprint output is merged into `integrate/phase-O`
- any sprint prerequisite that requires prior accepted outputs means accepted
  and merged to `integrate/phase-O`, not merely complete on a feature branch

## Phase Entry Criteria

`Phase O` execution may begin only when:

- `Phase N` remains accepted on `integrate/phase-N`
- provider harness verification remains green for Claude, Codex, and Gemini
- runtime work is limited to the approved `Phase N` surfaces listed above
- `CDR-B` is merged to the execution baseline before `O.3` begins

## Sprint Sequence

### O.1 Codebase Hygiene And Harness Layout Parity

Purpose:

- close pre-existing Rust best-practices and harness-layout issues that block a
  clean runtime-normalization baseline
- carry forward the accepted `SEAL-001` closure decision explicitly before
  runtime-normalization work begins

Execution branch:
- `feature/pO-s1-codebase-hygiene`

Execution worktree:
- `../schook-worktrees/feature/pO-s1-codebase-hygiene`

### O.2 `sc-lint` Setup And Boundary Enforcement

Purpose:

- adopt `sc-lint-boundary` in this repo through `just` wrappers using the same
  top-level help/lint/ci pattern used by `../atm-core`
- add boundary-enforcement plumbing before normalization code starts

Execution branch:
- `feature/pO-s2-sc-lint-setup`

Execution worktree:
- `../schook-worktrees/feature/pO-s2-sc-lint-setup`

### O.3 Runtime Normalization Foundation

Purpose:

- implement the provider-to-canonical normalization layer for approved Codex
  and Gemini surfaces only
- require one sealed normalization trait boundary so provider-specific parsing
  cannot bypass the canonical runtime path
- prove normalization against the approved provider fixtures before broader
  parity work begins

Execution branch:
- `feature/pO-s3-runtime-normalization-foundation`

Execution worktree:
- `../schook-worktrees/feature/pO-s3-runtime-normalization-foundation`

### O.4 Codex Runtime Parity

Purpose:

- make approved Codex surfaces run through the same runtime/plugin path as
  Claude
- prove session-state, gate, and ATM-extension behavior on Codex

Execution branch:
- `feature/pO-s4-codex-runtime-parity`

Execution worktree:
- `../schook-worktrees/feature/pO-s4-codex-runtime-parity`

### O.5 Gemini Runtime Parity

Purpose:

- make approved Gemini surfaces run through the same runtime/plugin path as
  Claude
- prove session-state, gate, and ATM-extension behavior on Gemini

Execution branch:
- `feature/pO-s5-gemini-runtime-parity`

Execution worktree:
- `../schook-worktrees/feature/pO-s5-gemini-runtime-parity`

### O.6 Cross-Provider Plugin Parity And E2E Validation

Purpose:

- prove that Claude, Codex, and Gemini all drive the same generic plugin stack
  on their approved surfaces
- freeze the runtime normalization boundary with end-to-end validation and
  observability proof

Execution branch:
- `feature/pO-s6-cross-provider-plugin-parity`

Execution worktree:
- `../schook-worktrees/feature/pO-s6-cross-provider-plugin-parity`

### O.7 Local Deployment And Cutover

Purpose:

- install and exercise the normalized `sc-hooks` runtime for all supported
  agents on this computer
- finish with a documented rollback-safe local cutover path

Execution branch:
- `feature/pO-s7-local-cutover`

Execution worktree:
- `../schook-worktrees/feature/pO-s7-local-cutover`

## Sprint Artifact Summary

- `O.1`:
  - `docs/implementation-gaps.md`
  - `pyproject.toml`
  - `test-harness/hooks/gemini/tests/__init__.py`
  - hygiene fixes in `crates/sc-hooks-core/`, `crates/sc-hooks-cli/`, and
    `crates/sc-hooks-sdk/`
- `O.2`:
  - repo-local `just` wrapper integration in `justfile` and `.just/`
  - `docs/sc-lint-boundary.md`
  - `boundaries/` records for the normalization boundary
  - `sc-lint-boundary` attributes/dependencies needed for boundary enforcement
- `O.3`:
  - runtime normalization code in `crates/sc-hooks-core/` and
    `crates/sc-hooks-cli/`
  - one sealed normalization trait boundary plus lint-enforced no-bypass rules
  - provider normalization tests backed by approved fixtures
- `O.4`:
  - Codex runtime adapter path
  - Codex end-to-end runtime tests
- `O.5`:
  - Gemini runtime adapter path
  - Gemini end-to-end runtime tests
- `O.6`:
  - cross-provider parity tests for Claude, Codex, and Gemini
  - runtime observability proof on approved surfaces
- `O.7`:
  - local install/cutover helpers and docs
  - machine-local smoke-test record for the supported providers

The sprint docs remain the only authoritative source for per-sprint
deliverables, acceptance criteria, and closure rules.

## Phase Rules

- `Phase O` is runtime-normalization work, not new schema-discovery work
- approved fixtures, provider hook API docs, and provider models remain the
  source of truth for runtime behavior
- `sc-lint-boundary` enforcement must be installed before normalization begins
- the normalization boundary must be one sealed trait surface with
  lint-detected no-bypass enforcement
- `NormalizedHookContext` feeds the existing `HookContext` construction path;
  `Phase O` does not run a second parallel runtime dispatch path
- deferred `Phase N` surfaces remain out of scope unless a later explicit phase
  reopens them
- Cursor remains out of scope for `Phase O`; `HKR-007` stays deferred
- `HKR-006` closes in `Phase O` only for approved Codex and Gemini runtime
  surfaces; it does not reopen Cursor
- `HKR-010` closes in `Phase O` only when spawn/tool/ATM behavior works through
  the normalized runtime path with exact retryable failures preserved
- `O.1` closes the pre-existing hygiene set before runtime normalization:
  `PN-008`, `RBP-1`, `RBP-2`, `RBP-4`, and the carried-forward `SEAL-001`
  closure decision
- provider-specific parsing belongs behind the normalization trait; generic
  plugin logic remains provider-agnostic
- no provider-local field may be promoted into the canonical runtime contract
  without new approved fixture evidence
- `O.7` may cut over local agents only after `O.6` is accepted on
  `integrate/phase-O`

## Initial Planning Outputs

- `docs/plan-phase-O.md`
- `docs/phase-O/readiness.md`
- `docs/phase-O/sprint-O1.md`
- `docs/phase-O/sprint-O2.md`
- `docs/phase-O/sprint-O3.md`
- `docs/phase-O/sprint-O4.md`
- `docs/phase-O/sprint-O5.md`
- `docs/phase-O/sprint-O6.md`
- `docs/phase-O/sprint-O7.md`
