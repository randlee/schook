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

- Claude: `SessionStart`, `SessionEnd`, `PreToolUse(Bash)`, `PreToolUse(Task)`,
  `PostToolUse(Bash)`, `PermissionRequest`, `Stop`, `Notification(idle_prompt)`

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

Pre-gate prototype rule:

- if a branch temporarily carries hook implementation code before Hook Phases 1
  and 2 are complete, that branch must be labeled a pre-gate prototype and
  blocked from merge until the harness capture and post-capture plan-revision
  gates are accepted

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
- verified observed `source` values are `init` and `compact`
- `resume` is not currently a verified literal payload value

Implementation-planning rule:

- any internal fresh/resume/compact classification must be derived from
  verified evidence and persisted state, not documented as a Claude wire enum
  unless the harness proves it

## Hook Inventory

| Hook behavior | Claude surface | Matcher/event | Python reference | Input fields consumed | Action logic | `schook` fit | Recommended crate |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Session start | `SessionStart` | `*` | `session-start.py` | `session_id`, `source`, `.atm.toml`, `ATM_TEAM`, `ATM_IDENTITY` | announce session, emit ATM start event, persist session record | fits current lifecycle hook model | `plugins/atm-session-lifecycle` |
| Session end | `SessionEnd` | `*` | `session-end.py` | `session_id`, `.atm.toml` core routing | emit ATM end event, delete session record | fits current lifecycle hook model | `plugins/atm-session-lifecycle` |
| ATM identity write | `PreToolUse` | `Bash` | `atm-identity-write.py` | `tool_input.command`, `session_id`, `.atm.toml`, `ATM_*` env | if command invokes `atm`, write temp identity file | fits current tool hook model | `plugins/atm-bash-identity` |
| ATM identity cleanup | `PostToolUse` | `Bash` | `atm-identity-cleanup.py` | routing context only | remove temp identity file from paired bash invocation | fits current tool hook model | `plugins/atm-bash-identity` |
| Agent spawn gate | `PreToolUse` | `Task` | `gate-agent-spawns.py` | `tool_input.subagent_type`, `tool_input.name`, `tool_input.team_name`, `session_id`, team config | enforce named-teammate and team spawn policy | fits current tool hook model | `plugins/gate-agent-spawns` |
| Idle relay | `Notification` | `idle_prompt` | `notification-idle-relay.py` | `session_id`, team/agent routing fields | emit ATM idle heartbeat | fits current notification hook model | `plugins/atm-state-relay` |
| Permission relay | `PermissionRequest` | `*` | `permission-request-relay.py` | `session_id`, `tool_name`, `tool_input`, team/agent routing fields | emit ATM permission-request event | fits current lifecycle-style hook model | `plugins/atm-state-relay` |
| Stop relay | `Stop` | `*` | `stop-relay.py` | `session_id`, team/agent routing fields | emit ATM stop/idle event | fits current lifecycle-style hook model | `plugins/atm-state-relay` |

## Protocol Compatibility Analysis

### Fully Compatible With Current `schook` Contract

These behaviors fit the currently documented hook taxonomy and stdin payload
model without requiring a new host protocol:

- `SessionStart`
- `SessionEnd`
- `PreToolUse/Bash`
- `PostToolUse/Bash`
- `PreToolUse/Task`
- `Notification/idle_prompt`
- `PermissionRequest`
- `Stop`

Why they fit:

- the host already supports these hook names and matcher rules in
  `docs/protocol-contract.md`
- payload passthrough already gives plugins access to nested tool input fields
- lifecycle hooks already allow `*` as the matcher posture for these events

### Design Gaps To Keep Explicit

- Claude-specific routing context such as `.atm.toml`, `ATM_TEAM`, and
  `ATM_IDENTITY` is external policy input, not part of the generic `schook`
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

- `plugins/atm-session-lifecycle`
- `plugins/atm-bash-identity`
- `plugins/gate-agent-spawns`
- `plugins/atm-state-relay`

These are planning targets only. Until implementation lands with tests and the
same-PR architecture update, they must be treated as scaffold/reference-only
proposals rather than existing source inventory.

Inventory rule:

- any implementation sprint that adds one of these crates must update
  `docs/architecture.md` §3 crate inventory in the same PR

- `plugins/atm-session-lifecycle`
  - owns SessionStart and SessionEnd
  - owns persistent session record read/write
- `plugins/atm-bash-identity`
  - owns PreToolUse/PostToolUse Bash pair for `atm` command identity files
- `plugins/gate-agent-spawns`
  - owns Task spawn policy enforcement
- `plugins/atm-state-relay`
  - owns Notification, PermissionRequest, and Stop event relay behaviors

Recommended supporting test infrastructure:

- `test-harness/hooks/` for provider adapters, capture fixtures, models, and
  schema-drift tests

Why this split:

- session lifecycle is stateful and foundational
- Bash identity hooks are paired and should share temp-file behavior
- spawn gating is policy-heavy and should not be mixed with generic relays
- notification and lifecycle relays are narrow event-forwarding behaviors

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

Dependencies:

- Phase 0 review and acceptance complete (umbrella plan accepted; `test-harness/hooks/README.md` accepted as harness contract)

### Hook Phase 2: Plan Revision From Captured Schema

Deliver:

- revised hook API docs with only verified fields
- revised S9 plan for the remaining Claude implementation work
- explicit deferral markers for anything still not captured or source-backed

Dependencies:

- Phase 1 completed with captured fixtures and model validation

### Hook Phase 3: Session Foundation

Deliver:

- `plugins/atm-session-lifecycle`
- persisted session record keyed by `session_id`
- explicit tests proving directory changes do not break later hook correlation
- if the crate is introduced in this sprint, update `docs/architecture.md` §3
  in the same PR to add the crate inventory entry

Dependencies:

- revised plan and verified field set from Phase 2

Current branch posture:

- `feature/s9-p3-session-foundation` is a pre-gate prototype branch only
- it remains blocked from merge until Hook Phase 1 and Hook Phase 2 complete

### Hook Phase 4: Command And Spawn Gates

Deliver:

- `plugins/atm-bash-identity`
- `plugins/gate-agent-spawns`
- if either crate is introduced in this sprint, update
  `docs/architecture.md` §3 in the same PR to add the crate inventory entries

Dependencies:

- session record is available for same-agent correlation

### Hook Phase 5: Relay Hooks

Deliver:

- `plugins/atm-state-relay`
- relay tests for Notification, PermissionRequest, and Stop payload handling
- if the crate is introduced in this sprint, update `docs/architecture.md` §3
  in the same PR to add the crate inventory entry

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
- promoting Claude-only policy behavior into the generic `schook` public
  contract without an explicit design decision

## Acceptance For This Plan

This plan is sufficient when:

- the live schema-capture harness is the first implementation gate
- that first implementation gate is explicitly Claude-first
- the post-capture plan revision gate is explicit and mandatory
- the Claude hook set is documented per-platform rather than mixed with Codex
  assumptions
- the session correlation model is explicit and cwd-independent
- each of the eight current Claude ATM hook behaviors has a hook type, consumed
  inputs, action summary, crate assignment, and sequencing position
- platform gaps are called out honestly instead of hidden inside implementation
  tasks
- Cursor remains documented without being turned into an immediate dev blocker
