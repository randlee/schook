---
id: Q.2
title: Smoke Infrastructure
status: planned
branch: feature/pQ-s2-smoke-infrastructure
worktree: ../schook-worktrees/feature/pQ-s2-smoke-infrastructure
target: integrate/phase-Q
---

# Sprint Q.2 — Smoke Infrastructure

## Goal

- add the curated `just smoke` surface following the `../atm-core` pattern
- create the repo-owned smoke runner and CI ownership path
- keep smoke execution separate from both `just test` and `just lint`

## Hard Dependencies

- accepted `Q.1` openshell recommendation

## Entry Criteria

- accepted `Q.1` openshell recommendation

## Exact Targets

- `justfile`
- `.just/run_smoke.py`
- `.just/smoke/`
- `.github/workflows/ci.yml`
- `docs/phase-Q/smoke-surface.md`

## Deliverables

- `just smoke` public entrypoint
- repo-owned smoke runner implementation under `.just/`
- CI job or CI extension for smoke execution
- operator docs for the smoke surface

## Required Contract Samples

Expected curated smoke entrypoint shape:

```just
smoke provider='all':
    @{{python_cmd}} .just/run_smoke.py {{provider}}
```

The landed `justfile` may include private helper recipes, but `Q.2` must keep
one public `just smoke` surface that remains separate from `just test` and
`just lint`.

## Acceptance Criteria

- `just smoke` exists and is documented
- the implementation pattern follows the curated `../atm-core` style
- CI owns the smoke surface explicitly
- the repo has one smoke ownership path under `.just/`; ad hoc shell-command
  smoke execution is not left as a parallel supported surface
- provider-specific live proof remains deferred to `Q.3`, `Q.4`, and `Q.5`
  rather than being implied complete by `Q.2` alone

## Out Of Scope

- provider-specific smoke scenarios beyond the runner contract
- Cursor Agent harness work
- opencode harness work

## Required Validation

- `just help`
- `just smoke`
- `cargo test --workspace`
- `git diff --check`

## Sprint QA Checklist

- Which requirement IDs or gap IDs changed status?
- What competing smoke path was removed or rejected early?
- Which files or docs are the owned write scope for the sprint?
- What validation proves the public `just smoke` surface and CI gate?
- What follow-on work is unblocked for `Q.3` through `Q.9`?
