---
id: N.1
title: Codex Harness Schema Capture
status: planned
branch: feature/pN-s1-codex-harness-schema-capture
worktree: ../schook-worktrees/feature/pN-s1-codex-harness-schema-capture
target: plan/phase-N
---

# Sprint N.1 — Codex Harness Schema Capture

```yaml
plan_type: sprint_plan
phase: N
sprint: N.1
worktree: ../schook-worktrees/feature/pN-s1-codex-harness-schema-capture
branch: feature/pN-s1-codex-harness-schema-capture
status: planned
estimated_scope: large
```

## Goal

- build a Codex harness line parallel to the existing Claude harness
- capture every locally exercisable Codex hook surface
- prove payload fields, hook env vars, and control semantics with repo-owned
  fixtures

## Hard Dependencies

- `docs/plan-phase-N.md`
- `docs/phase-N/readiness.md`
- `docs/hook-api/codex-hook-api.md`
- the existing Claude harness under `test-harness/hooks/claude/`

## Prerequisites

- the Codex local prototype evidence remains available for reference
- local Codex hook wiring is stable enough to reproduce captures

## Exact Targets

- `test-harness/hooks/codex/`
- `test_harness/hooks/codex/`
- `docs/phase-N/codex-capture-checklist.md`
- `docs/phase-N/codex-findings-ledger.md`
- `docs/hook-api/codex-hook-api.md`

## Deliverables

Every listed deliverable is expected to land at a production-ready level for
the scope this sprint claims. If that cannot be done cleanly in one sprint, the
sprint must be split before implementation begins. No deliverable may be
silently dropped or partially deferred.

- approved raw Codex fixtures for every locally exercisable hook surface
- hook-process environment snapshots for those same surfaces
- an approved field inventory covering every observed payload field and hook
  env var for each captured surface
- provider-specific Codex validation models
- pytest schema-proof tests for Codex fixtures/models
- a Codex drift artifact suitable for future version-bump checks
- updated Codex API evidence documentation matching only captured proof

## Required Work

- adapt the Claude harness pattern to Codex capture workflows
- enumerate every hook surface Codex actually fires locally
- capture raw payload plus env for each surface
- record the Codex CLI version and hook registration path used for each capture
- classify control semantics for each surface
- freeze the checklist and findings ledger for `N.1`
- promote only validated findings into the Codex doc set

## Explicit Code Samples

If the sprint introduces or changes important traits, features, enums, protocol
types, boundary contracts, or execution seams, this section must include
explicit code samples or signatures showing the intended end state.

- no runtime trait changes are authorized in `N.1`; code examples should remain
  harness/model-oriented only

## This Sprint Does Not Close

- Codex runtime adapter promotion
- Gemini harness work
- cross-provider mapping decisions beyond documented candidates

## Acceptance Criteria

- every locally tested Codex hook point has a repo-owned raw fixture
- every captured Codex hook point has an env snapshot fixture
- every approved fixture validates against a provider-specific model
- every approved Codex payload field and hook env var is enumerated in the
  approved fixtures or provider models
- Codex pytest schema-proof tests fail on fixture/model drift
- `docs/phase-N/codex-findings-ledger.md` is the only authoritative handoff
  ledger for Codex capture findings

## Required Validation

- `pytest test-harness/hooks/codex/tests/ -m provider_codex -v`
- `git diff --check`
