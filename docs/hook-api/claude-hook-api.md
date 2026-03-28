# Claude Hook API

## Purpose

This document records the currently verified Claude Code hook surfaces that
`schook` can target. It is a platform reference, not a generic hook contract.

The source-of-truth inputs for this document are:

- installed Claude hook scripts
- Claude-specific notes in the `synaptic-canvas` repo:
  - `docs/agent-tool-use-best-practices.md`
  - `docs/agent-teams-best-practices.md`
- current ATM hook docs, scripts, tests, and session fallback code in the
  `agent-team-mail` repo
- real Claude Haiku captures under `test-harness/hooks/claude/captures/raw/`
  for claims marked captured in this document

## Platform Rules

- Claude Code uses `settings.json` hook registration for project and global
  hooks.
- Claude Code does not honor agent frontmatter hooks as a reliable execution
  surface.
- PreToolUse hooks in `settings.json` work for both in-process and tmux
  teammate modes.
- Frontmatter hooks should not be treated as a Claude-compatible baseline.

## Path And Environment Rules

- hook working directory is not a stable identity signal
- `CLAUDE_PROJECT_DIR` is the correct project-root anchor when available
- `CLAUDE_PLUGIN_ROOT` is available in hook context, not in ordinary Bash tool
  execution
- relative hook paths are not reliable because Claude may change the current
  directory during a session

## Current Schema Baseline

The live harness now verifies actual Claude Haiku payloads for these surfaces:

- `SessionStart`
- `SessionEnd`
- `PreCompact`
- `PreToolUse(Bash)`
- teammate/background spawn via `PreToolUse(Agent)`
- `PostToolUse(Bash)`
- `PermissionRequest`
- `Stop`

`Notification(idle_prompt)` remains wired in the harness but unresolved in
local capture.

For `SessionStart`, the following is verified from live hook behavior:

- payload field names used by the live hook:
  - `session_id`
  - `source`
- verified observed `source` values:
  - `startup`
  - `compact`
  - `resume`
  - `clear`
- capture evidence:
  - `startup` and `compact` were captured in PR `#42`
  - `resume` and `clear` were captured in PR `#44`
- current script behavior:
  - `source == "compact"` -> compact-return message
  - any other value -> fresh-or-unknown start message

What is not verified today:

- a full upstream Claude JSON schema for all hook payloads
- cwd/root/agent metadata as guaranteed `SessionStart` payload fields across
  all launches
- parent/subagent/session lineage fields in Claude hook payloads
- a live `Notification` payload in this harness environment

What is verified by the committed Sprint 9 Phase 3 schema/tooling:

- `SessionStart` also carries optional `model`
- `SessionEnd` may carry optional `reason`
- `PreToolUse(Bash)` and `PreToolUse(Agent)` carry optional
  `permission_mode` and `tool_use_id`
- `PreToolUse(Agent).tool_input` carries verified `description`, `name`, and
  `run_in_background`
- `PostToolUse(Bash).tool_response` is currently observed with
  `stdout`, `stderr`, `interrupted`, `isImage`, and `noOutputExpected`
- `PermissionRequest` may carry optional `permission_mode` and
  `permission_suggestions`
- `Stop` may carry optional `permission_mode` and `last_assistant_message`

Deferred in the Phase 3 schema because the model allows them for future drift
comparison but the current approved fixture set does not prove them yet:

- `PreToolUse(Agent).tool_input.subagent_type`
- `PreToolUse(Agent).tool_input.team_name`
- `PostToolUse(Bash).tool_response.output`
- `PostToolUse(Bash).tool_response.error`

## Session Correlation Model

Claude hook calls should treat identity and context as separate concerns.

Current verified anchor:

1. SessionStart-captured `session_id`
2. `CLAUDE_PROJECT_DIR` as the source-backed project-root anchor when present
3. `ATM_TEAM` + `ATM_IDENTITY` only as routing labels, not as a unique instance
   key

Rules:

- directory changes do not change identity
- compaction does not change `session_id`
- `/clear` ends the prior session and starts a new `session_id`
- later hooks should read persisted session state rather than trying to infer
  identity from current working directory
- `PPID` can be used as a local diagnostic cross-check, but it is not the
  persisted identity key in the verified Sprint 9 plan

Current verified ATM-backed persistent record fields:

- key by `session_id`
- store `session_id`, `team`, `identity`, `pid`, `created_at`, `updated_at`
- preserve `created_at` on re-fire for the same `session_id`
- refresh `updated_at` when the session file is touched again

This is a statement of the current ATM implementation, not a claim that the
future `schook` base record must stay identical.

## Verified Claude Hook Behaviors

| Behavior | Claude surface | Current script | Fields consumed | Current side effects | Planned `schook` mapping |
| --- | --- | --- | --- | --- | --- |
| Session start | `SessionStart` | `session-start.py` | `session_id`, `source`, ATM repo/env context | prints `SESSION_ID=...`, emits ATM `session_start`, writes session record | `SessionStart` sync plugin |
| Session end | `SessionEnd` | `session-end.py` | `session_id`, `.atm.toml` core routing | emits ATM `session_end`, removes session record | `SessionEnd` sync plugin |
| ATM identity write | `PreToolUse` on `Bash` | `atm-identity-write.py` | `tool_input.command`, `session_id`, ATM repo/env context | writes temp identity file for `atm` commands only | `PreToolUse/Bash` sync plugin |
| ATM identity cleanup | `PostToolUse` on `Bash` | `atm-identity-cleanup.py` | hook routing context only | deletes temp identity file written by the paired pre-hook | `PostToolUse/Bash` sync plugin |
| Agent spawn gate | `PreToolUse` on `Agent` | `gate-agent-spawns.py` | `tool_input.subagent_type`, `name`, `team_name`, `session_id`, team config | blocks unsafe spawns or mismatched team usage | `PreToolUse/Agent` sync plugin |
| Idle notification relay | `Notification` on `idle_prompt` | `notification-idle-relay.py` | `session_id`, team/agent routing fields | emits ATM idle heartbeat | `Notification/idle_prompt` async-safe sync plugin |
| Permission relay | `PermissionRequest` | `permission-request-relay.py` | `session_id`, `tool_name`, `tool_input`, team/agent routing | emits ATM permission-request event | `PermissionRequest` sync plugin |
| Stop relay | `Stop` | `stop-relay.py` | `session_id`, team/agent routing | emits ATM stop/idle event | `Stop` sync plugin |

Adjacent but not part of the current eight-hook baseline:

- `atm-hook-relay.py` is a Codex notify relay, not a Claude Code hook
- `teammate-idle-relay.py` is separate team-state plumbing and should be planned
  only if the runtime elevates `TeammateIdle`

## Design Implications For `schook`

- `SessionStart` is the authoritative place to capture `session_id` for later
  hook calls
- the `source` field should be stored as raw payload evidence; any
  fresh/resume/compact classification must be documented as internal logic, not
  as a claimed Claude wire enum
- the runtime should preserve session records across directory changes
- `SessionStart(source="resume")` is now captured evidence, not just a
  documented provider claim
- `/clear` produces `SessionEnd(reason="clear")` followed by a new
  `SessionStart(source="clear")` and a new `session_id`
- Bash-specific hooks need command-sensitive behavior, not just hook-type
  matching
- Agent spawn gating is policy-heavy and should remain separate from generic ATM
  relays
- `Notification(idle_prompt)` stays part of the documented Claude surface, but
  should be labeled wired-but-unresolved in the harness until a local capture
  actually lands
- lifecycle and relay hooks are fail-open today; if `schook` changes that
  posture, the change must be explicit in requirements and protocol docs
- no `schook` code should be written against inferred Claude payload fields that
  are not backed by source-of-truth docs, scripts, tests, or captured harness
  fixtures

## Current Platform Gaps

- Claude hook payloads are only partially documented by the vendor, so some
  field names are verified from live scripts rather than a formal upstream
  schema
- `agent-team-mail` does not currently appear to use Pydantic models as the
  Claude hook source of truth; the current baseline is docs + scripts + tests +
  Rust fallback code
- `CLAUDE_SESSION_ID` is stable in the parent Claude process but is not
  directly available to bash subprocesses; `SessionStart` capture is required
- hook env var availability differs sharply between hook execution and ordinary
  Bash tool execution, so plugins must not assume hook-only env vars in
  non-hook code paths
