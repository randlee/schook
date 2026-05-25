# Phase Q Plan

## Goal

Finish the first production-readiness follow-on after `Phase P` by adding a
real smoke-test surface for the currently supported providers and expanding the
permanent provider harness to cover the next documented providers:

- `Cursor Agent`
- `opencode`

`Phase Q` is not a new runtime-normalization phase. Its scope is:

- smoke-test infrastructure and executable smoke coverage for the current
  supported providers
- harness, fixture, model, and provider-doc expansion for `Cursor Agent` and
  `opencode`
- CI and `just` surface hardening for the new smoke path

`Phase Q` does **not** include:

- Cursor runtime parity work
- opencode runtime parity work
- reopening Claude/Codex/Gemini normalization design

## Baseline

- planning branch: `docs/phase-Q-planning`
- integration branch: `integrate/phase-Q`
- prerequisite baseline: `develop` after `Phase P` merge
- prerequisite status: current Claude/Codex/Gemini hook runtime parity and
  published `sc-lint` lint surface are already in place

Current known planning inputs:

- `HKR-007` still defers Cursor execution work; `Phase Q` starts by expanding
  Cursor harness/docs only
- no dedicated opencode requirement row exists yet in `docs/requirements.md`;
  `Phase Q` must treat that as a control-doc addition to land with the first
  opencode execution sprint, not as an implied undocumented scope
- `RULING-NEEDED-ECR-002` remains a live implementation-gap item that may
  affect smoke-test error-policy wording but does not block planning

## Integration Branch

Accepted `Phase Q` sprint outputs merge into:

- `integrate/phase-Q`

Rules:

- sprint branches do not write accepted rows directly into
  `docs/phase-Q/readiness.md`
- the integration author updates `docs/phase-Q/readiness.md` only after the
  accepted sprint output is merged into `integrate/phase-Q`

## Phase Entry Criteria

`Phase Q` execution may begin only when:

- `Phase P` is merged to `develop`
- `just test hooks claude`, `just test hooks codex`, and
  `just test hooks gemini` remain green on the accepted baseline
- the repo has one approved smoke entrypoint surface to own provider smoke
  execution rather than ad hoc shell commands
- Cursor/opencode work is treated as harness/doc-model expansion only unless a
  later phase explicitly authorizes runtime support

## Pre-Sprint Kickoff Checklist

Before any `Phase Q` sprint starts, the handoff or working notes must record:

- exact requirement IDs and implementation-gap IDs in scope
- the accepted baseline commit from `develop` or `integrate/phase-Q`
- the single owning implementation path for the smoke or harness behavior in
  scope
- the tests expected to fail before the sprint and pass after it
- the docs that must change in the same PR as the code
- the files, crates, and docs that define the sprint write scope

## Sprint Sequence

### Q.1 openshell Evaluation

Purpose:

- evaluate `openshell` as a smoke/harness execution environment for
  `sc-hooks-test`
- determine whether it should replace the current shell execution path,
  supplement it, or remain rejected
- freeze the recommendation before smoke infrastructure lands

Execution branch:
- `feature/pQ-s1-openshell-evaluation`

Execution worktree:
- `../schook-worktrees/feature/pQ-s1-openshell-evaluation`

Entry criteria:
- accepted post-`Phase P` baseline on `develop`

### Q.2 Smoke Infrastructure

Purpose:

- add the curated `just smoke` surface following the `../atm-core` pattern
- create the repo-owned smoke runner implementation and wire it into CI
- keep the smoke surface separate from `just test` and `just lint`

Execution branch:
- `feature/pQ-s2-smoke-infrastructure`

Execution worktree:
- `../schook-worktrees/feature/pQ-s2-smoke-infrastructure`

Entry criteria:
- accepted `Q.1` openshell recommendation

### Q.3 Claude Smoke Tests

Purpose:

- implement live executable smoke coverage for the Claude runtime path
- prove the installed runtime layout, plugin chain, and logging path still work
  end to end through the new `just smoke` surface

Execution branch:
- `feature/pQ-s3-claude-smoke`

Execution worktree:
- `../schook-worktrees/feature/pQ-s3-claude-smoke`

Entry criteria:
- accepted `Q.2` smoke infrastructure

### Q.4 Codex Smoke Tests

Purpose:

- implement live executable smoke coverage for the Codex runtime path
- cover the retained live Codex lifecycle behavior now owned by the shared
  runtime path

Execution branch:
- `feature/pQ-s4-codex-smoke`

Execution worktree:
- `../schook-worktrees/feature/pQ-s4-codex-smoke`

Entry criteria:
- accepted `Q.2` smoke infrastructure

### Q.5 Gemini Smoke Tests

Purpose:

- implement live executable smoke coverage for the Gemini runtime path
- cover the retained Gemini lifecycle behavior, including the shared stop-path
  handling for `AfterAgent`

Execution branch:
- `feature/pQ-s5-gemini-smoke`

Execution worktree:
- `../schook-worktrees/feature/pQ-s5-gemini-smoke`

Entry criteria:
- accepted `Q.2` smoke infrastructure

### Q.6 Cursor Agent Hook Harness

Purpose:

- turn `Cursor Agent` from a docs-only deferred provider into a maintained
  harness provider with approved fixtures, provider models, and harness tests
- use the existing `test-harness/hooks/cursor-agent/` ownership boundary rather
  than inventing a second competing `cursor/` tree

Execution branch:
- `feature/pQ-s6-cursor-harness`

Execution worktree:
- `../schook-worktrees/feature/pQ-s6-cursor-harness`

Entry criteria:
- accepted `Q.2` smoke infrastructure

### Q.7 Cursor Agent API Doc And Pydantic Models

Purpose:

- update `docs/hook-api/cursor-agent-hook-api.md` from planning reference to
  harness-backed provider artifact
- add or tighten Pydantic models for all retained Cursor surfaces

Execution branch:
- `feature/pQ-s7-cursor-doc-models`

Execution worktree:
- `../schook-worktrees/feature/pQ-s7-cursor-doc-models`

Entry criteria:
- accepted `Q.6` Cursor harness output

### Q.8 opencode Hook Harness

Purpose:

- add `opencode` as a maintained harness provider with approved fixtures,
  provider models, and harness tests
- add the missing control-doc ownership needed for opencode provider scope

Execution branch:
- `feature/pQ-s8-opencode-harness`

Execution worktree:
- `../schook-worktrees/feature/pQ-s8-opencode-harness`

Entry criteria:
- accepted `Q.2` smoke infrastructure

### Q.9 opencode API Doc And Pydantic Models

Purpose:

- create `docs/hook-api/opencode-agent-hook-api.md`
- add or tighten Pydantic models for all retained opencode surfaces
- close the provider-doc/model side of the new opencode harness support

Execution branch:
- `feature/pQ-s9-opencode-doc-models`

Execution worktree:
- `../schook-worktrees/feature/pQ-s9-opencode-doc-models`

Entry criteria:
- accepted `Q.8` opencode harness output

## Dependency Rules

- `Q.1` must land before `Q.2` because the smoke environment choice affects the
  runner contract and CI shape.
- `Q.2` must land before all later sprints because `Phase Q` standardizes one
  smoke entrypoint and one CI ownership path first.
- `Q.3`, `Q.4`, and `Q.5` all depend on `Q.2` and may run in parallel because
  they write disjoint provider smoke assets.
- `Q.6` and `Q.8` both depend on `Q.2` so the new providers enter the repo
  after the smoke/CI contract is frozen.
- `Q.7` depends on `Q.6`; `Q.9` depends on `Q.8`.
- no Cursor or opencode runtime implementation starts in `Phase Q`; harness and
  provider-doc/model work must close first.

## Sprint Artifact Summary

- `Q.1`:
  - `docs/phase-Q/openshell-evaluation.md`
- `Q.2`:
  - `justfile`
  - `.just/`
  - `.github/workflows/ci.yml`
  - smoke operator docs
- `Q.3`:
  - Claude smoke scripts, fixtures, and smoke docs
- `Q.4`:
  - Codex smoke scripts, fixtures, and smoke docs
- `Q.5`:
  - Gemini smoke scripts, fixtures, and smoke docs
- `Q.6`:
  - `test-harness/hooks/cursor-agent/`
  - `test_harness/hooks/cursor-agent/`
- `Q.7`:
  - `docs/hook-api/cursor-agent-hook-api.md`
  - Cursor provider models and tests
- `Q.8`:
  - `test-harness/hooks/opencode/`
  - `test_harness/hooks/opencode/`
  - control-doc additions for opencode provider scope
- `Q.9`:
  - `docs/hook-api/opencode-agent-hook-api.md`
  - opencode provider models and tests

## Phase Exit Criteria

`Phase Q` is ready for closeout when:

- `just smoke` exists, is documented, and is green in CI
- Claude/Codex/Gemini smoke coverage proves the live supported runtime path
- Cursor Agent harness, fixtures, provider models, and API doc are current
- opencode harness, fixtures, provider models, and API doc are current
- all new provider scope added in `Phase Q` is reflected in
  `docs/requirements.md`, `docs/traceability.md`, and `docs/project-plan.md`
- no Cursor or opencode runtime claims are overstated beyond harness/doc-model
  support
