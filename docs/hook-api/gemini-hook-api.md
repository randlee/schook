# Gemini CLI Hook API

## Purpose

This document records the verified Gemini CLI hook surfaces, payload schemas,
and differences from the Claude Code / sc-hooks model.

Source of truth inputs:

- `/Users/randlee/Documents/github/gemini-cli` — pulled to dac373562
- `packages/core/src/hooks/types.ts` — complete HookEventName enum, input/output
  interfaces, and MCP context types
- `packages/commands/hooks/migrate.js` — canonical Claude→Gemini event and tool
  name mapping
- `~/.gemini/settings.json` — live hook registrations on this machine

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

Gemini-only hooks with no Claude Code equivalent:

| Gemini event | Description |
| --- | --- |
| `BeforeModel` | fires before model is called; can modify or replace LLM request |
| `AfterModel` | fires after model responds; can modify LLM response |
| `BeforeToolSelection` | fires before model selects tools; can modify tool config |

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

## Confirmed Hook Types (from types.ts)

| Hook type | Control capability |
| --- | --- |
| `SessionStart` | context injection, stop execution |
| `SessionEnd` | informational |
| `PreCompress` | informational |
| `BeforeAgent` | context injection, stop execution |
| `AfterAgent` | informational; can clear context |
| `BeforeTool` | allow/block/ask; can modify tool input |
| `AfterTool` | context injection; can request tail tool call |
| `Notification` | suppress output, inject system message |
| `BeforeModel` | can modify LLM request or return synthetic response |
| `AfterModel` | can modify LLM response |
| `BeforeToolSelection` | can modify tool config |

## Common Fields (All Hooks)

From `HookInput` interface in types.ts:

```json
{
  "session_id": "<string>",
  "transcript_path": "<path string>",
  "cwd": "<working directory>",
  "hook_event_name": "<string>",
  "timestamp": "<ISO 8601 string>"
}
```

Note: Gemini includes `transcript_path`, `cwd`, and `timestamp` as common fields.
Claude Code's common fields do not include `cwd` or `timestamp` at the documented level.

## Response Format

All hooks return JSON on stdout. Common response fields (`HookOutput`):

```json
{
  "continue": true,
  "stopReason": "<optional string>",
  "suppressOutput": false,
  "systemMessage": "<optional string>",
  "decision": "ask" | "block" | "deny" | "approve" | "allow" | null,
  "reason": "<optional string>",
  "hookSpecificOutput": {}
}
```

`continue: false` stops execution. `decision: "block"` or `"deny"` blocks the action.

## Payload Schemas (Verified From types.ts Source)

### SessionStart

```json
{
  "session_id": "<string>",
  "transcript_path": "<path>",
  "cwd": "<path>",
  "hook_event_name": "SessionStart",
  "timestamp": "<ISO 8601>",
  "source": "startup" | "resume" | "clear"
}
```

Response `hookSpecificOutput`:
```json
{ "hookEventName": "SessionStart", "additionalContext": "<string>" }
```

**Note**: Gemini `source` values: `startup`, `resume`, `clear`. Claude verified values: `init`, `compact`. These are different enums — do not conflate.

### SessionEnd

```json
{
  "session_id": "<string>",
  "transcript_path": "<path>",
  "cwd": "<path>",
  "hook_event_name": "SessionEnd",
  "timestamp": "<ISO 8601>",
  "reason": "exit" | "clear" | "logout" | "prompt_input_exit" | "other"
}
```

Informational — no response respected.

### PreCompress

```json
{
  "session_id": "<string>",
  "transcript_path": "<path>",
  "cwd": "<path>",
  "hook_event_name": "PreCompress",
  "timestamp": "<ISO 8601>",
  "trigger": "manual" | "auto"
}
```

Response: `{ "suppressOutput": bool, "systemMessage": "<string>" }` only.

### BeforeAgent (Claude: UserPromptSubmit)

```json
{
  "session_id": "<string>",
  "transcript_path": "<path>",
  "cwd": "<path>",
  "hook_event_name": "BeforeAgent",
  "timestamp": "<ISO 8601>",
  "prompt": "<user prompt string>"
}
```

Response `hookSpecificOutput`:
```json
{ "hookEventName": "BeforeAgent", "additionalContext": "<string>" }
```

### AfterAgent (Claude: Stop)

```json
{
  "session_id": "<string>",
  "transcript_path": "<path>",
  "cwd": "<path>",
  "hook_event_name": "AfterAgent",
  "timestamp": "<ISO 8601>",
  "prompt": "<original prompt>",
  "prompt_response": "<agent response>",
  "stop_hook_active": true | false
}
```

Response `hookSpecificOutput`:
```json
{ "hookEventName": "AfterAgent", "clearContext": true }
```

`clearContext: true` clears the conversation context.

### BeforeTool (Claude: PreToolUse)

```json
{
  "session_id": "<string>",
  "transcript_path": "<path>",
  "cwd": "<path>",
  "hook_event_name": "BeforeTool",
  "timestamp": "<ISO 8601>",
  "tool_name": "<Gemini tool name>",
  "tool_input": {},
  "mcp_context": {
    "server_name": "<string>",
    "tool_name": "<original MCP tool name>",
    "command": "<stdio command>",
    "args": ["<arg>"],
    "cwd": "<path>",
    "url": "<SSE/HTTP url>",
    "tcp": "<WebSocket address>"
  },
  "original_request_name": "<string>"
}
```

`mcp_context` is only present for MCP tools. `command`/`args`/`cwd` are for stdio transport; `url` for SSE/HTTP; `tcp` for WebSocket.

Response `hookSpecificOutput`:
```json
{ "hookEventName": "BeforeTool", "tool_input": {} }
```

Can return modified `tool_input` to change what the tool receives.

### AfterTool (Claude: PostToolUse)

```json
{
  "session_id": "<string>",
  "transcript_path": "<path>",
  "cwd": "<path>",
  "hook_event_name": "AfterTool",
  "timestamp": "<ISO 8601>",
  "tool_name": "<Gemini tool name>",
  "tool_input": {},
  "tool_response": {},
  "mcp_context": { ... },
  "original_request_name": "<string>"
}
```

Response `hookSpecificOutput`:
```json
{
  "hookEventName": "AfterTool",
  "additionalContext": "<injected context string>",
  "tailToolCallRequest": { "name": "<tool>", "args": {} }
}
```

`tailToolCallRequest` executes another tool immediately after, replacing the original response.

### Notification

```json
{
  "session_id": "<string>",
  "transcript_path": "<path>",
  "cwd": "<path>",
  "hook_event_name": "Notification",
  "timestamp": "<ISO 8601>",
  "notification_type": "ToolPermission",
  "message": "<string>",
  "details": {}
}
```

Response: `{ "suppressOutput": bool, "systemMessage": "<string>" }` only.

### BeforeModel (Gemini-only)

```json
{
  "session_id": "<string>",
  "transcript_path": "<path>",
  "cwd": "<path>",
  "hook_event_name": "BeforeModel",
  "timestamp": "<ISO 8601>",
  "llm_request": {}
}
```

`llm_request` is the full LLM request in Gemini's decoupled `LLMRequest` format (not the raw Google AI SDK format).

Response `hookSpecificOutput`:
```json
{
  "hookEventName": "BeforeModel",
  "llm_request": {},
  "llm_response": {}
}
```

Return `llm_request` to modify the model call, or `llm_response` to short-circuit with a synthetic response.

### AfterModel (Gemini-only)

```json
{
  "session_id": "<string>",
  "transcript_path": "<path>",
  "cwd": "<path>",
  "hook_event_name": "AfterModel",
  "timestamp": "<ISO 8601>",
  "llm_request": {},
  "llm_response": {}
}
```

Response `hookSpecificOutput`:
```json
{ "hookEventName": "AfterModel", "llm_response": {} }
```

Return `llm_response` to replace the model's response.

### BeforeToolSelection (Gemini-only)

```json
{
  "session_id": "<string>",
  "transcript_path": "<path>",
  "cwd": "<path>",
  "hook_event_name": "BeforeToolSelection",
  "timestamp": "<ISO 8601>",
  "llm_request": {}
}
```

Response `hookSpecificOutput`:
```json
{ "hookEventName": "BeforeToolSelection", "toolConfig": {} }
```

Return `toolConfig` to change which tools are available for the model's next tool selection.

## Live Hooks On This Machine

From `~/.gemini/settings.json` — Gemini is already running the same ATM
Python scripts as Claude Code:

| Event | Hook name | Command |
| --- | --- | --- |
| `SessionStart` | `atm-session-start` | `python3 ~/.claude/scripts/session-start.py` |
| `SessionEnd` | `atm-session-end` | `python3 ~/.claude/scripts/session-end.py` |
| `AfterAgent` | `atm-after-agent` | `python3 ~/.claude/scripts/teammate-idle-relay.py` |

This confirms the Python ATM hooks are already cross-platform between Claude and
Gemini without modification.

## Environment Variable Differences

| Claude Code | Gemini | Purpose |
| --- | --- | --- |
| `CLAUDE_PROJECT_DIR` | `GEMINI_PROJECT_DIR` | project root anchor |
| `CLAUDE_SESSION_ID` | unknown — verify | session ID env var |

Hook scripts that reference `CLAUDE_PROJECT_DIR` must handle both variables or
use the hook payload `session_id` field for identity.

## Mapping To sc-hooks HookType

| Gemini event | sc-hooks analog | Gap |
| --- | --- | --- |
| `SessionStart` | `SessionStart` | `source` enum values differ (`startup/resume/clear` vs Claude `init/compact`) |
| `SessionEnd` | `SessionEnd` | Gemini adds `reason` enum field |
| `PreCompress` | `PreCompact` | name differs; Gemini adds `trigger` field |
| `BeforeAgent` | `UserPromptSubmit` | payload has `prompt` field in both |
| `AfterAgent` | `Stop` | Gemini adds `prompt_response`, `stop_hook_active`, `clearContext` output |
| `BeforeTool` | `PreToolUse` | tool names differ; MCP context format richer than Claude |
| `AfterTool` | `PostToolUse` | tool names differ; `tailToolCallRequest` is Gemini-only |
| `Notification` | `Notification` | `notification_type` enum narrower (only `ToolPermission`) |
| `BeforeModel` | none | no Claude equivalent — intercepts LLM call |
| `AfterModel` | none | no Claude equivalent — intercepts LLM response |
| `BeforeToolSelection` | none | no Claude equivalent — intercepts tool config |

## Key Differences From Claude Code

| Dimension | Claude Code | Gemini |
| --- | --- | --- |
| Config format | `settings.json` hooks array | `settings.json` hooks object |
| Common base fields | `session_id`, `transcript_path`, `cwd`, `hook_event_name` | `session_id`, `transcript_path`, `cwd`, `timestamp` |
| Hook granularity | type + matcher regex | type-per-event |
| Session start source | `init`, `compact` (verified) | `startup`, `resume`, `clear` |
| Session end | no `reason` field | `reason` enum |
| PreCompact/PostCompact | both present | `PreCompress` only; no PostCompress confirmed |
| PermissionRequest | present | not present |
| TeammateIdle | present | not present |
| LLM interception | not present | `BeforeModel`, `AfterModel`, `BeforeToolSelection` |
| Tool input modification | not present | `BeforeTool` can return modified `tool_input` |
| Tail tool calls | not present | `AfterTool` can request tail tool call |
| Context clearing | not present | `AfterAgent` `clearContext` output |
| Sub-agent gating | `PreToolUse/Task` | no confirmed equivalent |

## Design Implications

- `PermissionRequest` and `TeammateIdle` have no Gemini equivalents; plugins
  using these hooks are Claude-only until Gemini adds them.
- `PostCompact` has no confirmed Gemini equivalent.
- Any plugin that avoids `CLAUDE_PROJECT_DIR` and uses `session_id` from the
  hook payload will be portable to Gemini with zero code changes.
- `BeforeModel`, `AfterModel`, and `BeforeToolSelection` are Gemini-specific
  power hooks with no sc-hooks HookType analog today. If implemented, they
  require new `HookType` variants.
- The `tailToolCallRequest` AfterTool feature has no sc-hooks model today.
- SessionStart `source` enum values differ between platforms — do not treat
  them as compatible enums in cross-platform hook code.
