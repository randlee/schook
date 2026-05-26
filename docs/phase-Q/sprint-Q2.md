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
- `.just/smoke/fixtures/`
- `.github/workflows/ci.yml`
- `docs/phase-Q/smoke-surface.md`

## Deliverables

- `just smoke` public entrypoint
- repo-owned smoke runner implementation under `.just/`
- CI job or CI extension for smoke execution
- operator docs for the smoke surface
- one explicit CI-versus-live smoke execution model

## Required Work

- add the public `just smoke` surface without changing `just test` or
  `just lint`
- land the shared smoke runner under `.just/`
- make `.just/run_smoke.py` the sole owner of provider discovery/dispatch so
  later provider smoke sprints add provider modules without reopening the
  shared dispatcher contract
- add the repo-owned offline replay or dry-run storage path under
  `.just/smoke/fixtures/`
- wire one CI-owned smoke gate
- document the smoke surface for operators

## Required Contract Samples

Expected curated smoke entrypoint shape:

```just
smoke provider='all' mode='live':
    @{{python_cmd}} .just/run_smoke.py {{provider}} --mode {{mode}}
```

The landed `justfile` may include private helper recipes, but `Q.2` must keep
one public `just smoke` surface that remains separate from `just test` and
`just lint`.

## CI Execution Model

- generic CI for `Q.2` runs smoke in offline mode only; it does not require
  Claude, Codex, or Gemini CLIs to be installed
- the CI-owned offline gate uses repo-owned replay or dry-run assets under
  `.just/smoke/fixtures/`
- provider-specific live smoke remains a later sprint concern:
  - `Q.3` records one live Claude accepted-baseline run
  - `Q.4` records one live Codex accepted-baseline run
  - `Q.5` records one live Gemini accepted-baseline run
- the offline gate proves the public smoke surface, provider discovery, and
  CI ownership path without overstating provider-runtime proof before the
  provider-specific smoke sprints land

## Acceptance Criteria

- `just smoke` exists and is documented
- the implementation pattern follows the curated `../atm-core` style
- CI owns the smoke surface explicitly
- `just smoke all ci` is the explicit CI-owned offline gate and is documented
  as not requiring provider CLIs
- the repo has one smoke ownership path under `.just/`; ad hoc shell-command
  smoke execution is not left as a parallel supported surface
- when no provider smoke modules are present yet, `just smoke all ci` still
  exits successfully and reports the deterministic zero-provider result
- provider-specific live proof remains deferred to `Q.3`, `Q.4`, and `Q.5`
  rather than being implied complete by `Q.2` alone

## Out Of Scope

- provider-specific smoke scenarios beyond the runner contract
- Cursor Agent harness work
- opencode harness work

## Required Validation

- `just help`
- `just smoke all ci`
- `cargo test --workspace`
- `git diff --check`

## Sprint QA Checklist

- Which requirement IDs or gap IDs changed status?
- What competing smoke path was removed or rejected early?
- Which files or docs are the owned write scope for the sprint?
- What validation proves the public `just smoke` surface and CI gate?
- What follow-on work is unblocked for `Q.3` through `Q.9`?
