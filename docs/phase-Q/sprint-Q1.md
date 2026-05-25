---
id: Q.1
title: openshell Evaluation
status: planned
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
