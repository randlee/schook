---
id: Q.1
title: openshell Evaluation
status: completed
branch: feature/pQ-s1-openshell-evaluation
worktree: ../schook-worktrees/feature/pQ-s1-openshell-evaluation
target: integrate/phase-Q
---

# Sprint Q.1 — openshell Evaluation

## Goal

- assess whether `openshell` should replace, supplement, or stay out of the
  current shell environment used by `sc-hooks-test` and the planned smoke
  runner
- freeze one recommendation before smoke infrastructure work starts

## Hard Dependencies

- accepted post-`Phase P` baseline on `develop`

## Entry Criteria

- accepted post-`Phase P` baseline on `develop`

## Exact Targets

- `docs/phase-Q/openshell-evaluation.md`
- `docs/plan-phase-Q.md`

## Deliverables

- one documented recommendation for `openshell`:
  - replace
  - supplement
  - reject for now
- explicit rationale, tradeoffs, and follow-on implications for `Q.2`

## Required Work

- evaluate `openshell` specifically against the planned smoke runner and
  `sc-hooks-test` execution needs
- record one explicit recommendation and the concrete implication for `Q.2`
- update the phase plan only as needed to carry that recommendation forward

## Required Contract Samples

Required recommendation shape:

```text
recommendation: replace | supplement | reject_for_now
smoke_runner_impact: <one paragraph>
ci_impact: <one paragraph>
follow_on_for_q2: <one literal next step list>
```

## Acceptance Criteria

- `docs/phase-Q/openshell-evaluation.md` exists
- the recommendation is explicit and justified
- `Q.2` can proceed without re-litigating the shell-environment choice

## Out Of Scope

- implementing the smoke runner
- changing `justfile`
- CI smoke execution

## Required Validation

- `git diff --check`

## Sprint QA Checklist

- Which requirement IDs or gap IDs changed status?
- What code or path was explicitly rejected rather than left ambiguous?
- Which files or docs are the owned write scope for the sprint?
- What validation proves the recommendation is ready for `Q.2` to consume?
- What follow-on work is unblocked by the recommendation?
- Does `docs/phase-Q/openshell-evaluation.md` contain all four required
  fields: `recommendation`, `smoke_runner_impact`, `ci_impact`,
  `follow_on_for_q2`?

## Sprint QA Checklist Answers

- No requirement IDs changed status in `Q.1`; this sprint only froze the
  execution recommendation that `Q.2` consumes.
- The explicitly rejected path was introducing `openshell` as either a
  replacement or supplement for the smoke runner. `Q.2` proceeds on the
  documented `reject_for_now` basis instead of keeping the shell choice open.
- Owned write scope:
  - `docs/phase-Q/openshell-evaluation.md`
  - `docs/phase-Q/sprint-Q1.md`
  - `docs/plan-phase-Q.md`
- Validation proof:
  - `git diff --check` PASS
  - the recommendation contract is fully present in
    `docs/phase-Q/openshell-evaluation.md`, so `Q.2` can consume it directly
    without revisiting the execution-model choice
- Follow-on work unblocked:
  - `Q.2` may proceed immediately on the repo-owned `just` + Python smoke
    dispatcher path
  - offline CI smoke fixtures remain under `.just/smoke/fixtures/`
  - no `openshell` dependency is required for the `Phase Q` smoke baseline
- Yes. `docs/phase-Q/openshell-evaluation.md` now has the required
  `recommendation`, `smoke_runner_impact`, `ci_impact`, and
  `follow_on_for_q2` fields, plus the required title frontmatter.
