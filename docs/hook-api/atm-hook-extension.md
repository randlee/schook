# ATM Hook Extension

## Purpose

This document records the currently verified ATM-specific extension behavior
that sits on top of Claude hook execution today.

This is not a generic `schook` contract. It is the current ATM baseline drawn
from `agent-team-mail`.

## Current Source Of Truth

- `/Users/randlee/Documents/github/agent-team-mail/docs/claude-hook-strategy.md`
- `/Users/randlee/Documents/github/agent-team-mail/docs/agent-teams-hooks.md`
- `/Users/randlee/Documents/github/agent-team-mail/docs/requirements.md`
- `/Users/randlee/Documents/github/agent-team-mail/.claude/scripts/session-start.py`
- `/Users/randlee/Documents/github/agent-team-mail/.claude/scripts/session-end.py`
- `/Users/randlee/Documents/github/agent-team-mail/tests/hook-scripts/test_session_start.py`
- `/Users/randlee/Documents/github/agent-team-mail/tests/hook-scripts/test_session_end.py`
- `/Users/randlee/Documents/github/agent-team-mail/crates/atm/src/util/hook_identity.rs`

## Verified ATM Session File

Current ATM `SessionStart` writes a session file at:

```text
{ATM_HOME}/.claude/teams/<team>/sessions/<session_id>.json
```

Current verified file shape:

```json
{
  "session_id": "<uuid>",
  "team": "<default_team>",
  "identity": "<identity>",
  "pid": 12345,
  "created_at": 1743120000.0,
  "updated_at": 1743120000.0
}
```

Rules verified today:

- `pid` is `os.getppid()`
- file is written only when `session_id`, team, and identity are all resolved
- `created_at` is preserved when the same `session_id` re-fires
- `updated_at` is refreshed on re-fire
- files are treated as stale after the configured TTL window
- CLI fallback resolves `session_id` by scanning these files for `team +
  identity`, subject to TTL and live-pid checks

## Verified ATM Routing Context

Current ATM routing context comes from:

1. `.atm.toml` in the working directory
2. `ATM_TEAM` and `ATM_IDENTITY` environment overrides

Current verified hook behavior:

- non-ATM sessions no-op when neither repo config nor env routing is present
- env routing may override repo routing
- a warning is emitted when `ATM_TEAM` overrides `.atm.toml` default team
- non-lead sessions are prevented from claiming a reserved `leadSessionId`

## Verified ATM Hook Responsibilities

| Hook | Current ATM behavior |
| --- | --- |
| `SessionStart` | announces `SESSION_ID`, resolves ATM routing, emits `session_start`, writes session file |
| `SessionEnd` | emits `session_end`, deletes current session file |
| `PreToolUse(Bash)` | writes temp hook identity file for `atm` commands only |
| `PostToolUse(Bash)` | removes temp hook identity file |
| `PreToolUse(Task)` | enforces team-aware spawn policy |
| `Notification(idle_prompt)` | relays idle heartbeat |
| `PermissionRequest` | relays blocked-on-permission state |
| `Stop` | relays idle/turn-stop state |

## What Is Not Yet Source-Of-Truth ATM Behavior

These may be good design targets, but they are not current verified ATM
implementation facts and must not be promoted as current behavior without new
evidence:

- a richer base session record carrying `agent_type`, `git_root_dir`,
  `parent_session_id`, or `subagent_id`
- a documented ATM JSON Schema for all Claude hook payloads
- Pydantic hook models as the active ATM validation layer
- a guaranteed literal `source = "resume"` payload value from Claude
- a complete subagent lifecycle schema for Task-created agents

## Planning Rule For `schook`

`schook` may extend this ATM baseline, but only after:

1. the live hook schema harness captures real payloads,
2. the resulting models are validated, and
3. the plan is revised from captured evidence before code is written.
