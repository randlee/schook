---
id: Q.1
title: openshell Evaluation
status: planned
branch: feature/pQ-s1-openshell-evaluation
worktree: ../schook-worktrees/feature/pQ-s1-openshell-evaluation
target: integrate/phase-Q
---

# Sprint Q.1 — openshell Evaluation

## Purpose

- assess whether `openshell` should replace, supplement, or stay out of the
  current shell environment used by `sc-hooks-test` and the planned smoke
  runner
- freeze one recommendation before smoke infrastructure work starts

## Entry Criteria

- accepted post-`Phase P` baseline on `develop`

## Exact Targets

- `docs/phase-Q/openshell-evaluation.md`
- any supporting control-doc updates needed to record the recommendation

## Deliverables

- one documented recommendation for `openshell`:
  - replace
  - supplement
  - reject for now
- explicit rationale, tradeoffs, and follow-on implications for `Q.2`

## Acceptance Criteria

- `docs/phase-Q/openshell-evaluation.md` exists
- the recommendation is explicit and justified
- `Q.2` can proceed without re-litigating the shell-environment choice

## Required Validation

- `git diff --check`
