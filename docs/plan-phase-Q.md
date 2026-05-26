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

- `HKR-007` now governs Cursor Agent harness/doc-model expansion only; Cursor
  runtime parity remains out of scope for `Phase Q`
- `HKR-018` governs the new opencode harness/doc-model expansion only;
  opencode runtime parity remains out of scope for `Phase Q`
- `HKR-019` governs the new curated smoke surface and CI-owned smoke gate
- `ADR-SHK-010` freezes the `Phase Q` rule that smoke and next-provider
  harness work do not authorize new runtime parity claims

Carry-forward note:

- `RULING-NEEDED-ECR-002` remains active in `docs/implementation-gaps.md`, but
  `Phase Q` does not own that ruling
- rationale: `Phase Q` adds smoke infrastructure plus Cursor/opencode
  harness/doc-model scope only; it does not change the public cross-crate
  error-type layout or backtrace policy where `ECR-002` would close
- disposition: keep `ECR-002` out of the primary Phase Q driver set and carry
  it forward to a later error-surface phase instead of implying closure inside
  a smoke or provider-doc sprint

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
- the smoke surface and Cursor/opencode harness follow-on are treated as the
  only owning paths for `Phase Q`; no provider-specific bypass around those
  contracts is allowed
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
- keep the smoke runner on the existing repo-owned `just` plus Python path;
  `Q.1` rejects `openshell` as a new required execution dependency for this
  phase
- freeze one explicit smoke execution model:
  - CI runs offline replay/dry-run smoke without provider CLIs
  - accepted-baseline provider records in later sprints come from live local
    provider invocations

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
  harness provider with approved fixtures and harness tests
- use the existing `test-harness/hooks/cursor-agent/` ownership boundary rather
  than inventing a second competing `cursor/` tree
- create the matching `test_harness/hooks/cursor_agent/` Python package root
  that later model work extends

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
  harness tests, and control-doc ownership
- add the missing control-doc ownership needed for opencode provider scope
- extend the shared harness README and shared control-doc baseline after `Q.6`
  rather than competing for those files in parallel
- create the matching `test_harness/hooks/opencode/` Python package root that
  later model work extends

Execution branch:
- `feature/pQ-s8-opencode-harness`

Execution worktree:
- `../schook-worktrees/feature/pQ-s8-opencode-harness`

Entry criteria:
- accepted `Q.6` Cursor harness output

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
  `Q.2` owns the shared `.just/run_smoke.py` dispatcher and later smoke
  sprints add only provider modules, replay fixtures, and provider smoke
  records.
- `Q.6` depends on `Q.2` so Cursor enters the repo after the smoke/CI
  contract is frozen.
- `Q.8` depends on `Q.6` so opencode extends the shared harness README and
  shared control-doc baseline sequentially instead of competing for them in a
  parallel branch.
- `Q.7` depends on `Q.6`; `Q.9` depends on `Q.8`.
- no Cursor or opencode runtime implementation starts in `Phase Q`; harness and
  provider-doc/model work must close first.

## Sprint Artifact Summary

- `Q.1`:
  - `docs/plan-phase-Q.md`
  - `docs/phase-Q/openshell-evaluation.md`
- `Q.2`:
  - `justfile`
  - `.just/run_smoke.py`
  - `.just/smoke/`
  - `.github/workflows/ci.yml`
  - `docs/phase-Q/smoke-surface.md`
- `Q.3`:
  - `.just/smoke/claude.py`
  - `docs/phase-Q/smoke-claude.md`
- `Q.4`:
  - `.just/smoke/codex.py`
  - `docs/phase-Q/smoke-codex.md`
- `Q.5`:
  - `.just/smoke/gemini.py`
  - `docs/phase-Q/smoke-gemini.md`
- `Q.6`:
  - `test-harness/hooks/cursor-agent/fixtures/`
  - `test-harness/hooks/cursor-agent/hooks/`
  - `test-harness/hooks/cursor-agent/schema/`
  - `test-harness/hooks/cursor-agent/tests/test_harness_structure.py`
  - `test-harness/hooks/cursor-agent/tests/test_fixture_validation.py`
  - `test-harness/hooks/README.md`
  - `docs/requirements.md`
  - `docs/traceability.md`
- `Q.7`:
  - `docs/hook-api/cursor-agent-hook-api.md`
  - `test_harness/hooks/cursor_agent/models/payloads.py`
  - `test_harness/hooks/cursor_agent/models/registry.py`
  - `test-harness/hooks/cursor-agent/tests/test_payload_models.py`
- `Q.8`:
  - `test-harness/hooks/opencode/fixtures/`
  - `test-harness/hooks/opencode/hooks/`
  - `test-harness/hooks/opencode/schema/`
  - `test-harness/hooks/opencode/tests/test_harness_structure.py`
  - `test-harness/hooks/opencode/tests/test_fixture_validation.py`
  - `docs/requirements.md`
  - `docs/traceability.md`
- `Q.9`:
  - `docs/hook-api/opencode-agent-hook-api.md`
  - `test_harness/hooks/opencode/models/payloads.py`
  - `test_harness/hooks/opencode/models/registry.py`
  - `test-harness/hooks/opencode/tests/test_payload_models.py`

## Phase Exit Criteria

`Phase Q` is ready for closeout when:

- `just smoke` exists, is documented, and is green in CI
- Claude/Codex/Gemini smoke coverage proves the live supported runtime path
- Cursor Agent harness, fixtures, provider models, and API doc are current
- opencode harness, fixtures, provider models, and API doc are current
- all new provider scope added in `Phase Q` is reflected in
  `docs/requirements.md`, `docs/traceability.md`, and `docs/project-plan.md`
- the smoke ownership path is reflected in `docs/architecture.md` and no
  smoke/provider-harness artifact is described as runtime parity by implication
- no Cursor or opencode runtime claims are overstated beyond harness/doc-model
  support
