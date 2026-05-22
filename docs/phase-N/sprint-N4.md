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
- `integrate/phase-N` is the `Phase N` integration branch; accepted `N.1`,
  `N.2`, and `N.3` outputs must already be merged there before `N.4` begins
- `docs/phase-N/readiness.md` (`N.4` records the final verdict, but the
  integration author remains the sole writer for accepted sprint rows and final
  verdict updates)
- approved outputs of `N.1`, `N.2`, and `N.3`

## Prerequisites

- Codex and Gemini fixture/model/test lines are complete on `integrate/phase-N`
- normalization inventory is merged to `integrate/phase-N` and the
  corresponding readiness rows were updated by the integration author

## Exact Targets

- `docs/phase-N/release-checklist.md`
- `docs/phase-N/readiness.md`
- `docs/project-plan.md`
- `docs/plan-cross-provider-hooks.md`
- `docs/traceability.md`

## Deliverables

Every listed deliverable is expected to land at a production-ready level for
the scope this sprint claims. If that cannot be done cleanly in one sprint, the
sprint must be split before implementation begins. No deliverable may be
silently dropped or partially deferred.

- final `Phase N` release checklist
- final `Phase N` readiness verdict
- explicit approved/deferred record for Codex and Gemini promotion
- summary updates in `docs/project-plan.md` and
  `docs/plan-cross-provider-hooks.md` that reference the final readiness
  verdict

## Required Work

- review the final Codex and Gemini harness evidence sets
- review normalization findings and open gaps
- record whether runtime adapter work is approved, deferred, or partially
  approved
- record the exact approved surfaces, deferred surfaces, open blocking
  findings, and open important findings that justify the final verdict
- freeze the final readiness verdict
- update `docs/project-plan.md` and `docs/plan-cross-provider-hooks.md` so
  they summarize and reference the final verdict from `docs/phase-N/readiness.md`
- update `docs/traceability.md` for the final `Phase N` closure status of
  `HKR-014` and `HKR-015`, marking them `implemented` for approved surfaces
  that close under `GO` / `PARTIAL_GO`, or `deferred` with a named follow-on
  sprint reference for surfaces that remain unapproved
- under full `NO_GO`, record `HKR-014` and `HKR-015` as `deferred` with reason
  `Phase N NO_GO` and follow-on sprint reference `TBD — to be assigned in
  remediation sprint planning`
- treat `HKR-016` as a no-op in `N.4` if `N.3` already marked it implemented;
  only update `HKR-016` in `N.4` if the prior sprint left it unresolved
- limit the `docs/project-plan.md` edit in `N.4` to the targeted `Phase N` row
  status update and final verdict summary; do not replace or remove the
  boundary note added by `N.3`
- this sprint must not introduce or modify Rust code; if execution uncovers a
  need for Rust runtime or plugin changes, split that work into a later
  approved runtime sprint

## Promotion Gate Criteria

`N.4` must use these criteria before any provider runtime adapter work is
authorized:

- `GO`:
  - approved fixtures exist for all locally exercisable hook surfaces recorded
    in the `N.1` and `N.2` findings ledgers
  - no surface remains in a `to be captured` state
  - `docs/phase-N/normalization-findings-ledger.md` has zero open `BLOCKING`
    or `IMPORTANT` findings
- `PARTIAL_GO`:
  - the readiness verdict names the exact approved surfaces and the exact
    deferred surfaces for each provider
  - any deferred surface includes a written reason and follow-on sprint target
- `NO_GO`:
  - any provider still has unresolved required capture work
  - or `N.3` still has open `BLOCKING` or `IMPORTANT` normalization findings
  - or the final readiness record cannot name the exact approved/deferred
    surfaces per provider

`NO_GO` suspends runtime adapter work and requires a new sprint before
promotion can be retried.

## Explicit Code Samples

If the sprint introduces or changes important traits, features, enums, protocol
types, boundary contracts, or execution seams, this section must include
explicit code samples or signatures showing the intended end state.

```json
{
  "release_verdict": "GO | PARTIAL_GO | NO_GO",
  "providers": {
    "codex": {
      "approved_surfaces": ["notify", "PreToolUse"],
      "deferred_surfaces": ["Stop"],
      "open_blocking_findings": 0,
      "open_important_findings": 0
    },
    "gemini": {
      "approved_surfaces": [],
      "deferred_surfaces": ["preTool", "postTool"],
      "open_blocking_findings": 0,
      "open_important_findings": 1
    }
  },
  "next_action": "authorize-runtime-adapter-work | plan-follow-on-sprint"
}
```

## This Sprint Does Not Close

- actual provider runtime adapter implementation
- release of Codex or Gemini support as shipped product behavior
- ATM integration or idle-notification promotion
- any Rust runtime or plugin implementation work

## Acceptance Criteria

- `docs/phase-N/release-checklist.md` records the final promotion-gate result
- `docs/phase-N/readiness.md` records the final `Phase N` verdict
- Codex and Gemini each have an explicit approved/deferred disposition
- the phase leaves one authoritative go/no-go record for subsequent runtime
  work
- `docs/phase-N/readiness.md` is the authoritative go/no-go record
- the final readiness record names approved surfaces, deferred surfaces, open
  blocking findings, and open important findings for each provider
- a `GO` verdict is impossible unless every locally exercisable surface has a
  final disposition and `N.3` has zero open `BLOCKING` or `IMPORTANT`
  findings
- `docs/traceability.md` reflects the final status for `HKR-014`, `HKR-015`,
  and, only if still unresolved from `N.3`, `HKR-016` before `N.4` closes
- under full `NO_GO`, `docs/traceability.md` records `HKR-014` and `HKR-015`
  as `deferred` with reason `Phase N NO_GO` and follow-on sprint reference
  `TBD — to be assigned in remediation sprint planning`
- `docs/project-plan.md` summarizes the verdict from
  `docs/phase-N/readiness.md`
- `docs/plan-cross-provider-hooks.md` summarizes the verdict from
  `docs/phase-N/readiness.md`
- before any `GO` or `PARTIAL_GO` verdict is recorded for a provider,
  `docs/architecture.md` section `9.3` must confirm the hook trait seal
  requirement is carried forward as a mandatory prerequisite for any runtime
  adapter sprint rather than deferred until after the first runtime crate lands

## Required Validation

- `pytest test-harness/hooks/`
- `git diff --check`
