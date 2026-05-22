---
id: N.3
title: Cross-Provider Normalization Inventory
status: planned
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
status: planned
estimated_scope: medium
```

## Goal

- compare Codex and Gemini approved fixtures against the existing Claude
  baseline
- define which verified provider fields map into canonical `schooks` concepts
- freeze unresolved provider differences before runtime work begins

## Hard Dependencies

- `docs/plan-phase-N.md`
- `docs/phase-N/readiness.md`
- approved output of `N.1`
- approved output of `N.2`

## Prerequisites

- Codex and Gemini fixture/model sets are already frozen
- provider API evidence docs are updated to match those fixtures

## Exact Targets

- `docs/phase-N/normalization-checklist.md`
- `docs/phase-N/normalization-findings-ledger.md`
- `docs/plan-cross-provider-hooks.md`
- provider hook API evidence docs touched by the mapping review

## Deliverables

Every listed deliverable is expected to land at a production-ready level for
the scope this sprint claims. If that cannot be done cleanly in one sprint, the
sprint must be split before implementation begins. No deliverable may be
silently dropped or partially deferred.

- a field-by-field normalization inventory for Claude, Codex, and Gemini
- an explicit list of canonical mapping candidates backed by fixture evidence
- an explicit list of non-mappable or unresolved provider differences
- updated planning docs reflecting the actual adapter gap after schema proof

## Required Work

- compare approved provider fixtures and model schemas
- group fields into canonical, provider-specific, and unresolved sets
- document correlation, lifecycle, and root-recovery differences by provider
- freeze the normalization checklist and findings ledger for `N.3`

## Explicit Code Samples

If the sprint introduces or changes important traits, features, enums, protocol
types, boundary contracts, or execution seams, this section must include
explicit code samples or signatures showing the intended end state.

- include explicit normalized field-shape examples if new canonical adapter
  fields are proposed

## This Sprint Does Not Close

- provider runtime adapter implementation
- debounce or idle orchestration promotion
- release sign-off for provider support

## Acceptance Criteria

- every mapping candidate cites provider-owned fixture evidence
- every unresolved difference is recorded explicitly
- `docs/phase-N/normalization-findings-ledger.md` is the only authoritative
  handoff ledger for normalization findings
- planning docs no longer rely on guessed provider contracts

## Required Validation

- `pytest test-harness/hooks/`
- `git diff --check`
