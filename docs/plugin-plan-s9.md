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
3. build and validate the global HTML reporting stack required by schema-drift reporting
4. Pydantic model and schema creation
5. plan revision from verified schema
6. only then hook implementation evaluation and sequencing

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
  - hook requirements, especially `HKR-001` through `HKR-012`
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
3. build and QA the global HTML reporting stack
4. create Pydantic models and schema artifacts from captured payloads
5. revise the plan from verified schema
6. re-evaluate or implement each hook plugin against validated schema

No later step may begin early because a prototype branch already exists.

## Implementation Start Rule

No hook runtime code starts until all of the following are true:

- this plan is reviewed and accepted
- Phase 1 has created `test-harness/hooks/README.md` and reviewers have accepted it as the harness contract
- Claude payloads for the required first-pass capture set are captured
- the global HTML reporting stack is complete and QA-approved
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
- `PreCompact`
- `PreToolUse(Bash)`
- `PostToolUse(Bash)`
- `PreToolUse(Agent)`
- `PermissionRequest`
- `Stop`

Still wired in the harness but not locally captured:

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

### S9-P2A: Phase 2A: Build Global HTML Reporting Stack

Status:
- Completed for Sprint 9 gating

Purpose:

- make the global HTML reporting dependency real before schema-drift tooling
  depends on it

Gate to start:

- Phase 2 complete with captured Claude fixtures

Deliverables:

- discovery layer:
  - `$HOME/.claude/skills/html-report/SKILL.md`
  - current local machine path:
    `/Users/randlee/.claude/skills/html-report/SKILL.md`
- execution layer:
  - `~/.claude/agents/html-report-generator.md`
  - current local machine path:
    `/Users/randlee/.claude/agents/html-report-generator.md`
- both files follow the prescriptive file/content requirements in
  `## Schema Drift Detection Tooling`
- both files pass review against:
  `/Users/randlee/Documents/github/synaptic-canvas/docs/claude-code-skills-agents-guidelines-0.4.md`
- one tested invocation (HKR-012) that produces a valid self-contained HTML file
  confirmed by `html-validate`

Done when:

- the discovery and execution layer files both exist at the required paths
- both pass review against the guidelines document
- the global execution layer can be invoked in the background from a caller
  skill
- a test invocation proves that the generated HTML is self-contained and valid
- Sprint 9 planning can point to the approved local draft as the satisfied
  prerequisite for Phase 3 and later phases

### S9-P3: Phase 3: Create Pydantic Models And Schema Artifacts

Status:
- Completed

Purpose:

- convert captured evidence into explicit validated contracts

Gate to start:

- Phase 2A complete

Deliverables:

- `pyproject.toml`
  - declares `pydantic>=2.0`
  - declares `pytest`
  - Done when:
    - file exists at repo root
    - `pip install -e .` succeeds
    - `pytest --collect-only` discovers the test suite without errors
- `test-harness/hooks/claude/models/payloads.py`
  - contains the complete discriminated-union Claude payload model
  - Done when:
    - `from test_harness.hooks.claude.models.payloads import ClaudeHookPayload`
      imports without error
    - all approved fixtures validate against the model
    - `pytest test-harness/hooks/claude/tests/` passes
- `test-harness/hooks/run-schema-drift.py`
  - single schema-drift CLI entry point
- `test-harness/hooks/<provider>/schema_drift.py`
  - one provider adapter module per supported provider
  - content requirements:
    - exports `run_drift(output_dir: Path) -> DriftReport`
    - encapsulates provider-specific capture, fixture loading, and drift
      comparison
    - returns only validated `DriftReport` data to the shared CLI
  - Done when:
    - `run-schema-drift.py claude` imports and calls the adapter
    - exit code matches the `PASS` / `DRIFT` / `ERROR` contract
    - drift JSON output validates against `DriftReport`
- `test-harness/hooks/<provider>/reports/<ISO-timestamp>/schema-drift-report.html`
  - single self-contained HTML report per run
- `test-harness/hooks/<provider>/drift-history/<ISO-timestamp>-drift.json`
  - immutable drift-history artifact for each run
- `.claude/skills/hook-schema-drift/SKILL.md`
  - project-local slash command definition
- discovery layer for global HTML reporting:
  - `$HOME/.claude/skills/html-report/SKILL.md`
  - current local machine path:
    `/Users/randlee/.claude/skills/html-report/SKILL.md`
- execution layer for global HTML reporting:
  - `~/.claude/agents/html-report-generator.md`
  - current local machine path:
    `/Users/randlee/.claude/agents/html-report-generator.md`
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
- every Phase 3 deliverable must name:
  - the exact file path
  - its content requirements
  - its done-when criteria

Phase 3 prerequisite:

- `S9-P2A` must be complete before Phase 3 begins
- the project-local `.claude/skills/hook-schema-drift/` command is the caller
  only; it does not replace the global HTML rendering stack

#### Pydantic Model Design

Phase 3 must include a complete Claude payload model design in Python so the
drift tooling and fixture validation path share the same contract:

```python
from __future__ import annotations

from enum import Enum
from typing import Annotated, Any, Literal, Optional, Union
from uuid import UUID

from pydantic import BaseModel, ConfigDict, Field, model_validator


class ProviderStatus(str, Enum):
    PASS = "PASS"
    DRIFT = "DRIFT"
    ERROR = "ERROR"
    NOT_SUPPORTED = "NOT_SUPPORTED"
    STALE = "STALE"


class DriftErrorCode(str, Enum):
    REQUIRED_FIELD_REMOVED = "REQUIRED_FIELD_REMOVED"
    FIELD_TYPE_CHANGED = "FIELD_TYPE_CHANGED"
    FIELD_ADDED = "FIELD_ADDED"
    OPTIONAL_FIELD_REMOVED = "OPTIONAL_FIELD_REMOVED"
    CAPTURE_FAILED = "CAPTURE_FAILED"
    PROVIDER_NOT_AVAILABLE = "PROVIDER_NOT_AVAILABLE"
    PROVIDER_NOT_SUPPORTED = "PROVIDER_NOT_SUPPORTED"


class BashToolInput(BaseModel):
    command: str
    description: Optional[str] = None


class AgentToolInput(BaseModel):
    prompt: str
    description: Optional[str] = None
    subagent_type: Optional[str] = None
    name: Optional[str] = None
    team_name: Optional[str] = None
    run_in_background: Optional[bool] = None


class BashToolResponse(BaseModel):
    output: Optional[str] = None
    stdout: Optional[str] = None
    error: Optional[str] = None
    stderr: Optional[str] = None
    interrupted: bool = False
    isImage: Optional[bool] = None
    noOutputExpected: Optional[bool] = None


class HookPayloadBase(BaseModel):
    model_config = ConfigDict(extra="allow")

    session_id: UUID
    hook_event_name: str
    cwd: str
    transcript_path: Optional[str] = None


class SessionStartPayload(HookPayloadBase):
    hook_event_name: Literal["SessionStart"]
    source: str
    model: Optional[str] = None


class SessionEndPayload(HookPayloadBase):
    hook_event_name: Literal["SessionEnd"]
    reason: Optional[str] = None


class PreCompactPayload(HookPayloadBase):
    hook_event_name: Literal["PreCompact"]
    trigger: Optional[str] = None
    custom_instructions: Optional[str] = None


class PreToolUseBashPayload(HookPayloadBase):
    hook_event_name: Literal["PreToolUse"]
    tool_name: Literal["Bash"]
    tool_input: BashToolInput
    permission_mode: Optional[str] = None
    tool_use_id: Optional[str] = None


class PreToolUseAgentPayload(HookPayloadBase):
    hook_event_name: Literal["PreToolUse"]
    tool_name: Literal["Agent"]
    tool_input: AgentToolInput
    permission_mode: Optional[str] = None
    tool_use_id: Optional[str] = None


class PostToolUseBashPayload(HookPayloadBase):
    hook_event_name: Literal["PostToolUse"]
    tool_name: Literal["Bash"]
    tool_input: BashToolInput
    tool_response: BashToolResponse
    permission_mode: Optional[str] = None
    tool_use_id: Optional[str] = None


class PermissionSuggestionRule(BaseModel):
    ruleContent: Optional[str] = None
    toolName: Optional[str] = None


class PermissionSuggestion(BaseModel):
    type: str
    behavior: Optional[str] = None
    destination: Optional[str] = None
    mode: Optional[str] = None
    rules: Optional[list[PermissionSuggestionRule]] = None


class PermissionRequestPayload(HookPayloadBase):
    hook_event_name: Literal["PermissionRequest"]
    tool_name: str
    tool_input: dict[str, Any]
    permission_mode: Optional[str] = None
    permission_suggestions: Optional[list[PermissionSuggestion]] = None


class StopPayload(HookPayloadBase):
    hook_event_name: Literal["Stop"]
    stop_hook_active: bool = False
    permission_mode: Optional[str] = None
    last_assistant_message: Optional[str] = None


class NotificationPayload(HookPayloadBase):
    hook_event_name: Literal["Notification"]
    # Deferred: no verified payload shape yet.


class DriftEntry(BaseModel):
    hook_event_name: str
    field_name: Optional[str] = None
    error_code: DriftErrorCode
    old_value: Optional[str] = None
    new_value: Optional[str] = None
    source: Optional[str] = None
    action: Optional[str] = None
    recovery: Optional[str] = None
    message: str


class DriftReport(BaseModel):
    provider: str
    run_timestamp: str
    status: ProviderStatus
    entries: list[DriftEntry]


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
- UUID format is enforced at the Python layer. This prevents malformed session
  IDs from corrupting Rust session-state deserialization in `S9-HP3`.
- `ProviderStatus` and `DriftErrorCode` are Python wire-format enums only. The
  Rust implementation in `S9-HP3` must define its own independent enums or
  newtypes in `sc-hooks-core`; Python and Rust do not share a type library.
- `HookPayloadBase` uses `ConfigDict(extra="allow")` for forward compatibility;
  extra fields must still be surfaced in drift output.
- `drift-report.json` must validate against `DriftReport`.
- unhandled Python exceptions in drift capture/classification must be caught and
  serialized as a `DriftEntry` with `error_code=CAPTURE_FAILED`.
- when `error_code=CAPTURE_FAILED`, the `DriftEntry` must include:
  - `source`: which hook surface or drift stage failed
  - `action`: user-facing recovery action such as `run manual capture`
  - `recovery`: optional machine-readable retry instruction when automatable

#### SessionStartPayload fields

| Field | Type | Required | Evidence source |
| --- | --- | --- | --- |
| `session_id` | `UUID` | yes | `test-harness/hooks/claude/fixtures/approved/session-start-startup.json` |
| `hook_event_name` | `Literal["SessionStart"]` | yes | same |
| `cwd` | `str` | yes | same |
| `transcript_path` | `Optional[str]` | no | same |
| `source` | `str` | yes | `test-harness/hooks/claude/fixtures/approved/session-start-startup.json`, `test-harness/hooks/claude/fixtures/approved/session-start-compact.json`, `test-harness/hooks/claude/fixtures/approved/session-start-resume.json`, `test-harness/hooks/claude/fixtures/approved/session-start-clear.json` |
| `model` | `Optional[str]` | no | same |

#### SessionEndPayload fields

| Field | Type | Required | Evidence source |
| --- | --- | --- | --- |
| `session_id` | `UUID` | yes | `test-harness/hooks/claude/fixtures/approved/session-end.json` |
| `hook_event_name` | `Literal["SessionEnd"]` | yes | same |
| `cwd` | `str` | yes | same |
| `transcript_path` | `Optional[str]` | no | same |
| `reason` | `Optional[str]` | no | `test-harness/hooks/claude/fixtures/approved/session-end-clear.json` |

#### PreCompactPayload fields

| Field | Type | Required | Evidence source |
| --- | --- | --- | --- |
| `session_id` | `UUID` | yes | `test-harness/hooks/claude/fixtures/approved/pre-compact-manual.json` |
| `hook_event_name` | `Literal["PreCompact"]` | yes | same |
| `cwd` | `str` | yes | same |
| `transcript_path` | `Optional[str]` | no | same |
| `trigger` | `Optional[str]` | no | same |
| `custom_instructions` | `Optional[str]` | no | same |

#### PreToolUseBashPayload fields

| Field | Type | Required | Evidence source |
| --- | --- | --- | --- |
| `session_id` | `UUID` | yes | `test-harness/hooks/claude/fixtures/approved/pretooluse-bash.json` |
| `hook_event_name` | `Literal["PreToolUse"]` | yes | same |
| `cwd` | `str` | yes | same |
| `transcript_path` | `Optional[str]` | no | same |
| `tool_name` | `Literal["Bash"]` | yes | same |
| `tool_input.command` | `str` | yes | same |
| `tool_input.description` | `Optional[str]` | no | same |
| `permission_mode` | `Optional[str]` | no | same |
| `tool_use_id` | `Optional[str]` | no | same |

#### PreToolUseAgentPayload fields

| Field | Type | Required | Evidence source |
| --- | --- | --- | --- |
| `session_id` | `UUID` | yes | `test-harness/hooks/claude/fixtures/approved/pretooluse-agent.json` |
| `hook_event_name` | `Literal["PreToolUse"]` | yes | same |
| `cwd` | `str` | yes | same |
| `transcript_path` | `Optional[str]` | no | same |
| `tool_name` | `Literal["Agent"]` | yes | same |
| `tool_input.prompt` | `str` | yes | same |
| `tool_input.description` | `Optional[str]` | no | same |
| `tool_input.run_in_background` | `Optional[bool]` | no | same |
| `permission_mode` | `Optional[str]` | no | same |
| `tool_use_id` | `Optional[str]` | no | same |

Deferred, not fixture-verified for implementation:

- `tool_input.subagent_type`
- `tool_input.name`
- `tool_input.team_name`

#### PostToolUseBashPayload fields

| Field | Type | Required | Evidence source |
| --- | --- | --- | --- |
| `session_id` | `UUID` | yes | `test-harness/hooks/claude/fixtures/approved/posttooluse-bash.json` |
| `hook_event_name` | `Literal["PostToolUse"]` | yes | same |
| `cwd` | `str` | yes | same |
| `transcript_path` | `Optional[str]` | no | same |
| `tool_name` | `Literal["Bash"]` | yes | same |
| `tool_input.command` | `str` | yes | same |
| `tool_input.description` | `Optional[str]` | no | same |
| `tool_response.stdout` | `Optional[str]` | no | same |
| `tool_response.stderr` | `Optional[str]` | no | same |
| `tool_response.interrupted` | `bool` | yes | same |
| `tool_response.isImage` | `Optional[bool]` | no | same |
| `tool_response.noOutputExpected` | `Optional[bool]` | no | same |
| `permission_mode` | `Optional[str]` | no | same |
| `tool_use_id` | `Optional[str]` | no | same |

Deferred, not fixture-verified for implementation:

- `tool_response.output`
- `tool_response.error`

#### PermissionRequestPayload fields

| Field | Type | Required | Evidence source |
| --- | --- | --- | --- |
| `session_id` | `UUID` | yes | `test-harness/hooks/claude/fixtures/approved/permission-request-write.json` |
| `hook_event_name` | `Literal["PermissionRequest"]` | yes | same |
| `cwd` | `str` | yes | same |
| `transcript_path` | `Optional[str]` | no | same |
| `tool_name` | `str` | yes | `test-harness/hooks/claude/fixtures/approved/permission-request-write.json`, `test-harness/hooks/claude/fixtures/approved/permission-request-bash.json` |
| `tool_input` | `dict[str, Any]` | yes | same |
| `permission_mode` | `Optional[str]` | no | same |
| `permission_suggestions` | `Optional[list[PermissionSuggestion]]` | no | same |

#### StopPayload fields

| Field | Type | Required | Evidence source |
| --- | --- | --- | --- |
| `session_id` | `UUID` | yes | `test-harness/hooks/claude/fixtures/approved/stop.json` |
| `hook_event_name` | `Literal["Stop"]` | yes | same |
| `cwd` | `str` | yes | same |
| `transcript_path` | `Optional[str]` | no | same |
| `stop_hook_active` | `bool` | yes | same |
| `permission_mode` | `Optional[str]` | no | same |
| `last_assistant_message` | `Optional[str]` | no | same |

#### NotificationPayload fields

`NotificationPayload` remains DEFERRED. No verified payload fixture exists in
`test-harness/hooks/claude/fixtures/approved/`, and the local Haiku harness
still records this surface as wired-but-unresolved rather than captured.

Done when:

- captured fixtures validate against the Claude Pydantic models
- schema artifacts exist for the validated Claude models
- drift policy is executable in tests, not just described in prose
- both drift JSON and the self-contained HTML report are produced by the Phase 3
  path; JSON-only output does not close the reporting deliverable
- the complete Phase 3 checklist in `Required Deliverables For Phase 3` exists;
  that checklist is the authoritative closure list for `S9-P3`

#### Rust design requirements for S9-P3 outputs

- the Python drift report is a wire-format boundary, not a shared library
  boundary
- `sc-hooks-core` must define Rust-native equivalents of:
  - `ProviderStatus`
  - `DriftErrorCode`
  - `DriftEntry`
  - `DriftReport`
- the Rust equivalents may mirror the Python names, but they must be declared
  independently with Rust derives and validation rules
- the Python CLI must preserve error provenance so the Rust side can surface the
  failure chain intact
- if `os.replace()` fails after the `.tmp` file is written:
  - the `.tmp` file must be explicitly removed
  - stderr must include the failing `.tmp` path
  - the command must exit non-zero
  - no partial success may be reported

### S9-P4: Phase 4: Revise The Plan From Verified Schema

Status:
- Completed

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
- Completed

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

### Implementation Start Gate

All five pre-implementation gate conditions now pass:

1. Capture baseline frozen: PASS
   - approved Claude fixture set exists under
     `test-harness/hooks/claude/fixtures/approved/`
   - locally captured startup sources now include `startup`, `compact`,
     `resume`, and `clear`
   - `Notification(idle_prompt)` remains explicitly excluded from the verified
     set and carried as a deferral
2. Reporting prerequisite satisfied: PASS
   - the global HTML reporting stack has an approved local draft and validated
     render/lint flow
   - `S9-P3` outputs no longer depend on an undefined reporting path
3. Schema + drift tooling complete: PASS
   - `test-harness/hooks/run-schema-drift.py claude` produces validated drift
     JSON, schema artifacts, and a self-contained HTML report
   - Phase 3 drift classification tests now cover:
     - required field removed
     - optional field removed
     - field added
     - field type changed
4. Plan revision complete: PASS
   - implementation-facing docs now distinguish:
     - fixture-backed fields
     - source-backed fields
     - explicit deferrals
   - no HP section below relies on `Notification` or other unresolved fields
     without a deferral label
5. Implementation sequence frozen: PASS
   - `S9-HP3 -> S9-HP4 -> S9-HP5`
   - `S9-HP5` is independent of Session Foundation at the planning layer only
     insofar as relay semantics were designed separately, but execution still
     depends on `S9-P5` and `S9-HP4`

Implementation may start because all five gate checks are now explicit and
reviewable in this document.

The implementation track after this phase must follow the Hook Phase 3/4/5
split from `docs/project-plan.md`:

#### S9-HP3: Hook Phase 3: Session Foundation

Status:
- Planned

Depends on:

- S9-P5 complete

Deliverables:

- `sc-hooks-core`
  - final trait/context freeze used by the hook runtime
  - canonical Rust types for:
    - `SessionId`
    - `ActivePid`
    - `ProjectRootDir`
    - `ProviderStatus`
    - `DriftErrorCode`
    - `DriftEntry`
    - `DriftReport`
  - structured `HookError` boundary and normalized `agent_state` transitions
- `sc-hooks-sdk`
  - provider adapters
  - handler registration
  - observability bridge integration
- `plugins/agent-session-foundation`
  - canonical session-state ownership
  - `SessionStart`, `SessionEnd`, `PreCompact`, and `Stop` handling
  - persisted session correlation keyed only by verified inputs
- same-PR updates to:
  - `docs/architecture.md`
  - `docs/requirements.md`
  - `docs/project-plan.md`
  whenever a new runtime crate lands

Verified field inputs allowed in HP3:

- `SessionStart`
  - `session_id`
  - `cwd`
  - `transcript_path`
  - `source`
  - `model` (optional)
- `SessionEnd`
  - `session_id`
  - `reason` (optional; verified from `/clear`)
- `PreCompact`
  - `trigger` (optional)
  - `custom_instructions` (optional)
- `Stop`
  - `stop_hook_active`
  - `permission_mode` (optional)
  - `last_assistant_message` (optional)
- source-backed runtime context:
  - `CLAUDE_PROJECT_DIR` -> `project_root_dir`

Explicitly deferred in HP3:

- `Notification(idle_prompt)` payload fields
- any parent/subagent lineage fields not present in approved Claude fixtures

Write scope:

- `sc-hooks-core/`
- `sc-hooks-sdk/`
- `plugins/agent-session-foundation/`
- same-PR updates to `docs/architecture.md`, `docs/requirements.md`, and
  `docs/project-plan.md`

Required tests:

- unit tests for normalized `agent_state` transitions
- integration tests for canonical session-state persistence keyed by
  `session_id`
- integration tests proving `SessionStart` in directory A and later lifecycle
  hooks in directory B still resolve the same session record through
  `project_root_dir`
- tests for atomic write behavior and unchanged-state no-rewrite behavior
- tests for mandatory hook-log emission on state-changing and no-op invocations
- `cargo test --workspace`
- `cargo clippy --all-targets --all-features -- -D warnings`

Success criteria:

- lifecycle code consumes only verified fields from the Phase 4 evidence set
- canonical session-state schema matches `HKR-004`, `HKR-008`, `HKR-009`, and
  `HKR-012`
- `project_root_dir` is chained from `CLAUDE_PROJECT_DIR`, never from cwd
- ATM-specific routing remains outside `plugins/agent-session-foundation`
- same-PR architecture inventory updates land when new runtime crates are added

Done when:

- the runtime work queue for session lifecycle is implementable without open
  design questions
- required tests pass
- no unverified fields are consumed in the lifecycle path

#### Rust design requirements for S9-HP3

- `HookError` must use structured variants with named fields and source-chain
  preservation via `thiserror`
- no library boundary may expose raw message-only errors
- every wrapped error must retain `#[source]`
- session identity types must remain newtypes, not bare primitives
- Python drift JSON is deserialized into Rust-owned types declared in
  `sc-hooks-core`, not shared directly across languages

#### S9-HP4: Hook Phase 4: Bash Identity + Spawn Gates

Status:
- Planned

Depends on:

- Hook Phase 3 complete

Deliverables:

- `plugins/agent-spawn-gates`
- `plugins/tool-output-gates`
- `PostToolUse(Bash)` hook routing — handled via `plugins/tool-output-gates`
- named-agent versus background-agent policy
- subagent linkage/tracking
- fenced/blocking JSON tool-utility behavior
- `.atm.toml` reference and lookup contract for ATM-aware policy decisions:
  - file name: `.atm.toml`
  - primary lookup location: `project_root_dir/.atm.toml`
  - child-agent behavior: inherit parent team context when child current
    directory differs but parent session still points at the same
    `project_root_dir`
  - fallback: no cwd heuristics; if `.atm.toml` is absent, ATM-specific policy
    is unavailable and the generic gate behavior continues
- same-PR architecture inventory updates when either plugin crate lands

Verified field inputs allowed in HP4:

- `PreToolUse(Bash)`
  - `tool_name == "Bash"`
  - `tool_input.command`
  - `tool_input.description` (optional)
  - `permission_mode` (optional)
  - `tool_use_id` (optional)
- `PreToolUse(Agent)`
  - `tool_name == "Agent"`
  - `tool_input.prompt`
  - `tool_input.description` (optional)
  - `tool_input.run_in_background` (optional)
  - `permission_mode` (optional)
  - `tool_use_id` (optional)
- `PostToolUse(Bash)`
  - `tool_response.stdout` (optional)
  - `tool_response.stderr` (optional)
  - `tool_response.interrupted`
  - `tool_response.isImage` (optional)
  - `tool_response.noOutputExpected` (optional)

Explicitly deferred in HP4:

- `PreToolUse(Agent).tool_input.subagent_type`
- `PreToolUse(Agent).tool_input.name`
- `PreToolUse(Agent).tool_input.team_name`
- `PostToolUse(Bash).tool_response.output`
- `PostToolUse(Bash).tool_response.error`

Write scope:

- `plugins/agent-spawn-gates/`
- `plugins/tool-output-gates/`
- `.atm.toml` documentation and any same-PR doc updates needed to clarify the
  blocking contract

Required tests:

- direct tests for `tool_name = "Agent"` spawn-gate routing
- tests for `PostToolUse(Bash)` payload routing through `tool-output-gates`
- tests for named-agent versus background-agent policy outcomes
- tests for subagent linkage fields written into the canonical session-state
  file
- tests for `.atm.toml` lookup order and missing-file behavior
- tests for fenced `json` extraction and schema validation success/failure
- tests proving invalid input returns exact retryable failure reasons
- `cargo test --workspace`
- `cargo clippy --all-targets --all-features -- -D warnings`

Success criteria:

- spawn and tool-blocking behavior is tested directly
- `plugins/tool-output-gates` is implemented separately from
  `plugins/agent-spawn-gates`
- `HKR-010`, `HKR-011`, and `HKR-013` are satisfied by concrete crate scope and
  tests
- no unverified field is relied on without a later approved schema capture
- block responses explain exactly how the caller can retry successfully

Done when:

- both gate plugins are specified well enough to implement without follow-up
  design questions
- required tests pass
- no unverified fields are consumed in spawn or tool-output policy

#### S9-HP5: Hook Phase 5: Relay Hooks

Status:
- Planned

Depends on:

- S9-P5 complete AND S9-HP4 complete

Deliverables:

- `plugins/atm-extension`
- ATM-specific Bash identity-file handling
- relay mechanism for:
  - `PermissionRequest`
  - `Stop`
  - teammate-idle
- `Notification(idle_prompt)` remains explicitly deferred unless it is captured
  live later
- same-PR architecture inventory updates when the ATM extension crate lands

Verified field inputs allowed in HP5:

- `PermissionRequest`
  - `tool_name`
  - `tool_input`
  - `permission_mode` (optional)
  - `permission_suggestions` (optional)
- `Stop`
  - `session_id`
  - `stop_hook_active`
  - `permission_mode` (optional)
  - `last_assistant_message` (optional)
- teammate-idle
  - ATM extension event only; not a Claude payload contract claim

Explicitly deferred in HP5:

- `Notification(idle_prompt)` payload shape until it is captured live
- any ATM relay field not backed by generic session-state schema plus ATM
  extension docs

Write scope:

- `plugins/atm-extension/`
- ATM-only docs where relay semantics or teammate-idle mapping must be frozen

Required tests:

- tests for ATM identity-file create/delete behavior around `atm` Bash commands
- tests for `PermissionRequest`, `Stop`, and teammate-idle relay behavior
- tests proving ATM extension fields layer onto the canonical session record
  instead of redefining it
- `cargo test --workspace`
- `cargo clippy --all-targets --all-features -- -D warnings`

Success criteria:

- ATM relay behavior layers on generic utilities instead of defining them
- relay targets and relay mechanism are explicitly documented
- ATM environment inheritance and child override behavior are tested
- unresolved `Notification(idle_prompt)` capture does not block HP5 unless a
  later requirement explicitly changes the baseline

Done when:

- relay-extension work is implementation-ready without open design questions
- required tests pass
- ATM behavior is clearly layered on generic utilities, not defining them

Rules:

- prototype branches remain reference-only until this phase completes the re-evaluation
- no field may be consumed in runtime code unless it is backed by the verified Phase 4 outputs
- architecture inventory updates must happen in the same PR as any accepted runtime crate introduction

Done when:

- implementation tasks are derived from verified schema rather than inferred payload shapes
- the runtime work queue is ready to start without guessing
- the ordered implementation checkpoint below is accepted as the final sequence

### Final Implementation Sequence Checkpoint

1. `S9-HP3` Session Foundation
   - first because all later crates depend on canonical session-state,
     `project_root_dir`, normalized `agent_state`, and mandatory hook logging
2. `S9-HP4` Bash Identity + Spawn Gates
   - second because spawn/tool-output policy depends on the HP3 state substrate
     but resolves its own deferred fields explicitly
3. `S9-HP5` Relay Hooks
   - third because ATM relay behavior layers onto the generic runtime and uses
     HP4 outputs for Bash/tool policy integration

No later hook-runtime PR may invert this sequence without updating this section
and the matching project-plan row.

## Immediate Work Package

S9-PBC is tracked separately as:

- `S9-PBC` — Plan-BC: BC Design Consolidation
- current status: In QA
- current authority: `docs/phase-bc-hook-runtime-design.md`

The immediate Sprint 9 work is limited to Phases 0 through 5.

That means the current task is:

- make the harness build-and-capture sequence explicit
- make the global HTML reporting prerequisite explicit
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

CLI contract:

- implemented with `argparse`
- positional argument: `<provider>`
- optional argument: `--output-dir`
- exit code `0` = `PASS`
- exit code `1` = `DRIFT`
- exit code `2` = `ERROR` or `NOT_SUPPORTED`
- drift JSON path is echoed to stdout
- all errors are written to stderr
- file output must use atomic write via `<output>.tmp` and `os.replace(...)`
- if `os.replace(...)` fails after the temp file is written:
  - remove the temp file explicitly
  - emit stderr that includes the temp-file path
  - return exit code `2`
  - do not report success

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
2. on completion, invoke the global `html-report-generator` execution layer as a
   background agent with `run_in_background=true`
3. the background agent reads the drift JSON and generates the annotated HTML report
4. the calling agent receives the report path and displays it to the user

The global HTML background execution must stay in the background so the calling
agent does not accumulate HTML-generation context.

`hook-schema-drift` skill file spec:

- file path: `.claude/skills/hook-schema-drift/SKILL.md`
- content requirements:
  - YAML frontmatter
  - command contract for `/hook-schema-drift <provider>`
  - explicit dependency on the global html-report stack
  - explicit `run_in_background=true` requirement for the HTML execution layer
  - failure behavior when the background HTML agent fails:
    - drift JSON path still returned
    - error surfaced to stderr/user
    - no silent success

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
- HTML5 `<!DOCTYPE html>`
- `<meta charset="utf-8">`
- HTML-escape all payload content before rendering
- minimal inline JavaScript is permitted only for clipboard copy actions on copy
  buttons; no other JavaScript is allowed

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
   - ANALYSIS block below the diff, supplied by the global HTML reporting stack
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

### Global HTML Reporting Stack

The plan requires a reusable two-tier global HTML reporting stack:

```text
$HOME/.claude/skills/html-report/SKILL.md
~/.claude/agents/html-report-generator.md
```

Required reading reference before implementation:

`/Users/randlee/Documents/github-radiant/data-sourcegenerators/.claude/agents/xhtml-plugin-expert.md`

This agent demonstrates the correct structure, CSS conventions, and content
patterns for self-contained HTML reports. The new
`html-report-generator` MUST follow its structural patterns unless the
guidelines document below says otherwise.

QA gate:

the completed `~/.claude/skills/html-report/SKILL.md` and
`~/.claude/agents/html-report-generator.md` MUST pass review against:

`/Users/randlee/Documents/github/synaptic-canvas/docs/claude-code-skills-agents-guidelines-0.4.md`

The QA reviewer will use that document as the acceptance checklist for those
two files. The implementer must read it before writing a single line.

Discovery layer:

- normalized path: `$HOME/.claude/skills/html-report/SKILL.md`
- current local machine path: `/Users/randlee/.claude/skills/html-report/SKILL.md`

Execution layer:

- normalized path: `~/.claude/agents/html-report-generator.md`
- current local machine path: `/Users/randlee/.claude/agents/html-report-generator.md`

Responsibilities:

- discovery layer (`~/.claude/skills/html-report/SKILL.md`):
  - routes callers to the execution layer
  - documents the generic HTML reporting purpose
  - contains no repo-specific policy
- execution layer (`~/.claude/agents/html-report-generator.md`):
  - consumes structured drift JSON
  - returns structured success/error JSON
  - emits the self-contained HTML report
  - owns visual system and report formatting
  - does not own schook-specific schema rules

The schook skill at `.claude/skills/hook-schema-drift/` is the domain-aware
caller. It runs the Python entry point, then hands the drift JSON to the
background `html-report-generator` execution layer.

Prescriptive deliverable spec for the global html-report stack:

File 1: `~/.claude/skills/html-report/SKILL.md`

Content requirements:

- YAML frontmatter:
  - `name: html-report`
  - `version: 1.0.0`
  - `description:` one line
- `## Purpose` section
- `## Agent Delegation` section naming `html-report-generator`
- `## Input Contract` section
- `## Scratchpad Areas` section
- `## Example invocation` section with a minimal fenced JSON input example
- zero repo-specific content

Done when:

- file exists at the normalized path
- content requirements are present in the file
- zero repo-specific content is preserved
- the file passes review against:
  `/Users/randlee/Documents/github/synaptic-canvas/docs/claude-code-skills-agents-guidelines-0.4.md`

File 2: `~/.claude/agents/html-report-generator.md`

Content requirements:

- YAML frontmatter:
  - `name: html-report-generator`
  - `version: 1.0.0`
  - `description:` one line
- `## Input` section with fenced JSON schema showing required fields
- `## Output` section with fenced JSON schema showing:
  - `{ success, data: { report_path }, error }`
- `## Report Structure` section
- `## Scratchpad` section documenting the `<div class="scratchpad">` pattern
- `## Code Examples` section with a minimal HTML document skeleton and inline CSS

Done when:

- file exists at the normalized path
- background-agent invocation from `hook-schema-drift` succeeds and returns
  `{ success: true, data: { report_path } }`
- generated HTML is self-contained and validates against the visual conventions
- the file passes review against:
  `/Users/randlee/Documents/github/synaptic-canvas/docs/claude-code-skills-agents-guidelines-0.4.md`

Required readiness before report-generating work can close:

- both files exist at the specified paths
- both pass review against:
  `/Users/randlee/Documents/github/synaptic-canvas/docs/claude-code-skills-agents-guidelines-0.4.md`
- `html-report-generator` can be invoked as a background agent from
  `hook-schema-drift`
- a test invocation produces a valid self-contained HTML file
- a tested invocation example is checked and documented
- the generated HTML is self-contained and matches the visual conventions in
  this section

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
- drift history path:
  `test-harness/hooks/<provider>/drift-history/<ISO-timestamp>-drift.json`
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
- `test-harness/hooks/claude/models/payloads.py` with the discriminated-union
  Claude payload model
- `test-harness/hooks/run-schema-drift.py`
- per-provider adapters at `test-harness/hooks/<provider>/schema_drift.py`
- per-hook field tables for all verified hook types matching captured fixture
  evidence (see §S9-P3 field tables)
- explicit `DriftEntry` model definition with all required fields:
  `hook_event_name`, `field_name`, `error_code`, `old_value`, `new_value`,
  `source`, `action`, `recovery`, `message`
- explicit `DriftReport` model definition with all required fields:
  `provider`, `run_timestamp`, `status`, `entries`
- drift JSON output per run that validates against `DriftReport`
- a self-contained HTML report per run
- `.claude/skills/hook-schema-drift/SKILL.md`
- `$HOME/.claude/skills/html-report/SKILL.md`
- `~/.claude/agents/html-report-generator.md`

Required pytest coverage:

- fixture validation for all approved Claude fixtures
- required-field removal detection
- extra-field tolerance with surfaced drift reporting
- drift classification logic for every `DriftErrorCode`
- HTML structure validation
- marker split between fixture/schema tests and `live_capture`

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
