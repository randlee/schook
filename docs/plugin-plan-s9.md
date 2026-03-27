# Sprint 9 Plugin Hook Plan

## Goal

Define the executable plan for the Claude-first hook work so implementation
starts only after the hook payload contract has been captured and validated.

This sprint is planning-only. It does not authorize hook runtime code.

## Scope

This document is the umbrella execution plan for Sprint 9.

It must be sufficient for `arch-hook` to execute the work in the correct order
without asking what comes first.

This plan covers:

1. harness build
2. live Claude payload capture
3. Pydantic model and schema creation
4. plan revision from verified schema
5. only then hook implementation evaluation and sequencing

Supporting documents remain source-of-truth references for platform facts,
requirements, architecture boundaries, and harness contract details. They do
not define a competing execution sequence.

## Source Documents

- `docs/hook-api/claude-hook-api.md`
  - verified Claude hook baseline and current known payload facts
- `docs/hook-api/atm-hook-extension.md`
  - ATM-specific behavior that remains separate from the generic hook contract
- `docs/hook-api/codex-hook-api.md`
  - documented provider reference only; not part of the first implementation path
- `docs/hook-api/cursor-agent-hook-api.md`
  - documented provider reference only; not part of the first implementation path
- `test-harness/hooks/README.md`
  - planned harness contract file created in Phase 1; it will own directory ownership, fixture policy, report lifecycle, and the `pytest` split
- `docs/requirements.md`
  - hook requirements, especially `HKR-001` through `HKR-007`
- `docs/architecture.md`
  - crate boundaries and inventory rules
- `docs/project-plan.md`
  - project-level sequencing for the hook track

## Current Planning Baseline

- Claude is the only implementation baseline for the first pass.
- ATM behavior remains an extension of the Claude baseline and stays documented
  separately in `docs/hook-api/atm-hook-extension.md`.
- Codex, Gemini, and Cursor remain documented follow-on providers, not current
  implementation targets.
- Current ATM source-of-truth comes from `agent-team-mail` docs, scripts,
  tests, and Rust fallback code.
- Current Sprint 9 prototype crates are reference-only:
  - `plugins/atm-session-lifecycle`
  - `plugins/atm-bash-identity`
  - `plugins/gate-agent-spawns`
  - `plugins/atm-state-relay`
- Those prototype crates are not authoritative until re-evaluated against
  captured Claude payloads and validated schema artifacts.

## Non-Negotiable Sequence

The work must happen in this order:

1. build the payload capture harness
2. capture real Claude hook payloads
3. create Pydantic models and schema artifacts from captured payloads
4. revise the plan from verified schema
5. re-evaluate or implement each hook plugin against validated schema

No later step may begin early because a prototype branch already exists.

## Implementation Start Rule

No hook runtime code starts until all of the following are true:

- this plan is reviewed and accepted
- Phase 1 has created `test-harness/hooks/README.md` and reviewers have accepted it as the harness contract
- Claude payloads for the required first-pass capture set are captured
- the captured payloads validate against provider-specific Pydantic models
- schema artifacts are generated from those validated models
- this plan is revised from the captured evidence

If any one of those conditions is missing, Sprint 9 remains in planning or
schema-capture mode.

## Required Claude Capture Set

The first pass must capture real payloads for these Claude hook surfaces:

- `SessionStart`
- `SessionEnd`
- `PreToolUse(Bash)`
- `PostToolUse(Bash)`
- `PreToolUse(Task)`
- `PermissionRequest`
- `Stop`
- `Notification(idle_prompt)`

The plan may not promote additional Claude fields or behaviors into
implementation scope unless they are backed by:

- current source-of-truth docs/scripts/tests, or
- captured harness fixtures

## Harness Output Contract

Each live capture run must produce:

- raw payload captures
- normalized comparison artifacts
- provider-specific Pydantic model validation
- generated schema artifacts
- `validation-summary.json`
- `drift-report.json`
- `run-report.md`

Recommended per-run layout:

```text
test-harness/hooks/<provider>/captures/<run-id>/
  raw/
  normalized/

test-harness/hooks/<provider>/reports/<run-id>/
  validation-summary.json
  drift-report.json
  run-report.md
```

Rules:

- `pytest` is the required test runner
- default `pytest` validates fixtures/models/schema only
- `pytest -m live_capture` performs provider launch and capture
- canned prompts are required for repeatable runs
- local Python capture hooks are the required capture mechanism
- no manual copying or hand-moving of artifacts is part of the workflow

## Phase Plan

### Phase 0: Review Baseline

Purpose:

- freeze the planning baseline before harness work starts

Gate to start:

- Sprint 9 planning docs are the active work
- no hook runtime implementation is in scope

Deliverables:

- this document accepted as the umbrella execution plan
- `docs/hook-api/claude-hook-api.md` accepted as the current Claude fact baseline
- `docs/hook-api/atm-hook-extension.md` accepted as the ATM-only extension layer

Done when:

- reviewers can read the docs and state the same build order
- Claude-first scope is explicit
- prototype crates are explicitly marked reference-only

### Phase 1: Build The Harness

Purpose:

- create the test-harness structure and execution path required for live Claude capture

Gate to start:

- Phase 0 complete

Deliverables:

- `test-harness/hooks/README.md` created as the harness contract file
- `test-harness/hooks/` directory structure in place
- Claude provider harness subdirectories in place:
  - `prompts/`
  - `hooks/`
  - `models/`
  - `schema/`
  - `fixtures/`
  - `captures/`
  - `reports/`
  - `scripts/`
  - `tests/`
- harness `pytest` entry points implemented
- canned Claude prompts added for the required first-pass capture set
- local Python capture hooks implemented for harness-only payload capture
- automated report generation wired into the harness run

Not part of this phase:

- hook runtime plugin code
- Codex/Gemini/Cursor implementation work
- promotion of unverified payload fields into implementation docs

Done when:

- `pytest` runs fixture/model/schema tests successfully
- `pytest -m live_capture` has a defined Claude execution path
- harness output locations are automatic and documented

### Phase 2: Capture Real Claude Payloads

Purpose:

- collect the actual Claude hook payloads the later implementation must obey

Gate to start:

- Phase 1 complete

Deliverables:

- real Claude captures for the required first-pass hook set
- raw payload evidence stored by hook type
- normalized artifacts suitable for comparison
- a formal run report for the capture session

Not part of this phase:

- hand-authored schema guesses
- hook plugin implementation

Done when:

- each required Claude hook surface has at least one captured payload fixture
- run artifacts exist under the harness output contract
- the capture run produces a complete `run-report.md`

### Phase 3: Create Pydantic Models And Schema Artifacts

Purpose:

- convert captured evidence into explicit validated contracts

Gate to start:

- Phase 2 complete with captured Claude fixtures

Deliverables:

- provider-specific Pydantic models for the captured Claude payloads
- generated schema artifacts derived from those models
- fixture validation tests for the captured payloads
- drift classification logic:
  - required field removed => fail
  - required field type changed => fail
  - new field added => report for review
  - optional field removed => report for review unless implementation relies on it

Rules:

- models start minimally
- required known fields are strict
- unknown extra fields may be allowed early, but they must be surfaced in reports
- no cross-provider shared schema is assumed

Done when:

- captured fixtures validate against the Claude Pydantic models
- schema artifacts exist for the validated Claude models
- drift policy is executable in tests, not just described in prose

### Phase 4: Revise The Plan From Verified Schema

Purpose:

- remove all remaining pre-capture assumptions before any hook code is accepted

Gate to start:

- Phase 3 complete

Deliverables:

- revised `docs/plugin-plan-s9.md`
- revised `docs/hook-api/claude-hook-api.md` where captured evidence clarifies fields
- explicit classification for each planned implementation dependency:
  - verified by source docs/scripts/tests
  - verified by captured fixture
  - deferred because still not verified
- implementation sequence updated only from captured evidence

Required review questions:

- which fields are merely observed?
- which fields are stable enough to implement against?
- which prior prototype assumptions were wrong, incomplete, or still unverified?

Done when:

- implementation-facing docs rely only on verified fields
- unknowns are called out explicitly as gaps or deferrals
- reviewers can point to fixture or source evidence for every required field used by the next implementation phase

### Phase 5: Re-Evaluate And Sequence Implementation

Purpose:

- begin hook runtime work only after the contract is verified

Gate to start:

- Phase 4 complete

Deliverables for the phase-start decision:

- explicit disposition for each prototype crate:
  - keep as-is and continue
  - refactor to match verified schema
  - replace entirely
- an implementation sequence for Claude ATM hooks derived from verified schema

Only after that disposition is complete may runtime implementation work begin.

The implementation track after this phase must follow the Hook Phase 3/4/5
split from `docs/project-plan.md`:

#### Hook Phase 3: Session Foundation

Depends on:

- Phase 4 complete

Deliverables:

- session lifecycle implementation
- persisted session correlation keyed by verified inputs only

#### Hook Phase 4: Bash Identity And Spawn Gates

Depends on:

- Hook Phase 3 complete

Deliverables:

- Bash ATM identity behavior
- Task spawn gating behavior

#### Hook Phase 5: Relay Hooks

Depends on:

- Hook Phase 3 complete

Deliverables:

- relay behavior for `Notification(idle_prompt)`, `PermissionRequest`, and `Stop`

Rules:

- prototype branches remain reference-only until this phase completes the re-evaluation
- no field may be consumed in runtime code unless it is backed by the verified Phase 4 outputs
- architecture inventory updates must happen in the same PR as any accepted runtime crate introduction

Done when:

- implementation tasks are derived from verified schema rather than inferred payload shapes
- the runtime work queue is ready to start without guessing

## Immediate Work Package

The immediate Sprint 9 work is limited to Phases 0 through 4.

That means the current task is:

- make the harness build-and-capture sequence explicit
- make the schema/model phase explicit
- make the post-capture plan revision gate explicit
- remove any wording that makes plugin crates sound like the current build task

The current task is not:

- building hook runtime crates
- widening scope to Codex, Gemini, or Cursor implementation
- treating the prototype crates as the accepted implementation baseline

## Provider Scope

Current provider posture:

- Claude: active baseline
- ATM: extension layer on top of the Claude baseline
- Codex: documented only
- Gemini: documented only
- Cursor: documented only

Provider expansion beyond Claude must wait until the Claude baseline is:

- captured
- modeled
- schematized
- reviewed
- revised in plan form

## Review Checklist

This plan is ready for review when all of the following are true:

- the first executable step is harness build, not plugin implementation
- the required Claude capture set is explicit
- harness outputs are explicit
- model/schema creation is a separate gated phase
- post-capture plan revision is mandatory
- prototype crates are described as reference-only
- implementation is clearly downstream of the schema gates
- other providers are documented but not blocking the first Claude path

## Acceptance For This Plan

This plan passes review when `arch-hook` can read it and answer these questions
without follow-up:

1. What do I build first?
2. Which Claude payloads must I capture?
3. What artifacts must the harness produce?
4. When do Pydantic models and schema artifacts get created?
5. What must be true before any hook runtime code is accepted?
6. Are the existing prototype crates authoritative?

The required answers are:

1. build the Claude harness first
2. capture the eight Claude hook surfaces listed above
3. raw captures, normalized artifacts, validation summary, drift report, and run report
4. after real payload capture, before any implementation work
5. reviewed plan + accepted harness contract + captured payloads + validated models + schema artifacts + revised plan
6. no, they are reference-only until re-evaluated against validated schema
