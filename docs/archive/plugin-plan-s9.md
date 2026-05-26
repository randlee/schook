> Archived during `SC-DOCS-COMPLIANCE-1`.
>
> This file is retained as the completed Sprint 9 planning artifact and
> provider-evidence ledger. Current control-doc ownership lives in
> `docs/requirements.md`, `docs/architecture.md`, `docs/project-plan.md`, and
> `docs/hook-api/`.

# Sprint 9 Plugin Hook Plan

## Goal

Plan the Rust plugin implementation of the current Claude ATM hook set using
the installed Python hooks under `/Users/randlee/.claude/scripts/` as the
behavioral reference.

This sprint is planning-only. It does not change runtime code.

The intended execution sequence after review is:

1. build the Claude-first harness
2. capture verified Claude payloads
3. revise this plan from captured evidence
4. implement the Claude ATM hook crates
5. defer other providers until the Claude baseline is stable

## Umbrella Plan Status

This is the umbrella execution plan for Sprint 9.

QA should be able to read this document and understand the complete sequence
from:

1. harness build
2. live schema capture
3. model and schema validation
4. formal report generation
5. plan revision from captured evidence
6. hook implementation sequencing

Supporting documents remain necessary, but they are source-of-truth references
for facts and subcontracts rather than parallel execution plans.

## Planning Baseline

Supporting documents and their roles:

- `docs/hook-api/claude-hook-api.md`
  - verified Claude hook baseline and current known payload facts
- `docs/hook-api/codex-hook-api.md`
  - documented provider reference only; not part of the first implementation path
- `docs/hook-api/cursor-agent-hook-api.md`
  - documented provider reference only; not part of the first implementation path
- `docs/hook-api/atm-hook-extension.md`
  - ATM-specific behavior that must remain separate from the generic hook contract
- `test-harness/hooks/README.md`
  - harness directory contract, `pytest` split, fixture policy, and report lifecycle
- `docs/project-plan.md`
  - high-level project sequencing, including the Hook Phase 0-6 track
- `docs/requirements.md`
  - requirements that gate hook implementation start and scope
- `docs/architecture.md`
  - architecture boundaries and planned hook crate ownership

Sprint 9 focuses on the verified Claude hook set. Codex is documented as a
separate platform reference, but it is not the implementation baseline for
this sprint.

Current ATM-specific source of truth comes from `agent-team-mail` docs, hook
scripts, tests, and Rust fallback code. The repo does not currently appear to
use Pydantic models as the hook source of truth, so this plan treats the
schema-validation harness as new required work rather than something already
available upstream.

## Core Recommendation

Do not plan this work as eight flat ports. Plan it in five layers:

1. live schema capture and drift validation
2. session/context foundation
3. command and spawn gates
4. ATM lifecycle relays
5. cross-platform follow-on gaps

That order keeps the hook contract honest before implementation starts. The
first requirement is to validate real provider payloads against explicit models
so upstream schema drift is detected immediately after provider upgrades.

## Supplementary Background For Hook Phase 1: Hook Schema Validation Harness

Authoritative sequencing for implementation belongs to the `### Hook Phase N`
sections later in this document. This background section exists to explain the
first gate in detail, not to define a parallel authoritative sequence.

This is now the first step of the plan.

Purpose:

- capture real hook payloads from installed AI runtimes
- validate them against provider-specific models
- detect schema drift immediately when Claude, Codex, Gemini, or Cursor Agent
  changes hook payloads

Installed runtimes currently available on this machine:

- `claude`
- `codex`
- `gemini`
- `cursor-agent`

Recommended repo layout:

- `test-harness/hooks/README.md`
- `test-harness/hooks/pytest.ini`
- `test-harness/hooks/common/`
- `test-harness/hooks/claude/`
- `test-harness/hooks/codex/`
- `test-harness/hooks/gemini/`
- `test-harness/hooks/cursor-agent/`

Recommended validation design:

- keep provider-native payload models separate
- validate captured payloads against explicit models
- fail on required-field removal or type drift
- report unknown new fields for review rather than silently discarding them
- preserve raw captured payload fixtures as evidence

Recommended technology split:

- documented JSON shape as the durable contract artifact
- Python validation models for capture-time and CI-time schema checking
- live capture scenarios that exercise real hooks with cheap/fast tasks
- canned prompts for repeatable provider runs
- local Python capture hooks
- automatic report generation under the harness output tree

Harness source-of-truth rule:

- the detailed harness contract, directory ownership, `pytest` split, fixture
  policy, and report lifecycle live under `test-harness/hooks/README.md`

Harness execution contract summary for this plan:

- `pytest` is the required harness runner
- default `pytest` runs fixture/model/schema validation only
- `pytest -m live_capture` runs provider launches and live payload capture
- canned prompts are required for repeatable provider runs
- local Python hook scripts are the required capture mechanism
- every live run writes raw captures, normalized artifacts, and formal reports
- no manual copying or hand-moving of run artifacts is part of the workflow

Harness output contract:

- raw captured payloads are evidence
- approved fixtures are long-lived contract snapshots
- generated reports are the formal review artifact for each live run

Expected output tree per live run:

```text
<provider>/captures/<run-id>/
  raw/
  normalized/

<provider>/reports/<run-id>/
  validation-summary.json
  drift-report.json
  run-report.md
```

Current evidence rule:

- no field may be promoted into implementation-facing docs unless it is backed
  by existing source-of-truth docs/scripts/tests or by captured harness
  fixtures
- no hook code is written until the relevant provider payload schema has been
  captured and validated

Minimum first-pass capture matrix:

- Claude: `SessionStart`, `SessionEnd`, `PreCompact`, `PreToolUse(Bash)`,
  logical teammate/background spawn via current `PreToolUse(tool_name="Agent")`,
  `PostToolUse(Bash)`, `PermissionRequest`, `Stop`, and unresolved `Notification`

Documented but deferred from the first harness pass:

- Codex
- Gemini
- Cursor Agent

Acceptance for this first step:

- the harness can launch Claude in a minimal scripted run
- raw Claude hook payloads are captured and stored by hook type
- provider-specific models validate the captured Claude payloads
- CI fails on breaking hook schema drift
- the S9 implementation plan only promotes Claude implementation fields that
  are backed by captured payload evidence or existing source-of-truth docs

Model and schema promotion rules for this step:

- provider-native models stay separate; no cross-provider shared schema is assumed
- known required fields are strict
- newly observed extra fields are reported for review rather than silently relied on
- implementation may depend only on fields promoted from evidence into the validated model
- schema artifacts should be generated from models whenever possible
- captured payloads and reports must be created automatically by harness scripts

## Supplementary Background For Hook Phase 2: Plan Revision After Full Schema Capture

Authoritative sequencing for implementation belongs to the `### Hook Phase N`
sections later in this document. This background section explains the required
post-capture revision gate in detail.

This is a required step before any hook implementation code.

Purpose:

- revise the remaining hook plan from verified payload evidence
- remove any placeholder assumptions left from pre-capture planning
- freeze the exact fields each provider/hook implementation is allowed to rely
  on

Required outputs:

- update `docs/plugin-plan-s9.md`
- update provider hook API documents under `docs/hook-api/`
- mark each planned field or behavior as:
  - verified by existing source-of-truth implementation/docs, or
  - verified by captured harness fixtures

Acceptance for this second step:

- every planned Claude implementation field is traceable to a source document,
  live script, test, Rust reader, or captured fixture
- every still-unknown field remains explicitly marked unknown/deferred
- no code-writing task starts before this revision step is complete

## Implementation Start Gate

No hook implementation code starts until all of the following are true:

- this umbrella plan is reviewed and accepted
- `test-harness/hooks/README.md` is reviewed and accepted as the harness contract
- Claude hook payloads for the required first-pass matrix are captured
- provider models validate the captured Claude payloads
- this plan is revised from that captured evidence

If any one of those conditions is missing, the work remains in planning or
schema-capture mode rather than implementation mode.

## Review Gate

This document is ready for review when:

- the first development step is clearly the Claude-first harness
- the required Claude capture matrix is explicit
- the post-capture plan revision step is mandatory
- ATM-specific behavior remains isolated in `docs/hook-api/atm-hook-extension.md`
- Codex, Gemini, and Cursor remain documented without being turned into
  immediate development blockers
- the harness execution contract is summarized here rather than hidden only in
  the harness README
- the implementation start gate is explicit and binary

## Full QA Review Package

QA should review this document as the umbrella plan together with:

- `test-harness/hooks/README.md`
- `docs/hook-api/claude-hook-api.md`
- `docs/hook-api/atm-hook-extension.md`
- `docs/project-plan.md`
- `docs/requirements.md`
- `docs/architecture.md`

Review intent:

- this document owns the execution sequence
- the referenced documents own platform facts, ATM-specific details, harness
  contract specifics, and project/architecture boundaries
- no essential execution step should be present only in a supporting document

## Immediate Development Scope

First development pass:

- Claude harness capture
- Claude plan revision
- Claude ATM hook implementation

Not part of the first development pass:

- Codex harness or runtime work
- Gemini harness or runtime work
- Cursor harness or runtime work

## Session And Agent Correlation Model

The session/agent key must survive directory changes and compact/resume.

Identity anchor:

1. `session_id` captured during `SessionStart`
2. hook subprocess parent PID (`PPID`) as a same-process cross-check
3. `ATM_TEAM` + `ATM_IDENTITY` as routing labels only

Current verified ATM-backed behavior:

- persist a per-session record keyed by `session_id`
- store `pid`, `team`, `identity`, and timestamps
- allow later hooks to recover session/agent context from that record even when
  current working directory changes
- never use cwd as the identity key

Current verified Claude payload facts:

- `SessionStart` currently uses raw payload field `source`
- verified observed `source` values are `startup` and `compact`
- `resume` is not currently a verified literal payload value
- logical teammate/background-agent spawn currently arrives as `PreToolUse`
  with `tool_name = "Agent"` in Haiku capture
- `PermissionRequest` was live-captured for both `tool_name = "Write"` and
  `tool_name = "Bash"`
- `PreCompact` and post-compact `SessionStart(source="compact")` are both now
  live-captured
- `Notification` is still unresolved after repeated long-idle runs in this
  environment

Implementation-planning rule:

- any internal fresh/resume/compact classification must be derived from
  verified evidence and persisted state, not documented as a Claude wire enum
  unless the harness proves it

## Normalized Agent State Model

Raw provider events and normalized runtime state are separate concerns.

Required normalized field:

- `agent_state`

Recommended normalized enum:

- `starting`
- `busy`
- `awaiting_permission`
- `compacting`
- `idle`
- `ended`

Required identity/state tuple:

- `session_id`
- `active_pid`
- `agent_state`

Transition guidance from current Claude + ATM evidence:

- `SessionStart(source="startup")` -> `starting`
- tool execution / active turn -> `busy`
- `PermissionRequest` -> `awaiting_permission`
- `PreCompact` -> `compacting`
- `Stop` -> `idle`
- `SessionEnd` -> `ended`

Adjacent ATM/team signal:

- `teammate_idle` is a distinct raw event used in `agent-team-mail`
- it should also map to normalized `idle` for long-lived teammate agents that
  may never emit `Stop`

## Session State File Schema

The hook track should use a single canonical session-state file per
`session_id`. ATM enriches that same file; it does not create a second
authoritative session file.

Required base fields:

- `session_id`: string
- `active_pid`: integer
- `agent_state`: enum
- `created_at`: unix timestamp
- `updated_at`: unix timestamp
- `ai_root_dir`: string
- `ai_current_dir`: string

Recommended optional generic fields:

- `agent_type`: string
- `agent_type_source`: string
- `parent_session_id`: string
- `subagent_id`: string

Recommended optional ATM extension object in the same file:

- `extensions.atm.atm_name`: string
- `extensions.atm.atm_team`: string
- `extensions.atm.atm_agent_id`: string

Recommended schema shape:

```json
{
  "session_id": "<uuid>",
  "active_pid": 12345,
  "agent_state": "idle",
  "created_at": 1743120000.0,
  "updated_at": 1743120060.0,
  "ai_root_dir": "/repo/root",
  "ai_current_dir": "/repo/root/subdir",
  "agent_type": "explorer",
  "agent_type_source": "built_in",
  "parent_session_id": null,
  "subagent_id": null,
  "extensions": {
    "atm": {
      "atm_name": "arch-hook",
      "atm_team": "atm-dev",
      "atm_agent_id": "arch-hook@atm-dev"
    }
  }
}
```

Rules:

- `session_id` identifies the logical agent session
- `active_pid` is the single authoritative live process id for that session
- pid rebinding happens only at deterministic startup handoff points
- `PreCompact` does not change `session_id`
- `SessionStart(source="compact")` preserves `session_id` and updates state
  after compaction
- directory changes update `ai_current_dir` only; they do not redefine identity
- ATM fields are optional extensions on the same file, not a separate state file

## Hook Inventory

| Hook behavior | Claude surface | Matcher/event | Python reference | Input fields consumed | Action logic | `sc-hooks` fit | Recommended crate |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Session start | `SessionStart` | `*` | `session-start.py` | `session_id`, `source`, `.atm.toml`, `ATM_TEAM`, `ATM_IDENTITY` | announce session, emit ATM start event, persist session record | fits current lifecycle hook model | `plugins/agent-session-foundation` |
| Session end | `SessionEnd` | `*` | `session-end.py` | `session_id`, `.atm.toml` core routing | emit ATM end event, delete session record | fits current lifecycle hook model | `plugins/agent-session-foundation` |
| Pre-compact | `PreCompact` | `""` | harness capture now proves the raw surface | `session_id`, `trigger`, `custom_instructions`, transcript/cwd context | emit pre-restart compact lifecycle signal | fits lifecycle hook model | `plugins/agent-session-foundation` |
| ATM identity write | `PreToolUse` | `Bash` | `atm-identity-write.py` | `tool_input.command`, `session_id`, `.atm.toml`, `ATM_*` env | if command invokes `atm`, write temp identity file | fits ATM extension behavior | `plugins/atm-extension` |
| ATM identity cleanup | `PostToolUse` | `Bash` | `atm-identity-cleanup.py` | routing context only | remove temp identity file from paired bash invocation | fits ATM extension behavior | `plugins/atm-extension` |
| Agent spawn gate | `PreToolUse` | logical teammate/spawn surface; current Haiku payload uses `tool_name = "Agent"` | `gate-agent-spawns.py` | `tool_input.subagent_type`, `tool_input.name`, `tool_input.team_name`, `session_id`, team config | enforce named-teammate and team spawn policy | fits current tool hook model | `plugins/agent-spawn-gates` |
| Idle relay | `Notification` | `""` | `notification-idle-relay.py` | intended to use `session_id`, team/agent routing fields | emit ATM idle heartbeat | documented surface remains in scope; live Claude payload unresolved in this harness | `plugins/atm-extension` |
| Permission relay | `PermissionRequest` | `*` | `permission-request-relay.py` | `session_id`, `tool_name`, `tool_input`, team/agent routing fields | emit ATM permission-request event | fits ATM extension behavior | `plugins/atm-extension` |
| Stop relay | `Stop` | `*` | `stop-relay.py` | `session_id`, team/agent routing fields | emit ATM stop/idle event | fits ATM extension behavior | `plugins/atm-extension` |

## Protocol Compatibility Analysis

### Fully Compatible With Current `sc-hooks` Contract

These behaviors fit the currently documented hook taxonomy and stdin payload
model without requiring a new host protocol:

- `SessionStart`
- `SessionEnd`
- `PreCompact`
- `PreToolUse/Bash`
- `PostToolUse/Bash`
- logical teammate/background spawn surface via current `PreToolUse(tool_name="Agent")`
- unresolved `Notification`
- `PermissionRequest`
- `Stop`

Why they fit:

- the host already supports these hook names and matcher rules in
  `docs/protocol-contract.md`
- payload passthrough already gives plugins access to nested tool input fields
- lifecycle hooks already allow `*` as the matcher posture for these events

### Design Gaps To Keep Explicit

- Claude-specific routing context such as `.atm.toml`, `ATM_TEAM`, and
  `ATM_IDENTITY` is external policy input, not part of the generic `sc-hooks`
  protocol contract
- command-sensitive Bash behavior depends on payload shape under
  `payload.tool_input.command`; this is compatible today, but the plan should
  treat that field as required metadata for the relevant plugins
- session continuity depends on SessionStart persistence; later hooks should not
  re-infer identity from cwd
- Codex parity is not part of this sprint baseline because Codex does not yet
  have the same verified session-start capture path in this repo

## Cursor-Agent Follow-On Scope

Cursor Agent is in scope for the same planning branch/PR, but it is not part of
the verified Claude implementation baseline.

Current verified planning baseline for Cursor comes from:

- `cursor-agent --help`
- `https://cursor.com/docs/hooks`
- local Cursor CLI state under `/Users/randlee/.cursor/`

Current locally verified facts:

- `cursor-agent` is installed on this machine
- current CLI supports `--print`, `--output-format`, `--mode`, `--resume`,
  `--continue`, `--workspace`, and `--worktree`
- there is no current `/Users/randlee/.cursor/hooks.json` on this machine

Current publicly documented hook names relevant to this plan:

- controllable:
  - `beforeShellExecution`
  - `beforeMCPExecution`
  - `beforeReadFile`
- informational:
  - `afterFileEdit`
  - `stop`

Planning rule:

- use these hook names for sequencing and crate layout only
- do not promote any Cursor stdin fields into implementation scope until a
  later dedicated Cursor harness pass captures real payloads for the installed
  `cursor-agent` runtime

Current execution decision:

- keep Cursor API documentation in this planning set now
- defer Cursor harness capture and Cursor-targeting development until after the
  Claude ATM baseline has been captured, reviewed, revised, and implemented

Recommended crate split after schema capture:

- `plugins/cursor-agent-gates`
  - `beforeShellExecution`
  - `beforeMCPExecution`
  - `beforeReadFile`
- `plugins/cursor-agent-relay`
  - `afterFileEdit`
  - `stop`

These are planning targets only. Like the Claude/ATM crate targets above, they
remain scaffold/reference-only proposals until implementation lands with tests
and the same-PR architecture inventory update.

## Recommended Crate Layout

Keep policy-heavy and stateful hooks separated from generic relays.

Posture statement:

- freeze the generic hook trait and normalized context first
- keep generic hook utilities separate from ATM-specific extension behavior
- treat the archived prototype crates as reference only, not as the final crate
  split

Preferred post-capture implementation split:

- generic utilities:
  - `plugins/agent-session-foundation`
  - `plugins/agent-spawn-gates`
  - `plugins/tool-output-gates`
- ATM-specific extension:
  - `plugins/atm-extension`

Archived prototype branches/crates remain reference-only:

- `plugins/atm-session-lifecycle`
- `plugins/atm-bash-identity`
- `plugins/gate-agent-spawns`
- `plugins/atm-state-relay`

Inventory rule:

- any implementation sprint that adds one of these crates must update
  `docs/architecture.md` §3 crate inventory in the same PR

- `plugins/agent-session-foundation`
  - owns `SessionStart`, `SessionEnd`, and `PreCompact`
  - owns persistent session record read/write
  - owns normalized `agent_state` transitions
- `plugins/agent-spawn-gates`
  - owns named-agent vs background-agent policy
  - owns subagent linkage and spawn tracking
- `plugins/tool-output-gates`
  - owns tool blocking/fenced-JSON policy and related utility enforcement
- `plugins/atm-extension`
  - owns ATM routing enrichment
  - owns ATM-specific Bash identity-file behavior
  - owns ATM relay emission behavior such as `PermissionRequest`, `Stop`, and
    teammate-idle mapping

Recommended supporting test infrastructure:

- `test-harness/hooks/` for provider adapters, capture fixtures, models, and
  schema-drift tests

Why this split:

- session lifecycle and normalized state are foundational and generic
- spawn policy and subagent tracking are generic agent-control concerns
- tool blocking/fenced-JSON handling is a separate correctness utility
- ATM routing and relay behaviors are extensions, not the generic hook baseline
- the archived prototypes mostly got the broad responsibility boundaries right,
  but they pulled ATM behavior into the first-pass crate split too early

## Sprint Sequencing

### Hook Phase 0: Review Baseline

Deliver:

- freeze this document as the umbrella Sprint 9 execution plan
- review and freeze the provider API docs
- review and freeze `test-harness/hooks/README.md` as the harness source of
  truth
- confirm the first implementation path is Claude-first and ATM-aware without
  widening the generic hook contract

Dependencies:

- Sprint 6 formally accepted; this is the review gate before harness build work starts

### Hook Phase 1: Live Schema Capture And Drift Validation

Deliver:

- Claude provider launch adapter
- Claude hook payload models
- Claude fixture capture and validation tests
- drift-report output for unknown-field additions and breaking schema changes
- approved fixture snapshots and a first live Claude Haiku report

Write scope:

- `test-harness/hooks/README.md`
- `test-harness/hooks/scripts/run-capture.sh`
- `test-harness/hooks/claude/{prompts,hooks,models,fixtures,captures,reports,scripts,tests}/`
- fixture manifests and harness runner helpers

Required tests:

- `pytest test-harness/hooks/`
- harness structure and fixture validation tests under
  `test-harness/hooks/claude/tests/`

Success criteria:

- Claude payloads for the required baseline are captured and curated as review
  evidence
- the harness can be rerun from the repo docs without reconstructing ad hoc
  setup
- CI fails on required-field removal or type drift

Dependencies:

- Phase 0 review and acceptance complete (umbrella plan accepted; `test-harness/hooks/README.md` accepted as harness contract)

### Hook Phase 2: Plan Revision From Captured Schema

Deliver:

- revised hook API docs with only verified fields
- revised S9 plan for the remaining Claude implementation work
- explicit deferral markers for anything still not captured or source-backed
- frozen normalized `agent_state` model and session-record schema
- frozen hook-plugin trait/result/context contract

Write scope:

- `docs/plugin-plan-s9.md`
- `docs/hook-api/claude-hook-api.md`
- `docs/hook-api/atm-hook-extension.md`
- `docs/project-plan.md`
- `docs/requirements.md`
- `docs/architecture.md`

Required tests:

- `pytest test-harness/hooks/`
- `cargo test --workspace`

Success criteria:

- every implementation-facing Claude field is backed by captured fixtures or
  current source-of-truth code/docs/tests
- unknown fields remain explicitly deferred
- later phases define exact code to write, tests required, and success
  criteria

Dependencies:

- Phase 1 completed with captured fixtures and model validation

### Hook Phase 3: Session Foundation

Deliver:

- final trait/context freeze in `sc-hooks-core` / `sc-hooks-sdk`
- `plugins/agent-session-foundation`
- persisted session record keyed by `session_id`
- normalized `agent_state` transitions and `active_pid` ownership rules
- explicit tests proving directory changes do not break later hook correlation
- if the crate is introduced in this sprint, update `docs/architecture.md` §3
  in the same PR to add the crate inventory entry

Write scope:

- `sc-hooks-core/`
- `sc-hooks-sdk/`
- `plugins/agent-session-foundation/`
- same-PR updates to `docs/architecture.md`, `docs/requirements.md`, and
  `docs/project-plan.md`

Required tests:

- unit tests for normalized `agent_state` transitions
- integration tests for session-state persistence keyed by `session_id`
- integration tests proving `SessionStart` in directory A and later lifecycle
  events in directory B still resolve the same session record
- `cargo test --workspace`
- `cargo clippy --all-targets --all-features -- -D warnings`

Success criteria:

- lifecycle code uses only verified inputs
- the session-state schema matches the documented canonical record
- ATM-specific routing stays out of the generic lifecycle crate
- the trait boundary no longer relies on raw `serde_json::Value` alone as the
  only plugin-facing abstraction

Dependencies:

- revised plan and verified field set from Phase 2

### Hook Phase 4: Command And Spawn Gates

Deliver:

- `plugins/agent-spawn-gates`
- `plugins/tool-output-gates`
- named-agent vs background-agent policy
- subagent linkage/tracking
- fenced/blocking JSON tool-utility behavior
- if either crate is introduced in this sprint, update
  `docs/architecture.md` §3 in the same PR to add the crate inventory entries

Write scope:

- `plugins/agent-spawn-gates/`
- `plugins/tool-output-gates/`
- any same-PR doc updates required if the captured schema or blocking contract
  needs clarifying

Required tests:

- direct tests for `tool_name = "Agent"` spawn-gate routing
- tests for named-agent versus background-agent policy outcomes
- tests for subagent linkage fields written into the canonical session-state file
- tests for fenced `json` extraction and schema validation success/failure
- tests proving invalid input returns exact retryable failure reasons
- `cargo test --workspace`
- `cargo clippy --all-targets --all-features -- -D warnings`

Success criteria:

- spawn and tool-blocking behavior is tested directly
- no unverified field is relied on without a later approved schema capture
- block responses explain exactly how the caller can retry successfully
- generic blocking/fenced-JSON policy remains separate from ATM-specific relay
  behavior

Dependencies:

- session record is available for same-agent correlation

### Hook Phase 5: Relay Hooks

Deliver:

- `plugins/atm-extension`
- ATM-specific Bash identity-file handling
- ATM relay tests for `PermissionRequest`, `Stop`, and teammate-idle handling
- `Notification` remains explicitly deferred unless the surface is captured
- if the crate is introduced in this sprint, update `docs/architecture.md` §3
  in the same PR to add the crate inventory entry

Write scope:

- `plugins/atm-extension/`
- ATM-only docs where relay semantics or teammate-idle mapping must be frozen

Required tests:

- tests for ATM identity-file create/delete behavior around `atm` Bash commands
- tests for ATM extension fields on the canonical session-state record
- tests for relay mapping on `PermissionRequest`, `Stop`, and teammate-idle
- `cargo test --workspace`
- `cargo clippy --all-targets --all-features -- -D warnings`

Success criteria:

- ATM behavior is layered on top of generic utilities rather than defining
  them
- failure posture is documented and tested
- `Notification` stays wired but does not block completion of this phase until
  a live payload is captured and promoted

Dependencies:

- session foundation for agent/session lookup

### Hook Phase 6: Cross-Platform Follow-On

Deliver:

- Codex session-identity follow-up plan only as a planning target if the runner
  gains a verified `SessionStart`-equivalent surface
- Gemini follow-on plan only as a planning target after Gemini capture work is
  explicitly approved
- Cursor Agent schema-backed follow-on plan revision only as a planning target
  after a later dedicated Cursor harness pass captures
  `beforeShellExecution`, `beforeMCPExecution`, `beforeReadFile`,
  `afterFileEdit`, and `stop`
- `plugins/cursor-agent-gates` only as a planning target after controllable
  hook request/response schemas are captured
- `plugins/cursor-agent-relay` only as a planning target after informational
  hook payloads are captured
- any future `TeammateIdle`, `PreCompact`, or `PostCompact` hooks only as
  planning targets after their payloads and persistence boundaries are
  verified

Write scope:

- provider follow-on planning docs only
- no runtime crate work without separate approval and provider-specific capture

Required tests:

- docs-only validation plus any provider harness tests explicitly approved for
  that provider follow-on

Success criteria:

- follow-on provider work is represented as schema-backed planning, not guessed
  implementation
- Claude remains the only active runtime baseline until another provider is
  explicitly captured and approved

Dependencies:

- Claude baseline implemented and validated first
- separate approval to expand beyond the Claude-first path

## Per-Hook Notes

### SessionStart / SessionEnd

These should be treated as the authoritative lifecycle pair. The Rust plugins
should preserve the current fail-open posture for ATM relay failures unless the
requirements explicitly change that behavior.

### ATM Bash Identity Pair

The Bash identity hooks should remain command-sensitive:

- no-op for non-`atm` commands
- write on PreToolUse
- delete on PostToolUse

The plugin must not assume hook-only env vars are available in ordinary Bash
tool execution.

### Agent Spawn Gate

This hook is the highest policy-risk item in the set. It should keep its own
crate because it needs:

- frontmatter inspection for agent metadata
- `.atm.toml` team policy lookup
- session-to-identity resolution
- explicit block messaging
- named-agent vs background-agent policy
- subagent linkage and later context-injection compatibility

### Tool Output / JSON Gates

The next implementation pass should not bury tool-output enforcement inside
ATM-only code. Blocking/fenced-JSON behavior should be treated as a generic
utility concern because it applies before ATM routing.

Required policy:

- subagent launches may require fenced JSON input
- the schema may be defined either:
  - inline in the agent prompt file, or
  - in a separate schema file with the same base name as the agent prompt in
    the same directory
- if a schema is present, the spawn gate validates the caller input against it
  before the agent starts
- if validation fails, the hook must block the spawn and return an exact,
  retryable explanation of what was wrong

Recommended lookup order:

1. agent prompt file declares an inline fenced JSON schema
2. sibling schema file such as `<agent-name>.schema.json`

Recommended enforcement behavior:

- require exactly one fenced `json` block for schema-governed agent launches
- parse the fenced block as the machine-readable request envelope
- validate against the agent schema before spawn
- on failure, return:
  - missing fenced block
  - invalid JSON parse error
  - schema validation error with field path and expected type/rule

Why it matters:

- the caller must be able to retry immediately with a corrected message
- blocking must explain the failure precisely, not just say “bad format”

### Prototype Review

Archived prototype branches are still useful as design input, but not as
authoritative implementation.

What they got broadly right:

- session persistence belongs in its own foundational unit
- Bash identity hooks are paired and command-sensitive
- spawn gating is policy-heavy and deserves separation
- relay behavior is mostly narrow and fail-open

What they missed or over-assumed:

- they used ATM-specific crate names as the first-pass architecture instead of
  separating generic utilities from an ATM extension crate
- they relied on payload assumptions later corrected by live capture
  (`tool_name = "Agent"`, `source = "startup"`)
- they did not freeze a normalized `agent_state` model before implementation
- they did not include `PreCompact` in the lifecycle crate
- they treated raw events and normalized idle-state semantics too loosely

### Relay Hooks

The relay hooks are lower design risk than the policy hooks. They mostly need:

- stable session/agent lookup
- event-name mapping
- fail-open ATM emission behavior

## Out Of Scope For Sprint 9

- Codex session lifecycle parity
- frontmatter guard plugins not represented by the current installed Claude hook
  set
- `PostCompact` reinjection until there is a verified hook payload and a clear
  persistence boundary
- promoting Claude-only policy behavior into the generic `sc-hooks` public
  contract without an explicit design decision

## Acceptance For This Plan

This plan is sufficient when:

- the live schema-capture harness is the first implementation gate
- that first implementation gate is explicitly Claude-first
- the post-capture plan revision gate is explicit and mandatory
- the Claude hook set is documented per-platform rather than mixed with Codex
  assumptions
- the session correlation model is explicit and cwd-independent
- each captured Claude baseline behavior plus the unresolved `Notification`
  surface has a hook type, consumed inputs, action summary, crate assignment,
  and sequencing position
- platform gaps are called out honestly instead of hidden inside implementation
  tasks
- Cursor remains documented without being turned into an immediate dev blocker
