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
- `docs/phase-bc-hook-runtime-design.md`
  - clean post-capture runtime design authority for state, logging, trait
    boundaries, and crate split

## Current Planning Baseline

- Claude is the only implementation baseline for the first pass.
- ATM behavior remains an extension of the Claude baseline and stays documented
  separately in `docs/hook-api/atm-hook-extension.md`.
- Codex, Gemini, and Cursor remain documented follow-on providers, not current
  implementation targets.
- Current ATM behavior may be referenced from `agent-team-mail` docs, scripts,
  tests, and Rust fallback code, but those materials are reference-only for the
  clean redesign.
- The clean post-capture runtime design authority lives in this repo, not in
  `agent-team-mail`.
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

The first pass capture set is now locally evidenced for these Claude hook
surfaces:

- `SessionStart`
- `SessionEnd`
- `PreToolUse(Bash)`
- `PostToolUse(Bash)`
- `PreToolUse(Agent)`
- `PermissionRequest`
- `Stop`
- `Notification(idle_prompt)`

The runtime plan may not promote additional Claude fields or behaviors into
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

### S9-P0: Phase 0: Review Baseline

Status:
- Completed

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

### S9-P1: Phase 1: Build The Harness

Status:
- Completed

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

### S9-P2: Phase 2: Capture Real Claude Payloads

Status:
- Completed

Purpose:

- collect the actual Claude hook payloads the later implementation must obey

Gate to start:

- Phase 1 complete

Note:
- completed as part of the executed harness pass; captured artifacts now live under `test-harness/hooks/claude/captures/raw/`

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

### S9-P3: Phase 3: Create Pydantic Models And Schema Artifacts

Status:
- Not started

Purpose:

- convert captured evidence into explicit validated contracts

Gate to start:

- Phase 2 complete with captured Claude fixtures

Deliverables:

- provider-specific Pydantic models for the captured Claude payloads
- `pyproject.toml` with `pydantic>=2.0` and `pytest` declared
- complete Pydantic discriminated-union model for Claude hook payloads
- generated schema artifacts derived from those models
- `test-harness/hooks/run-schema-drift.py` as the single schema-drift entry point
- per-provider adapters under `test-harness/hooks/<provider>/`
- fixture validation tests for the captured payloads
- single self-contained HTML report per run
- `.claude/skills/hook-schema-drift/` slash command definition
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

#### Pydantic Model Design

Phase 3 must include a complete Claude payload model design in Python so the
drift tooling and fixture validation path share the same contract:

```python
from __future__ import annotations

from typing import Annotated, Any, Literal, Optional, Union

from pydantic import BaseModel, Field, model_validator


class BashToolInput(BaseModel):
    command: str


class AgentToolInput(BaseModel):
    prompt: str
    subagent_type: Optional[str] = None
    name: Optional[str] = None
    team_name: Optional[str] = None


class BashToolResponse(BaseModel):
    output: Optional[str] = None
    error: Optional[str] = None
    interrupted: bool = False


class HookPayloadBase(BaseModel):
    session_id: str
    hook_event_name: str
    cwd: str
    transcript_path: Optional[str] = None


class SessionStartPayload(HookPayloadBase):
    hook_event_name: Literal["SessionStart"]
    source: str


class SessionEndPayload(HookPayloadBase):
    hook_event_name: Literal["SessionEnd"]


class PreCompactPayload(HookPayloadBase):
    hook_event_name: Literal["PreCompact"]
    trigger: Optional[str] = None
    custom_instructions: Optional[str] = None


class PreToolUseBashPayload(HookPayloadBase):
    hook_event_name: Literal["PreToolUse"]
    tool_name: Literal["Bash"]
    tool_input: BashToolInput


class PreToolUseAgentPayload(HookPayloadBase):
    hook_event_name: Literal["PreToolUse"]
    tool_name: Literal["Agent"]
    tool_input: AgentToolInput


class PostToolUseBashPayload(HookPayloadBase):
    hook_event_name: Literal["PostToolUse"]
    tool_name: Literal["Bash"]
    tool_input: BashToolInput
    tool_response: BashToolResponse


class PermissionRequestPayload(HookPayloadBase):
    hook_event_name: Literal["PermissionRequest"]
    tool_name: str
    tool_input: dict[str, Any]


class StopPayload(HookPayloadBase):
    hook_event_name: Literal["Stop"]
    stop_hook_active: bool = False


class NotificationPayload(HookPayloadBase):
    hook_event_name: Literal["Notification"]
    # Deferred: no verified payload shape yet.


PrimaryClaudeHookPayload = Annotated[
    Union[
        SessionStartPayload,
        SessionEndPayload,
        PreCompactPayload,
        PostToolUseBashPayload,
        PermissionRequestPayload,
        StopPayload,
        NotificationPayload,
    ],
    Field(discriminator="hook_event_name"),
]

PreToolUsePayload = Annotated[
    Union[PreToolUseBashPayload, PreToolUseAgentPayload],
    Field(discriminator="tool_name"),
]


class ClaudeHookEnvelope(BaseModel):
    payload: Union[PrimaryClaudeHookPayload, PreToolUsePayload]

    @model_validator(mode="before")
    @classmethod
    def dispatch_pre_tool_use(cls, value: Any) -> dict[str, Any]:
        if not isinstance(value, dict):
            raise TypeError("Claude hook payload must be a mapping")

        if value.get("hook_event_name") == "PreToolUse":
            tool_name = value.get("tool_name")
            if tool_name not in {"Bash", "Agent"}:
                raise ValueError(f"Unsupported PreToolUse tool_name: {tool_name!r}")

        return {"payload": value}


ClaudeHookPayload = ClaudeHookEnvelope
```

Model notes:

- `SessionStartPayload.source` remains `str`, not a `Literal`, because
  `compact` and `clear` should not be frozen into code as enum-only assumptions
  until the multi-provider drift tooling is in place.
- `PreToolUse` and `PostToolUse` require a second discriminator on `tool_name`.
- `NotificationPayload` remains deferred because live Haiku capture has not
  produced a verified payload shape.

Done when:

- captured fixtures validate against the Claude Pydantic models
- schema artifacts exist for the validated Claude models
- drift policy is executable in tests, not just described in prose

### S9-P4: Phase 4: Revise The Plan From Verified Schema

Status:
- Not started

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

### S9-P5: Phase 5: Re-Evaluate And Sequence Implementation

Status:
- Not started

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

#### S9-HP3: Hook Phase 3: Session Foundation

Status:
- Planned

Depends on:

- Phase 4 complete

Deliverables:

- session lifecycle implementation
- persisted session correlation keyed by verified inputs only

#### S9-HP4: Hook Phase 4: Bash Identity + Spawn Gates

Status:
- Planned

Depends on:

- Hook Phase 3 complete

Deliverables:

- Bash ATM identity behavior
- spawn gating behavior

#### S9-HP5: Hook Phase 5: Relay Hooks

Status:
- Planned

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

S9-PBC is tracked separately as:

- `S9-PBC` — Plan-BC: BC Design Consolidation
- current status: In QA
- current authority: `docs/phase-bc-hook-runtime-design.md`

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

## Schema Drift Detection Tooling

### Purpose

Phase 3 must add a single manual CLI entry point for detecting provider hook
payload drift after provider upgrades.

This tooling is not part of CI. It is a manual investigation tool that uses a
single provider argument and a shared reporting flow.

### Entry Point

The required entry point is:

```text
test-harness/hooks/run-schema-drift.py <provider>
```

Supported providers:

- `claude`
- `codex`
- `gemini`
- `cursor`
- `opencode`

### Slash Command

Phase 3 must also define a slash-command skill at:

```text
.claude/skills/hook-schema-drift/
```

Required command:

```text
/hook-schema-drift <provider>
```

Required flow:

1. invoke `run-schema-drift.py <provider>`
2. on completion, spawn `html-report-expert` as a background agent with
   `run_in_background=true`
3. the background agent reads the drift JSON and generates the annotated HTML report
4. the calling agent receives the report path and displays it to the user

The `html-report-expert` spawn must stay in the background so the calling agent
does not accumulate HTML-generation context.

### Provider States

#### Supported + CLI available

The tooling must:

- run the automatable capture sequence for that provider
- parse fresh captures through the provider Pydantic models
- compare fresh captures against approved fixtures as the old schema baseline
- emit drift JSON covering:
  - added fields
  - removed required fields
  - type changes
- include a note for non-automatable hooks using their last-known fixture

#### Supported + CLI not available

The tooling must:

- skip live capture
- emit the error `Agent CLI not available on this machine`
- list all hook types from the last approved fixture set
- label those hook entries as `STALE` with the fixture date

#### Not supported

The tooling must:

- emit the error `Provider not yet supported`
- include a subsection listing the work required to add support:
  - provider CLI dependency
  - capture script
  - Pydantic models
  - approved fixtures
  - adapter registration in `run-schema-drift.py`

### HTML Report Structure

Each run must produce one self-contained HTML report:

- no external CSS
- no iframes
- full `DOCTYPE`
- charset declaration required

Required output path:

```text
test-harness/hooks/<provider>/reports/<ISO-timestamp>/schema-drift-report.html
```

Required report sections:

1. Header
   - provider
   - run timestamp
   - overall status: `PASS`, `DRIFT`, `ERROR`, or `NOT_SUPPORTED`
2. Summary table
   - hook type
   - old status
   - new status
   - verdict
3. One per-hook `<section>`
   - OLD SCHEMA field table: name, type, required, evidence source
   - NEW SCHEMA field table if fresh capture exists
   - `No fresh capture — last known: <date>` when no fresh capture exists
   - DRIFT callout if schemas differ
   - ANALYSIS block below the diff, supplied by `html-report-expert`
4. Non-automatable hooks subsection
   - `SessionStart(source=compact)` with last-known fixture and manual procedure
   - `SessionStart(source=clear)` with last-known fixture and manual procedure
5. Drift action section
   - summary of all detected changes
   - recommended action:
     `Create a schook issue — do not divert current project to fix schema`
   - no auto-fix
   - no auto-model rewrite

### Visual Conventions

The generated HTML must follow the `xhtml-plugin-expert` visual conventions:

- all CSS inline
- `DOCTYPE` and charset required
- green (`#065f46` header / `#6ee7b7` border) for PASS / no change
- amber (`#92400e` / `#f59e0b`) for DRIFT / action needed
- red (`#991b1b` / `#ef4444`) for removed required fields / error / unsupported
- blue (`#1e40af` / `#3b82f6`) for informational analysis
- callout boxes:
  - verdict = green
  - action = amber
  - warning = red
  - impact = blue

### Global `html-report` Skill

The plan assumes a reusable global skill at:

```text
$HOME/.claude/skills/html-report/
```

Path note:

- the exact user-global path remains pending user confirmation
- this plan assumes the standard `$HOME/.claude/` location until confirmed

Responsibilities:

- input: structured drift JSON
- output: annotated self-contained HTML
- owns visual system and report formatting
- does not own schook-specific schema rules

The schook skill at `.claude/skills/hook-schema-drift/` is the domain-aware
caller. It runs the Python entry point, then hands the drift JSON to the
background `html-report` skill flow.

### Automatable Hook Coverage

The harness can automate:

- `SessionStart(startup)`
- `SessionEnd`
- `PreCompact`
- `PreToolUse(Bash)`
- `PostToolUse(Bash)`
- `PreToolUse(Agent)`
- `PermissionRequest(Write)`
- `PermissionRequest(Bash)`
- `Stop`

The harness cannot fully automate:

- `SessionStart(source=compact)` because it requires user `/compact`
- `SessionStart(source=clear)` because it requires user `/clear`

Schema impact of those manual-only hooks:

- the difference is in the `source` value
- they do not introduce additional structural fields

The report must therefore show:

- the last-known approved fixture
- a short manual capture procedure note

### Schema Version Strategy

Versioning rules:

- old schema = committed Pydantic models plus approved fixtures in the repo
- new schema = current-run captures
- Git history is the version archive
- no `schema-vN.json` files

On drift:

- report the change to the user
- recommend creating a schook issue
- do not auto-fix
- do not auto-update models

### Required Deliverables For Phase 3

Phase 3 is not complete until all of the following exist:

- `pyproject.toml` with `pydantic>=2.0` and `pytest`
- the discriminated-union Claude payload model
- `test-harness/hooks/run-schema-drift.py`
- per-provider adapters under `test-harness/hooks/<provider>/`
- drift JSON output per run
- a self-contained HTML report per run
- `.claude/skills/hook-schema-drift/`

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
