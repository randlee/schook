# Sprint 9 Plugin Hook Plan

## Goal

Plan the Rust plugin implementation of the current Claude ATM hook set using
the installed Python hooks under `/Users/randlee/.claude/scripts/` as the
behavioral reference.

This sprint is planning-only. It does not change runtime code.

## Planning Baseline

Durable platform references for this plan live in:

- `docs/hook-api/claude-hook-api.md`
- `docs/hook-api/codex-hook-api.md`

Sprint 9 focuses on the verified Claude hook set. Codex is documented as a
separate platform reference, but it is not the implementation baseline for
this sprint.

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

## Step 1: Hook Schema Validation Harness

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

- `test-harness/hooks/providers/claude/`
- `test-harness/hooks/providers/codex/`
- `test-harness/hooks/providers/gemini/`
- `test-harness/hooks/providers/cursor-agent/`
- `test-harness/hooks/models/`
- `test-harness/hooks/fixtures/`
- `test-harness/hooks/reports/`
- `test-harness/hooks/scripts/`

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

Minimum first-pass capture matrix:

- Claude: `SessionStart`, `SessionEnd`, `PreToolUse(Bash)`, `PreToolUse(Task)`,
  `PostToolUse(Bash)`, `PermissionRequest`, `Stop`, `Notification(idle_prompt)`
- Codex: document and capture the currently available hook/notify surfaces
- Gemini: document and capture the currently available hook surfaces
- Cursor Agent: document and capture the currently available lifecycle hooks

Acceptance for this first step:

- the harness can launch each installed provider in a minimal scripted run
- raw hook payloads are captured and stored by provider/hook type
- provider-specific models validate the captured payloads
- CI fails on breaking hook schema drift
- the S9 implementation plan only promotes fields that are backed by captured
  payload evidence or existing source-of-truth docs

## Session And Agent Correlation Model

The session/agent key must survive directory changes and compact/resume.

Identity anchor:

1. `session_id` captured during `SessionStart`
2. hook subprocess parent PID (`PPID`) as a same-process cross-check
3. `ATM_TEAM` + `ATM_IDENTITY` as routing labels only

Required behavior:

- persist a per-session record keyed by `session_id`
- store `pid`, `team`, `identity`, and timestamps
- allow later hooks to recover session/agent context from that record even when
  current working directory changes
- never use cwd as the identity key

This matches the guidance supplied by `team-lead`: SessionStart-captured
`session_id` is the primary key, PPID is the cross-check, and routing labels
are not unique-instance identifiers.

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

## Recommended Crate Layout

Keep policy-heavy and stateful hooks separated from generic relays.

- `plugins/atm-session-lifecycle`
  - owns SessionStart and SessionEnd
  - owns persistent session record read/write
- `plugins/atm-bash-identity`
  - owns PreToolUse/PostToolUse Bash pair for `atm` command identity files
- `plugins/gate-agent-spawns`
  - owns Task spawn policy enforcement
- `plugins/atm-state-relay`
  - owns Notification, PermissionRequest, and Stop event relay behaviors

Why this split:

- session lifecycle is stateful and foundational
- Bash identity hooks are paired and should share temp-file behavior
- spawn gating is policy-heavy and should not be mixed with generic relays
- notification and lifecycle relays are narrow event-forwarding behaviors

## Sprint Sequencing

### Phase 1: Live Schema Capture And Drift Validation

Deliver:

- provider launch adapters for `claude`, `codex`, `gemini`, and `cursor-agent`
- provider-specific hook payload models
- fixture capture and validation tests
- drift-report output for unknown-field additions and breaking schema changes

Dependencies:

- none; this is the first gate

### Phase 2: Session Foundation

Deliver:

- `plugins/atm-session-lifecycle`
- persisted session record keyed by `session_id`
- explicit tests proving directory changes do not break later hook correlation

Dependencies:

- hook payload evidence from Phase 1

### Phase 3: Command And Spawn Gates

Deliver:

- `plugins/atm-bash-identity`
- `plugins/gate-agent-spawns`

Dependencies:

- session record is available for same-agent correlation

### Phase 4: Relay Hooks

Deliver:

- `plugins/atm-state-relay`
- relay tests for Notification, PermissionRequest, and Stop payload handling

Dependencies:

- session foundation for agent/session lookup

### Phase 5: Cross-Platform Follow-On

Deliver:

- Codex session-identity follow-up plan if the runner gains a verified
  SessionStart-equivalent surface
- any future `TeammateIdle`, `PreCompact`, or `PostCompact` hooks only after
  their payloads and persistence boundaries are verified

Dependencies:

- Claude baseline implemented and validated first

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
- the Claude hook set is documented per-platform rather than mixed with Codex
  assumptions
- the session correlation model is explicit and cwd-independent
- each of the eight current Claude ATM hook behaviors has a hook type, consumed
  inputs, action summary, crate assignment, and sequencing position
- platform gaps are called out honestly instead of hidden inside implementation
  tasks
