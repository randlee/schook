---
id: N.3
title: Cross-Provider Normalization Inventory
status: complete
branch: feature/pN-s3-cross-provider-normalization-inventory
worktree: ../schook-worktrees/feature/pN-s3-cross-provider-normalization-inventory
target: plan/phase-N
---

# Sprint N.3 — Cross-Provider Normalization Inventory

```yaml
plan_type: sprint_plan
phase: N
sprint: N.3
worktree: ../schook-worktrees/feature/pN-s3-cross-provider-normalization-inventory
branch: feature/pN-s3-cross-provider-normalization-inventory
status: complete
estimated_scope: large
```

## Goal

- compare Codex and Gemini approved fixtures against the existing Claude
  baseline
- define which verified provider fields map into canonical `schooks` concepts
- freeze unresolved provider differences before runtime work begins

## Hard Dependencies

- `docs/plan-phase-N.md`
- `integrate/phase-N` is the `Phase N` integration branch; `N.1` and `N.2`
  accepted outputs must already be merged there before `N.3` begins
- `docs/phase-N/readiness.md` (`read-only` during `N.3`; only the integration
  author updates accepted rows after sprint acceptance)
- `docs/architecture.md` (`ADR-SHK-007` governs the readiness ownership rule)
- `docs/architecture.md` (`ADR-SHK-006` was introduced by the `Phase N`
  planning branch and must be carried to `integrate/phase-N` before `N.3`
  begins; `N.3` must cite it)
- accepted and merged-to-integration-branch output of `N.1`
- accepted and merged-to-integration-branch output of `N.2`
- `docs/hook-api/gemini-hook-api.md` (created by `N.2`; must exist on the
  integration branch before `N.3` begins)

## Prerequisites

- Codex and Gemini fixture/model sets are already merged to the integration
  branch and the corresponding `docs/phase-N/readiness.md` rows were updated by
  the integration author; feature-branch fixture freeze alone is not sufficient
- provider API evidence docs are updated to match those fixtures

## Exact Targets

- `docs/phase-N/normalization-checklist.md`
- `docs/phase-N/normalization-findings-ledger.md`
- `docs/plan-cross-provider-hooks.md`
- `docs/hook-api/claude-hook-api.md`
- `docs/hook-api/codex-hook-api.md`
- `docs/hook-api/gemini-hook-api.md`
- `docs/requirements.md`
- `docs/architecture.md`
- `docs/project-plan.md`
- `docs/traceability.md`

## Deliverables

Every listed deliverable is expected to land at a production-ready level for
the scope this sprint claims. If that cannot be done cleanly in one sprint, the
sprint must be split before implementation begins. No deliverable may be
silently dropped or partially deferred.

- a field-by-field normalization inventory for Claude, Codex, and Gemini
- an explicit list of canonical mapping candidates backed by fixture evidence
- an explicit list of non-mappable or unresolved provider differences
- updated planning docs reflecting the actual adapter gap after schema proof
- reconciled control docs describing the approved Phase N harness-planning
  boundary and the still-deferred runtime boundary
- explicit sprint-scope note that `N.3` classifies reviewed field families and
  findings by evidence-backed disposition rather than targeting a fixed raw
  field-count or unresolved-difference count

## Required Work

- compare approved provider fixtures and model schemas
- group fields into canonical, provider-specific, and unresolved sets
- document correlation, lifecycle, and root-recovery differences by provider
- freeze the normalization checklist and findings ledger for `N.3`
- update `HKR-014` and `HKR-015` from `Planned` to `Implemented` once accepted
  `N.1` / `N.2` fixture evidence is merged to `integrate/phase-N`
- update `HKR-016` from `Planned` to `Implemented` citing `ADR-SHK-007` and
  the accepted readiness-ownership boundary
- cite the existing provider-normalization boundary ADR from this sprint
- update the `HKR-014`, `HKR-015`, and `HKR-016` traceability rows from
  `planned` to `implemented`, citing the accepted fixture evidence,
  normalization ledger, `ADR-SHK-007`, and readiness-ownership docs
- if normalization tooling encounters a missing or unparsable fixture, it
  must emit a structured error identifying the provider, fixture path, and
  missing or invalid field rather than raising a bare exception; add a
  normalization harness test that exercises this path
- reconcile `docs/requirements.md`, `docs/architecture.md`, and
  `docs/project-plan.md` so no provider capability is overstated or
  understated versus the verified `Phase N` planning boundary
- limit the `docs/project-plan.md` edit in `N.3` to the `Phase N` boundary
  note and readiness pointer; the `Phase N` row remains `In progress` in this
  sprint
- this sprint must not introduce or modify Rust code; if execution uncovers a
  need for Rust runtime or plugin changes, split that work into a later
  approved runtime sprint

## Explicit Code Samples

If the sprint introduces or changes important traits, features, enums, protocol
types, boundary contracts, or execution seams, this section must include
explicit code samples or signatures showing the intended end state.

- if any field is added to the canonical normalization candidates list, include
  an explicit JSON shape example showing the normalized field name, type, and
  source provider fields it maps from
- semantic identifier fields promoted from provider fixtures into future Rust
  runtime types must become Newtype wrappers rather than bare `String`
  aliases, for example:

```rust
pub struct ThreadId(String);
pub struct SessionId(String);
```

## This Sprint Does Not Close

- provider runtime adapter implementation
- debounce or idle orchestration promotion
- release sign-off for provider support
- any Rust runtime or plugin implementation work

## Acceptance Criteria

- every mapping candidate cites provider-owned fixture evidence
- every unresolved difference is recorded explicitly
- explicit do-not-map items are listed in
  `docs/phase-N/normalization-checklist.md` or
  `docs/phase-N/normalization-findings-ledger.md`
- `docs/phase-N/normalization-findings-ledger.md` is the only authoritative
  handoff ledger for normalization findings
- planning docs no longer rely on guessed provider contracts
- `docs/requirements.md`, `docs/architecture.md`, and `docs/project-plan.md`
  describe the same approved Phase N boundary
- `docs/requirements.md` records `HKR-014`, `HKR-015`, and `HKR-016` as
  `Implemented` once their accepted execution evidence is merged to
  `integrate/phase-N`
- `docs/architecture.md` includes `ADR-SHK-006` and `N.3` cites it as the
  normalization boundary decision governing canonical fields, provider-local
  fields, unresolved fields, and the deferred runtime-adapter boundary
- `docs/hook-api/gemini-hook-api.md` exists with at least one named surface
  entry before `N.3` updates it
- `docs/project-plan.md` adds a Phase N boundary note that cites
  `docs/phase-N/readiness.md` as the authoritative verdict record and states
  that `N.4` is the step that changes the Phase N row from `In progress` to
  `Completed`
- `docs/traceability.md` records `HKR-014`, `HKR-015`, and `HKR-016` as
  `implemented` with accepted evidence citations
- normalization harness tests prove missing or unparsable fixtures fail with
  structured provider/path/field diagnostics rather than a bare exception
- sprint closure is based on complete classification of the reviewed
  field-family matrix, not on a pre-set count of canonical candidates or
  unresolved differences; the current accepted matrix closes with four
  canonical candidates, seven provider-local findings, and two unresolved
  findings

## Required Validation

- `pytest test-harness/hooks/`
- `git diff --check`
