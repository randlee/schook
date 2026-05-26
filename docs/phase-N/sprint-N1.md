---
id: N.1
title: Codex Harness Schema Capture
status: complete
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
status: complete
estimated_scope: large
```

## Goal

- build a Codex harness line parallel to the existing Claude harness
- capture every locally exercisable Codex hook surface
- prove payload fields, hook env vars, and control semantics with repo-owned
  fixtures

## Hard Dependencies

- `docs/plan-phase-N.md`
- `integrate/phase-N` is the `Phase N` integration branch for accepted sprint
  outputs and readiness updates
- `docs/phase-N/readiness.md` (`read-only` during `N.1`; only the integration
  author updates accepted rows after sprint acceptance)
- `docs/hook-api/codex-hook-api.md`
- the existing Claude harness under `test-harness/hooks/claude/`

## Prerequisites

- the Codex local prototype evidence remains available for reference
- local Codex hook wiring is stable enough to reproduce captures

## Exact Targets

- `test-harness/hooks/codex/`
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
- an extension of the existing debounce prototype into a full fixture-capture
  harness; the five current debounce tests are baseline evidence only and do
  not by themselves close `N.1`
- provider-specific Codex validation models
- pytest schema-proof tests for Codex fixtures/models
- a Codex fixture manifest / drift artifact suitable for future version-bump
  checks
- updated Codex API evidence documentation matching only captured proof

## Required Work

- adapt the Claude harness pattern to Codex capture workflows
- extend the existing debounce-only prototype so `N.1` covers the full capture
  spec rather than counting the current five tests as sprint closure
- enumerate every hook surface Codex actually fires locally
- capture raw payload plus env for each surface
- record the Codex CLI version and hook registration path used for each capture
- classify control semantics for each surface using the schema defined in
  `docs/plan-cross-provider-hooks.md`: blocking vs non-blocking, exit-code
  handling, and stdout/stderr contract
- maintain an approved manifest that records every audited Codex surface with a
  per-surface disposition: `captured` or `confirmed-not-exercisable`
- classify every Codex checklist surface before the first `N.1` harness commit
  as either `direct hook surface (harness-capturable)` or
  `relay-synthetic (confirmed-not-exercisable)` with the reason recorded in
  `docs/phase-N/codex-capture-checklist.md`
- classify `resume`, `fork`, and `--cd` explicitly in
  `docs/phase-N/codex-capture-checklist.md` before the first `N.1` harness
  commit, and carry each one through to a per-surface disposition in the
  approved manifest or Codex findings ledger
- any field in the provider model that represents a semantic identifier
  (`session_id`, run ID, provider version, hook surface name used as a
  correlation key) must be flagged in the findings ledger as a newtype
  candidate for later Rust runtime code, distinct from display-only string
  fields
- add a provider-codex pytest that fails if the approved manifest only
  describes the debounce prototype surfaces without a final disposition for the
  rest of the audited Codex surface set
- ensure the `provider_codex` pytest suite asserts that
  `test-harness/hooks/codex/fixtures/approved/manifest.json` exists and
  carries the required top-level keys
- ensure every field table kept in `docs/hook-api/codex-hook-api.md` after `N.1`
  is either backed by a repo-owned `N.1` fixture citation or explicitly labeled
  `relay-only planning evidence — not part of approved schook fixture
  inventory`
- freeze the checklist and findings ledger for `N.1`
- promote only validated findings into the Codex doc set
- this sprint must not introduce or modify Rust code; if execution uncovers a
  need for Rust runtime or plugin changes, split that work into a later
  approved runtime sprint

## Capture Scope Boundary

`N.1` captures only `schook`-owned hook inputs observed directly by the local
Codex harness:

- raw stdin payloads as delivered to the capture script
- hook-process environment variables as delivered to the capture script
- harness-owned control-semantic observations captured during local execution

Reference material from `agent-team-mail`, `atm-hook-relay.py`, or any other
relay-side event pipeline may be cited as planning evidence, but those fields
must not be promoted into the approved `N.1` fixture inventory unless the
`schook` harness captures them directly.

## Explicit Code Samples

If the sprint introduces or changes important traits, features, enums, protocol
types, boundary contracts, or execution seams, this section must include
explicit code samples or signatures showing the intended end state.

Approved Codex fixture manifest / drift artifact shape:

```json
{
  "provider": "codex",
  "codex_version": "codex-cli 0.133.0",
  "capture_date": "2026-05-22T00:00:00Z",
  "capture_root": "test-harness/hooks/codex/fixtures/approved/",
  "prototype_baseline": {
    "existing_tests": 5,
    "surfaces": ["notify", "PreToolUse"],
    "note": "baseline only; N.1 closure requires the full audited surface set"
  },
  "hook_surfaces": [
    {
      "surface": "notify",
      "status": "captured",
      "payload_fixture": "notify/agent-turn-complete.json",
      "env_fixture": "notify/env.json",
      "control_semantics": {
        "blocking": false,
        "exit_code_contract": "ignored",
        "stdio_contract": "notification only"
      }
    },
    {
      "surface": "Stop",
      "status": "confirmed-not-exercisable",
      "reason": "did not fire in local codex exec on 2026-05-22"
    }
  ]
}
```

## This Sprint Does Not Close

- Codex runtime adapter promotion
- Gemini harness work
- cross-provider mapping decisions beyond documented candidates
- any Rust runtime or plugin implementation work

## Acceptance Criteria

- every locally tested Codex hook point has a repo-owned raw fixture
- every captured Codex hook point has an env snapshot fixture
- every approved fixture validates against a provider-specific model
- every approved Codex payload field and hook env var is enumerated in the
  approved fixtures or provider models
- `test-harness/hooks/codex/fixtures/approved/manifest.json` records
  `provider`, `codex_version`, `capture_date`, `capture_root`, and
  `hook_surfaces` by name
- every captured Codex manifest surface records `control_semantics` with
  `blocking`, `exit_code_contract`, and `stdio_contract`
- every audited Codex hook surface has an explicit manifest / findings-ledger
  disposition: `captured` or `confirmed-not-exercisable`
- before the first `N.1` harness commit,
  `docs/phase-N/codex-capture-checklist.md` classifies every planned Codex
  surface as either `direct hook surface (harness-capturable)` or
  `relay-synthetic (confirmed-not-exercisable)`; `SessionStart`, `resume`,
  `fork`, and `--cd` must be classified explicitly, with `SessionStart`
  retaining its relay source citation
- the `codex-capture-checklist.md` classification commit and the first `N.1`
  harness code commit are separate git commits; the classification commit must
  pre-date any harness code commit in the sprint branch
- the approved Codex manifest or `docs/phase-N/codex-findings-ledger.md`
  records a per-surface final disposition for `resume`, `fork`, and `--cd`
- `N.1` closure cannot rely on the existing debounce prototype alone; the
  approved manifest must enumerate more than the current prototype baseline or
  explicitly record every additional audited surface as
  `confirmed-not-exercisable`
- only `schook`-owned raw payload and hook-process environment fields are
  promoted into the approved field inventory
- every remaining field table in `docs/hook-api/codex-hook-api.md` is either
  backed by a repo-owned `N.1` fixture citation or explicitly labeled
  `relay-only planning evidence — not part of approved schook fixture
  inventory`
- Codex pytest schema-proof tests fail on fixture/model drift
- `docs/phase-N/codex-findings-ledger.md` is the only authoritative handoff
  ledger for Codex capture findings

## Required Validation

- `pytest test-harness/hooks/codex/tests/ -m provider_codex -v`
- `git log --oneline -- docs/phase-N/codex-capture-checklist.md test-harness/hooks/codex/`
  proves the checklist-classification commit predates the first harness code
  commit and is a separate commit, with explicit pre-harness entries for
  `resume`, `fork`, and `--cd`
- `git diff --check`
