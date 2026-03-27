# Gemini CLI Hook API

## Purpose

This document records the verified Gemini CLI hook surfaces, their mapping to
Claude Code equivalents, and the differences that affect cross-platform plugin
portability.

Source of truth inputs:

- `~/.gemini/settings.json` — live hook registrations on this machine
- `@google/gemini-cli/dist/src/commands/hooks/migrate.js` — canonical
  Claude→Gemini event and tool name mapping
- `@google/gemini-cli/dist/src/config/settingsSchema.js` — full hook type
  schema definitions

## Platform Rules

- Gemini uses `~/.gemini/settings.json` (user) and `.gemini/settings.json`
  (project) for hook registration.
- Hook format is identical to Claude Code: `type: "command"`, `command`,
  optional `timeout`.
- `gemini hooks migrate --from-claude` automates migration of Claude Code
  `settings.json` hooks to Gemini format.
- `CLAUDE_PROJECT_DIR` must be replaced with `GEMINI_PROJECT_DIR` in migrated
  hook commands.
- Gemini has no sub-agent concept; `SubAgentStop` maps to `AfterAgent`.

## Event Name Mapping (Claude → Gemini)

This mapping is from `migrate.js` — it is the canonical cross-platform translation.

| Claude Code event | Gemini event | Notes |
| --- | --- | --- |
| `SessionStart` | `SessionStart` | exact match |
| `SessionEnd` | `SessionEnd` | exact match |
| `PreToolUse` | `BeforeTool` | renamed |
| `PostToolUse` | `AfterTool` | renamed |
| `UserPromptSubmit` | `BeforeAgent` | renamed |
| `Stop` | `AfterAgent` | renamed |
| `SubAgentStop` | `AfterAgent` | Gemini has no sub-agents |
| `PreCompact` | `PreCompress` | renamed |
| `Notification` | `Notification` | exact match |
| `PostCompact` | — | no confirmed Gemini equivalent |
| `PermissionRequest` | — | no confirmed Gemini equivalent |
| `TeammateIdle` | — | no Gemini equivalent |

## Tool Name Mapping (Claude → Gemini)

Hook matchers that reference tool names must be updated when migrating.

| Claude tool | Gemini tool |
| --- | --- |
| `Edit` | `replace` |
| `Bash` | `run_shell_command` |
| `Read` | `read_file` |
| `Write` | `write_file` |
| `Glob` | `glob` |
| `Grep` | `grep` |
| `LS` | `ls` |

## Confirmed Hook Types

From `settingsSchema.js`:

| Hook type | Gemini label | Control capability |
| --- | --- | --- |
| `SessionStart` | Session Start Hooks | session init |
| `SessionEnd` | Session End Hooks | session cleanup |
| `BeforeTool` | Before Tool Hooks | intercept, validate, modify tool calls |
| `AfterTool` | After Tool Hooks | process results, log, trigger follow-up |
| `BeforeAgent` | Before Agent Hooks | set up context, initialize resources |
| `AfterAgent` | After Agent Hooks | cleanup, summarize results |
| `Notification` | Notification Hooks | idle and notification events |
| `PreCompress` | — | pre-compaction |

## Live Hooks On This Machine

From `~/.gemini/settings.json` — Gemini is already running the same ATM
Python scripts as Claude Code:

| Event | Hook name | Command |
| --- | --- | --- |
| `SessionStart` | `atm-session-start` | `python3 ~/.claude/scripts/session-start.py` |
| `SessionEnd` | `atm-session-end` | `python3 ~/.claude/scripts/session-end.py` |
| `AfterAgent` | `atm-after-agent` | `python3 ~/.claude/scripts/teammate-idle-relay.py` |

This confirms the Python ATM hooks are already cross-platform between Claude and
Gemini without modification. The `session-start.py` and `session-end.py` scripts
run identically under both runtimes.

## Environment Variable Differences

| Claude Code | Gemini | Purpose |
| --- | --- | --- |
| `CLAUDE_PROJECT_DIR` | `GEMINI_PROJECT_DIR` | project root anchor |
| `CLAUDE_SESSION_ID` | unknown — verify | session ID env var |

Hook scripts that reference `CLAUDE_PROJECT_DIR` must handle both variables or
use the hook payload `session_id` field exclusively for identity.

## Payload Schema Status

Gemini's hook stdin payload schema has not yet been captured via the test
harness. The following is inferred from Claude parity and migrate.js:

- `BeforeTool` likely receives equivalent fields to Claude `PreToolUse` with
  Gemini tool names (e.g. `run_shell_command` instead of `Bash`)
- `SessionStart` / `SessionEnd` likely receive `session_id` matching Claude
  behavior (confirmed: live `session-start.py` runs without modification)
- Exact field names must be verified by harness capture before implementation

## Design Implications

- Gemini is the easiest cross-platform target: event names differ but the
  command hook format and Python scripts are already shared
- The `migrate.js` mapping is the authoritative translation table — do not
  invent mappings that conflict with it
- `PermissionRequest` and `TeammateIdle` have no Gemini equivalents; plugins
  using these hooks are Claude-only until Gemini adds them
- `PostCompact` has no confirmed Gemini equivalent; cross-platform
  post-compaction injection is currently Claude-only
- Any plugin that avoids `CLAUDE_PROJECT_DIR` and uses `session_id` from the
  hook payload will be portable to Gemini with zero code changes
