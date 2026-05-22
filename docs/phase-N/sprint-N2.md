---
id: N.2
title: Gemini Harness Schema Capture
status: planned
branch: feature/pN-s2-gemini-harness-schema-capture
worktree: ../schook-worktrees/feature/pN-s2-gemini-harness-schema-capture
target: plan/phase-N
---

# Sprint N.2 — Gemini Harness Schema Capture

```yaml
plan_type: sprint_plan
phase: N
sprint: N.2
worktree: ../schook-worktrees/feature/pN-s2-gemini-harness-schema-capture
branch: feature/pN-s2-gemini-harness-schema-capture
status: planned
estimated_scope: large
```

## Goal

- build a Gemini harness line parallel to the existing Claude harness
- capture every locally exercisable Gemini hook surface
- prove payload fields, hook env vars, and control semantics with repo-owned
  fixtures

## Hard Dependencies

- `docs/plan-phase-N.md`
- `integrate/phase-N` is the `Phase N` integration branch for accepted sprint
  outputs and readiness updates
- `docs/phase-N/readiness.md` (`read-only` during `N.2`; only the integration
  author updates accepted rows after sprint acceptance)
- `docs/plan-cross-provider-hooks.md`
- the existing Claude harness under `test-harness/hooks/claude/`

## Prerequisites

- local Gemini CLI and `gemini hooks` surfaces are available for testing
- the capture matrix is frozen by `team-lead` or `chook` before the first
  `N.2` harness commit:
  - freeze means `docs/phase-N/gemini-capture-checklist.md` is already
    committed with at least one named Gemini hook-surface entry
  - that checklist-freeze commit is the execution gate for later `N.2` harness
    work; the sprint must not create and freeze the matrix implicitly inside
    the first harness commit

## Exact Targets

- `test-harness/hooks/gemini/`
- `docs/phase-N/gemini-capture-checklist.md`
- `docs/phase-N/gemini-findings-ledger.md`
- `docs/hook-api/gemini-hook-api.md`

## Deliverables

Every listed deliverable is expected to land at a production-ready level for
the scope this sprint claims. If that cannot be done cleanly in one sprint, the
sprint must be split before implementation begins. No deliverable may be
silently dropped or partially deferred.

- approved raw Gemini fixtures for every locally exercisable hook surface
- hook-process environment snapshots for those same surfaces
- an approved field inventory covering every observed payload field and hook
  env var for each captured surface
- provider-specific Gemini validation models
- the all-blocked MVC placeholder model artifact when no Gemini surfaces are
  capturable locally
- pytest schema-proof tests for Gemini fixtures/models
- a Gemini fixture manifest / drift artifact suitable for future version-bump
  checks
- the first `schook`-owned Gemini API evidence document

## Required Work

- adapt the Claude harness pattern to Gemini capture workflows
- enumerate every hook surface Gemini actually fires locally
- capture raw payload plus env for each surface
- record the Gemini CLI version and hook registration path used for each capture
- verify whether output-format choice changes hook-observable behavior
- classify control semantics for each surface using the schema defined in
  `docs/plan-cross-provider-hooks.md`: blocking vs non-blocking, exit-code
  handling, and stdout/stderr contract
- record every attempted Gemini hook surface in the checklist and findings
  ledger with one of: `captured` or `confirmed-not-exercisable`
- add a provider-gemini structural harness test that passes even when every
  Gemini hook surface is blocked locally, so the validation gate still proves
  the harness exists in an all-blocked MVC outcome
- in the all-blocked MVC outcome, write at least one parseable Gemini provider
  model file under `test-harness/hooks/gemini/models/` that records an empty
  approved-field set and the blocking reason
- freeze the checklist and findings ledger for `N.2`
- promote only validated findings into Gemini evidence docs

## Explicit Code Samples

If the sprint introduces or changes important traits, features, enums, protocol
types, boundary contracts, or execution seams, this section must include
explicit code samples or signatures showing the intended end state.

Approved Gemini fixture manifest / drift artifact shape:

```json
{
  "provider": "gemini",
  "gemini_version": "gemini-cli 0.0.0",
  "capture_date": "2026-05-22T00:00:00Z",
  "capture_root": "test-harness/hooks/gemini/fixtures/approved/",
  "hook_surfaces": [
    {
      "surface": "preTool",
      "status": "captured",
      "payload_fixture": "preTool/payload.json",
      "env_fixture": "preTool/env.json",
      "control_semantics": {
        "blocking": true,
        "exit_code_contract": "non-zero blocks tool execution",
        "stdio_contract": "stdout/stderr consumed as hook response"
      }
    },
    {
      "surface": "sessionStart",
      "status": "confirmed-not-exercisable",
      "reason": "surface not exposed by local gemini hooks runtime"
    }
  ]
}
```

MVC placeholder provider-model shape when every Gemini surface is
`confirmed-not-exercisable`:

```json
{
  "provider": "gemini",
  "model_name": "empty-approved-fields",
  "approved_fields": [],
  "status": "confirmed-not-exercisable",
  "reason": "no Gemini hook surfaces were capturable locally during N.2"
}
```

## This Sprint Does Not Close

- Codex runtime adapter promotion
- Gemini runtime adapter promotion
- cross-provider mapping decisions beyond documented candidates
- final provider promotion verdict

## Minimum Viable Closure

If Gemini exposes no locally exercisable hook surfaces during `N.2`, the sprint
still closes only by documenting that result explicitly:

- `docs/phase-N/gemini-findings-ledger.md` must list every attempted surface as
  `confirmed-not-exercisable`
- each blocked surface row must include `reason: <why it could not be
  exercised>`
- `docs/phase-N/gemini-capture-checklist.md` must record the corresponding
  status as `BLOCKED`, not `COMPLETE`
- the provider-gemini pytest run must still collect and pass at least one
  structural harness-layout test in this all-blocked outcome
- `test-harness/hooks/gemini/models/` must still contain at least one parseable
  Gemini provider-model file recording the empty approved-field set and the
  blocking reason

That documented blocked outcome is the minimum viable closure path for surfaces
that cannot be exercised locally.

## Acceptance Criteria

- every locally tested Gemini hook point has a repo-owned raw fixture, or the
  findings ledger records `confirmed-not-exercisable` with a reason
- every captured Gemini hook point has an env snapshot fixture
- every approved fixture validates against a provider-specific model
- `docs/phase-N/gemini-capture-checklist.md` contains at least one explicitly
  named Gemini hook-surface entry committed before the first `N.2` harness
  commit
- the `gemini-capture-checklist.md` freeze commit and the first `N.2` harness
  code commit are separate git commits; the freeze commit must pre-date any
  harness code commit in the sprint branch
- every approved Gemini payload field and hook env var is enumerated in the
  approved fixtures or provider models
- `test-harness/hooks/gemini/fixtures/approved/manifest.json` records
  `provider`, `gemini_version`, `capture_date`, `capture_root`, and
  `hook_surfaces` by name
- every captured Gemini manifest surface records `control_semantics` with
  `blocking`, `exit_code_contract`, and `stdio_contract`
- Gemini pytest schema-proof tests fail on fixture/model drift
- the provider-gemini pytest validation gate collects and passes at least one
  structural harness-layout test regardless of whether fixture capture closes
  through full capture or the all-blocked MVC path
- in the all-blocked MVC outcome, `test-harness/hooks/gemini/models/`
  contains at least one parseable Gemini provider-model file recording an empty
  approved-field set with a blocking reason
- `docs/phase-N/gemini-capture-checklist.md` and
  `docs/phase-N/gemini-findings-ledger.md` contain a per-surface final
  disposition for every attempted Gemini hook point
- `docs/phase-N/gemini-findings-ledger.md` is the only authoritative handoff
  ledger for Gemini capture findings

## Required Validation

- `pytest test-harness/hooks/gemini/tests/ -m provider_gemini -v`
- `git log --oneline -- docs/phase-N/gemini-capture-checklist.md test-harness/hooks/gemini/`
  proves the checklist-freeze commit predates the first harness code commit and
  is a separate commit
- `git diff --check`
