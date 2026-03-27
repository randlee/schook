# Claude Hook API

## Purpose

This document records the currently verified Claude Code hook surfaces that
`schook` can target. It is a platform reference, not a generic hook contract.

The source-of-truth inputs for this document are:

- installed hook scripts under `/Users/randlee/.claude/scripts/`
- Claude-specific notes in
  `/Users/randlee/Documents/github/synaptic-canvas/docs/agent-tool-use-best-practices.md`
- Claude-specific notes in
  `/Users/randlee/Documents/github/synaptic-canvas/docs/agent-teams-best-practices.md`
- current ATM hook docs, scripts, tests, and session fallback code in
  `/Users/randlee/Documents/github/agent-team-mail`
- real Claude Haiku captures under `test-harness/hooks/claude/captures/raw/`
  for every schema or behavior claim marked captured below

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
- teammate/background spawn via `PreToolUse`
- `PostToolUse(Bash)`
- `PermissionRequest`
- `Stop`

The live harness has not yet captured:

- `Notification`

Verified `SessionStart.source` values from live capture:

- `startup`
- `compact`

What is not verified today:

- a full upstream Claude JSON schema for all hook payloads
- a confirmed literal `source = "resume"` value
- cwd/root/agent metadata as guaranteed `SessionStart` payload fields across
  all launches
- parent/subagent/session lineage fields in Claude hook payloads
- a live `Notification` payload in this harness environment

## Session Correlation Model

Claude hook calls should treat identity and context as separate concerns.

Current verified anchor:

1. SessionStart-captured `session_id`
2. hook subprocess parent PID (`PPID`) as a same-process cross-check
3. `ATM_TEAM` + `ATM_IDENTITY` only as routing labels, not as a unique instance
   key

Rules:

- directory changes do not change identity
- compaction does not change `session_id`
- a fresh Claude process creates a new `session_id` and a new PPID
- later hooks should read persisted session state rather than trying to infer
  identity from current working directory

Current verified ATM-backed persistent record fields:

- key by `session_id`
- store `session_id`, `team`, `identity`, `pid`, `created_at`, `updated_at`
- preserve `created_at` on re-fire for the same `session_id`
- refresh `updated_at` when the session file is touched again

This is a statement of the current ATM implementation, not a claim that the
future `schook` base record must stay identical.

## Normalized Agent State Model

Raw hook events and normalized runtime state must remain separate.

Recommended normalized `agent_state` enum:

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

Recommended adjacent tracking fields:

- `last_hook_event`
- `last_hook_event_at`

Observed and planned transitions:

- `SessionStart(source="startup")` -> `starting`
- active tool use / active turn execution -> `busy`
- `PermissionRequest` -> `awaiting_permission`
- `PreCompact` -> `compacting`
- `SessionStart(source="compact")` -> compact-return startup for the same
  `session_id`
- `Stop` -> `idle`
- `SessionEnd` -> `ended`

Observed negative case:

- `SessionStart` alone did not emit `Stop` in live Haiku capture when no turn
  occurred

## Captured Hook Schemas

Only the following surfaces were captured in the live Haiku pass. Their schema
examples are copied from `test-harness/hooks/claude/captures/raw/` and should
be treated as current observed shape, not a vendor-guaranteed contract.

### `SessionStart`

Observed fields:

- `cwd`
- `hook_event_name`
- `model` (present in some captures)
- `session_id`
- `source`
- `transcript_path`

Observed values:

- `hook_event_name = "SessionStart"`
- `source = "startup"` on a fresh start
- `source = "compact"` after `/compact`

Example observed shape:

```json
{
  "cwd": "/path/to/worktree",
  "hook_event_name": "SessionStart",
  "model": "claude-haiku-4-5-20251001",
  "session_id": "<uuid>",
  "source": "startup|compact",
  "transcript_path": "/Users/.../.claude/projects/...jsonl"
}
```

### `SessionEnd`

Observed fields:

- `cwd`
- `hook_event_name`
- `reason`
- `session_id`
- `transcript_path`

Observed values:

- `hook_event_name = "SessionEnd"`
- `reason = "other"`
- `reason = "prompt_input_exit"`

Example observed shape:

```json
{
  "cwd": "/path/to/worktree",
  "hook_event_name": "SessionEnd",
  "reason": "other|prompt_input_exit",
  "session_id": "<uuid>",
  "transcript_path": "/Users/.../.claude/projects/...jsonl"
}
```

### `PreCompact`

Observed fields:

- `custom_instructions`
- `cwd`
- `hook_event_name`
- `session_id`
- `transcript_path`
- `trigger`

Observed values:

- `hook_event_name = "PreCompact"`
- `trigger = "manual"`

Example observed shape:

```json
{
  "custom_instructions": "",
  "cwd": "/path/to/worktree",
  "hook_event_name": "PreCompact",
  "session_id": "<uuid>",
  "transcript_path": "/Users/.../.claude/projects/...jsonl",
  "trigger": "manual"
}
```

### `PreToolUse(Bash)`

Observed fields:

- `cwd`
- `hook_event_name`
- `permission_mode`
- `session_id`
- `tool_input.command`
- `tool_input.description`
- `tool_name`
- `tool_use_id`
- `transcript_path`

Observed values:

- `hook_event_name = "PreToolUse"`
- `tool_name = "Bash"`

Example observed shape:

```json
{
  "cwd": "/path/to/worktree",
  "hook_event_name": "PreToolUse",
  "permission_mode": "default",
  "session_id": "<uuid>",
  "tool_input": {
    "command": "pwd",
    "description": "Print current working directory"
  },
  "tool_name": "Bash",
  "tool_use_id": "<tool-id>",
  "transcript_path": "/Users/.../.claude/projects/...jsonl"
}
```

### teammate/background spawn via `PreToolUse`

Observed fields:

- `cwd`
- `hook_event_name`
- `permission_mode`
- `session_id`
- `tool_input.description`
- `tool_input.name`
- `tool_input.prompt`
- `tool_input.run_in_background`
- `tool_name`
- `tool_use_id`
- `transcript_path`

Observed values:

- `hook_event_name = "PreToolUse"`
- `tool_name = "Agent"`

Important note:

- the current Haiku capture names this surface `Agent`, not `Task`

Example observed shape:

```json
{
  "cwd": "/path/to/worktree",
  "hook_event_name": "PreToolUse",
  "permission_mode": "default",
  "session_id": "<uuid>",
  "tool_input": {
    "description": "<string>",
    "name": "<string>",
    "prompt": "<string>",
    "run_in_background": true
  },
  "tool_name": "Agent",
  "tool_use_id": "<tool-id>",
  "transcript_path": "/Users/.../.claude/projects/...jsonl"
}
```

### `PostToolUse(Bash)`

Observed fields:

- `cwd`
- `hook_event_name`
- `permission_mode`
- `session_id`
- `tool_input.command`
- `tool_input.description`
- `tool_name`
- `tool_response.interrupted`
- `tool_response.isImage`
- `tool_response.noOutputExpected`
- `tool_response.stderr`
- `tool_response.stdout`
- `tool_use_id`
- `transcript_path`

Observed values:

- `hook_event_name = "PostToolUse"`
- `tool_name = "Bash"`

Example observed shape:

```json
{
  "cwd": "/path/to/worktree",
  "hook_event_name": "PostToolUse",
  "permission_mode": "default",
  "session_id": "<uuid>",
  "tool_input": {
    "command": "pwd",
    "description": "Print current working directory"
  },
  "tool_name": "Bash",
  "tool_response": {
    "interrupted": false,
    "isImage": false,
    "noOutputExpected": false,
    "stderr": "",
    "stdout": "/path/to/worktree"
  },
  "tool_use_id": "<tool-id>",
  "transcript_path": "/Users/.../.claude/projects/...jsonl"
}
```

### `PermissionRequest`

Observed fields:

- `cwd`
- `hook_event_name`
- `permission_mode`
- `permission_suggestions`
- `session_id`
- `tool_input`
- `tool_name`
- `transcript_path`

Observed values:

- `hook_event_name = "PermissionRequest"`
- `tool_name = "Write"`
- `tool_name = "Bash"`

Example observed shape:

```json
{
  "cwd": "/path/to/worktree",
  "hook_event_name": "PermissionRequest",
  "permission_mode": "default",
  "permission_suggestions": [
    {
      "destination": "session",
      "mode": "acceptEdits",
      "type": "setMode"
    }
  ],
  "session_id": "<uuid>",
  "tool_input": {
    "...": "provider-specific tool payload"
  },
  "tool_name": "Write|Bash",
  "transcript_path": "/Users/.../.claude/projects/...jsonl"
}
```

### `Stop`

Observed fields:

- `cwd`
- `hook_event_name`
- `last_assistant_message`
- `permission_mode`
- `session_id`
- `stop_hook_active`
- `transcript_path`

Observed values:

- `hook_event_name = "Stop"`
- `stop_hook_active = false`

Example observed shape:

```json
{
  "cwd": "/path/to/worktree",
  "hook_event_name": "Stop",
  "last_assistant_message": "<assistant text>",
  "permission_mode": "default",
  "session_id": "<uuid>",
  "stop_hook_active": false,
  "transcript_path": "/Users/.../.claude/projects/...jsonl"
}
```

### `Notification`

Current status:

- not captured in this harness environment

What we tried:

- manual interactive Claude launches with harness-local settings
- `Notification` wired first with `matcher = "idle_prompt"`
- then rewired with `matcher = ""`
- repeated long-idle waits after startup
- repeated long-idle waits after completed turns
- repeated long-idle waits after permission and compact scenarios

Observed result:

- no `notification` payload was captured in this environment despite repeated
  long-idle runs

## Verified Claude Hook Behaviors

| Behavior | Claude surface | Current script | Fields consumed | Current side effects | Planned `schook` mapping |
| --- | --- | --- | --- | --- | --- |
| Session start | `SessionStart` | `session-start.py` | `session_id`, `source`, ATM repo/env context | prints `SESSION_ID=...`, emits ATM `session_start`, writes session record | `SessionStart` sync plugin |
| Session end | `SessionEnd` | `session-end.py` | `session_id`, `.atm.toml` core routing | emits ATM `session_end`, removes session record | `SessionEnd` sync plugin |
| Pre-compact | `PreCompact` | live harness capture now proves the raw surface | `session_id`, `trigger`, `custom_instructions`, transcript/cwd context | pre-restart compact lifecycle signal | `PreCompact` sync plugin |
| ATM identity write | `PreToolUse` on `Bash` | `atm-identity-write.py` | `tool_input.command`, `session_id`, ATM repo/env context | writes temp identity file for `atm` commands only | `PreToolUse/Bash` sync plugin |
| ATM identity cleanup | `PostToolUse` on `Bash` | `atm-identity-cleanup.py` | hook routing context only | deletes temp identity file written by the paired pre-hook | `PostToolUse/Bash` sync plugin |
| Agent spawn gate | logical teammate/background spawn surface | `gate-agent-spawns.py` | `tool_input.subagent_type`, `name`, `team_name`, `session_id`, team config | blocks unsafe spawns or mismatched team usage | logical spawn-gate sync plugin; current Haiku payload arrives as `tool_name = "Agent"` |
| Idle notification relay | `Notification` | `notification-idle-relay.py` | intended to use session/team routing fields | emits ATM idle heartbeat | surface stays wired; live payload unresolved in this harness environment |
| Permission relay | `PermissionRequest` | `permission-request-relay.py` | `session_id`, `tool_name`, `tool_input`, team/agent routing | emits ATM permission-request event | `PermissionRequest` sync plugin |
| Stop relay | `Stop` | `stop-relay.py` | `session_id`, team/agent routing | emits ATM stop/idle event | `Stop` sync plugin |

Adjacent but not part of the current captured baseline:

- `atm-hook-relay.py` is a Codex notify relay, not a Claude Code hook
- `teammate-idle-relay.py` is separate team-state plumbing and should be planned
  only if the runtime elevates `TeammateIdle`

Live harness note:

- long-lived teammate agents may transition to normalized `idle` through a
  separate raw `teammate_idle` event in ATM plumbing rather than `Stop`

## Design Implications For `schook`

- `SessionStart` is the authoritative place to capture `session_id` for later
  hook calls
- the `source` field should be stored as raw payload evidence; any
  fresh/resume/compact classification must be documented as internal logic, not
  as a claimed Claude wire enum
- `Stop` is the reliable observed transition back to normalized `idle`
- the runtime should preserve session records across directory changes
- Bash-specific hooks need command-sensitive behavior, not just hook-type
  matching
- teammate/background spawn gating is policy-heavy and should remain separate
  from generic ATM relays; current Haiku capture names that surface `Agent`
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
- `Notification` remains unresolved in this harness environment; the hook
  should stay wired, but implementation should not depend on a captured
  payload until the surface is reproduced
