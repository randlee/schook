---
id: N.4
title: Promotion Gate
status: planned
branch: feature/pN-s4-promotion-gate
worktree: ../schook-worktrees/feature/pN-s4-promotion-gate
target: plan/phase-N
---

# Sprint N.4 — Promotion Gate

```yaml
plan_type: sprint_plan
phase: N
sprint: N.4
worktree: ../schook-worktrees/feature/pN-s4-promotion-gate
branch: feature/pN-s4-promotion-gate
status: planned
estimated_scope: medium
```

## Goal

- decide whether Codex and Gemini have enough proven contract coverage for
  runtime adapter work to begin
- freeze the final `Phase N` readiness record

## Hard Dependencies

- `docs/plan-phase-N.md`
- `docs/phase-N/readiness.md`
- approved outputs of `N.1`, `N.2`, and `N.3`

## Prerequisites

- Codex and Gemini fixture/model/test lines are complete
- normalization inventory is frozen

## Exact Targets

- `docs/phase-N/release-checklist.md`
- `docs/phase-N/readiness.md`
- `docs/project-plan.md`
- `docs/plan-cross-provider-hooks.md`

## Deliverables

Every listed deliverable is expected to land at a production-ready level for
the scope this sprint claims. If that cannot be done cleanly in one sprint, the
sprint must be split before implementation begins. No deliverable may be
silently dropped or partially deferred.

- final `Phase N` release checklist
- final `Phase N` readiness verdict
- explicit approved/deferred record for Codex and Gemini promotion
- one authoritative project-level record of the final Phase N go/no-go decision

## Required Work

- review the final Codex and Gemini harness evidence sets
- review normalization findings and open gaps
- record whether runtime adapter work is approved, deferred, or partially
  approved
- freeze the final readiness verdict
- reconcile any final project-level status wording with the release verdict

## Explicit Code Samples

If the sprint introduces or changes important traits, features, enums, protocol
types, boundary contracts, or execution seams, this section must include
explicit code samples or signatures showing the intended end state.

- none required unless the sprint explicitly authorizes a new canonical adapter
  shape

## This Sprint Does Not Close

- actual provider runtime adapter implementation
- release of Codex or Gemini support as shipped product behavior

## Acceptance Criteria

- `docs/phase-N/release-checklist.md` records the final promotion-gate result
- `docs/phase-N/readiness.md` records the final `Phase N` verdict
- Codex and Gemini each have an explicit approved/deferred disposition
- the phase leaves one authoritative go/no-go record for subsequent runtime
  work
- `docs/project-plan.md` and `docs/plan-cross-provider-hooks.md` reflect the
  same final promotion verdict

## Required Validation

- `pytest test-harness/hooks/`
- `git diff --check`
