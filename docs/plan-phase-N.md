# Phase N Plan

## Goal

Define and execute the first cross-provider schema-capture phase for Codex and
Gemini so `schook` can normalize their hook contracts from repo-owned evidence
instead of inferred provider behavior.

Phase `N` owns the planning and harness-prep line that should not be mixed into
the completed Claude-first implementation history:

- Codex hook harness capture and schema proof
- Gemini hook harness capture and schema proof
- provider-owned fixture/model/drift artifacts for both providers
- normalization-target inventory built from verified fields only
- go/no-go criteria for later provider runtime promotion

## Baseline

- planning branch: `feature/delay-idle-hook-testing`
- integration branch: `integrate/phase-N`
- prerequisite implementation line:
  - Claude hook harness and schema validation already exist in-repo
  - Codex local prototype evidence from `Hook Phase 6 / N`
- this phase is planning-first and harness-first, not runtime-promotion-first

## Integration Branch

Accepted `Phase N` sprint outputs merge into:

- `integrate/phase-N`

Rules:

- sprint branches do not write accepted rows directly into
  `docs/phase-N/readiness.md`
- the integration author updates `docs/phase-N/readiness.md` only after the
  accepted sprint output is merged into `integrate/phase-N`
- any sprint prerequisite that requires prior accepted outputs means accepted
  and merged to `integrate/phase-N`, not merely frozen on a feature branch

## Phase Entry Criteria

`Phase N` planning may proceed because:

- the Claude harness is already the reference implementation path
- Codex local evidence now includes working `notify`, `PreToolUse`, and
  SessionStart-backed session-record behavior
- Gemini is installed locally and is ready for first-pass hook-surface capture

`Phase N` execution should not promote provider runtime behavior until:

- Codex and Gemini each have repo-owned raw fixtures for every locally
  exercisable hook surface
- Codex and Gemini each have validation models derived from those fixtures
- Codex and Gemini each have automated schema-proof tests in the harness
- normalization candidates are explicitly mapped from approved fixtures

## Sprint Sequence

### N.1 Codex Harness Schema Capture

Purpose:

- build the Codex live-capture harness in the same style as the Claude harness
- capture every locally exercisable Codex hook surface
- freeze the authoritative Codex fixture set and findings ledger

Execution branch:
- `feature/pN-s1-codex-harness-schema-capture`

Execution worktree:
- `../schook-worktrees/feature/pN-s1-codex-harness-schema-capture`

### N.2 Gemini Harness Schema Capture

Purpose:

- build the Gemini live-capture harness in parallel with the Codex harness
- capture every locally exercisable Gemini hook surface
- freeze the authoritative Gemini fixture set and findings ledger

Execution branch:
- `feature/pN-s2-gemini-harness-schema-capture`

Execution worktree:
- `../schook-worktrees/feature/pN-s2-gemini-harness-schema-capture`

### N.3 Cross-Provider Normalization Inventory

Purpose:

- compare Codex and Gemini approved fixtures against the Claude baseline
- identify which fields map cleanly into canonical `schooks` concepts
- document unresolved provider differences before runtime work begins
- reconcile `requirements`, `architecture`, and `project-plan` to the approved
  Phase N planning boundary

Execution branch:
- `feature/pN-s3-cross-provider-normalization-inventory`

Execution worktree:
- `../schook-worktrees/feature/pN-s3-cross-provider-normalization-inventory`

### N.4 Promotion Gate

Purpose:

- decide whether Codex and Gemini are ready for runtime adapter work
- freeze the provider evidence, open issues, and explicit non-goals
- record the final Phase N readiness verdict
- freeze the authoritative project-level go/no-go record for later runtime work

Execution branch:
- `feature/pN-s4-promotion-gate`

Execution worktree:
- `../schook-worktrees/feature/pN-s4-promotion-gate`

## Sprint Artifact Summary

`Phase N` uses one named artifact set throughout execution:

- `N.1`:
  - `docs/phase-N/codex-capture-checklist.md`
  - `docs/phase-N/codex-findings-ledger.md`
  - `test-harness/hooks/codex/fixtures/approved/manifest.json`
  - `test-harness/hooks/codex/models/`
  - `test-harness/hooks/codex/tests/`
  - Codex drift artifact
  - `docs/hook-api/codex-hook-api.md`
- `N.2`:
  - `docs/phase-N/gemini-capture-checklist.md`
  - `docs/phase-N/gemini-findings-ledger.md`
  - `test-harness/hooks/gemini/fixtures/approved/manifest.json`
  - `test-harness/hooks/gemini/models/`
  - `test-harness/hooks/gemini/tests/`
  - Gemini drift artifact
  - `docs/hook-api/gemini-hook-api.md`
- `N.3`:
  - `docs/phase-N/normalization-checklist.md`
  - `docs/phase-N/normalization-findings-ledger.md`
- `N.4`:
  - `docs/phase-N/release-checklist.md`
  - `docs/phase-N/readiness.md`

The sprint docs remain the only authoritative source for per-sprint
deliverables, acceptance criteria, and closure rules.

## Phase Rules

- Codex and Gemini should be executed in parallel where possible
- `docs/phase-N/readiness.md` is read-only in parallel sprint branches; only
  the integration author updates accepted rows after sprint acceptance, per
  `ADR-SHK-007`
- the canonical non-exercisable disposition string for `Phase N` ledgers and
  manifests is `confirmed-not-exercisable`
- schema proof means payload plus hook-process environment coverage at each hook
  point
- no provider field may be mapped into `schooks` without repo-owned fixture
  evidence and model validation
- `Phase N` is still a harness/documentation/normalization phase, not a runtime
  promotion guarantee
- `Phase N` does not close ATM integration or idle-notification promotion; any
  ATM-aware idle prototype evidence remains planning input only until a later
  runtime phase approves it explicitly
- later runtime work must cite the `Phase N` fixtures/models rather than local
  shell behavior or provider memory

## Initial Planning Outputs

- `docs/plan-phase-N.md`
- `docs/phase-N/codex-capture-checklist.md`
- `docs/phase-N/codex-findings-ledger.md`
- `docs/phase-N/gemini-capture-checklist.md`
- `docs/phase-N/gemini-findings-ledger.md`
- `docs/phase-N/normalization-checklist.md`
- `docs/phase-N/normalization-findings-ledger.md`
- `docs/phase-N/release-checklist.md`
- `docs/phase-N/readiness.md`
- `docs/phase-N/sprint-N1.md`
- `docs/phase-N/sprint-N2.md`
- `docs/phase-N/sprint-N3.md`
- `docs/phase-N/sprint-N4.md`
