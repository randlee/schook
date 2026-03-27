# Codex Hook API

## Purpose

This document records the verified Codex hook surfaces, wire schemas, and
differences from the Claude Code / sc-hooks model.

Source of truth inputs:

- `/Users/randlee/Documents/github/codex` — pulled to e8949f450
- `codex-rs/hooks/src/schema.rs` — complete input/output wire types with
  serde attributes
- `codex-rs/hooks/src/types.rs` — HookPayload notify-style struct
- `codex-rs/hooks/src/events/` — per-event request/outcome types
- `codex-rs/hooks/src/lib.rs` — public exports
- `codex-rs/core/src/hook_runtime.rs` — call sites and field populations

## Platform Rules

- Codex uses `hooks.json` (not `settings.json`) for hook registration.
- Config file locations follow the Codex config layer stack; the user-level
  default is `~/.codex/hooks.json`.
- Claude-style settings.json hooks are **not** read by Codex.
- Hooks are behind a feature flag (`Feature::CodexHooks`) that must be enabled.
- Hook scripts are invoked with the event payload on stdin as JSON.
- Hooks respond via JSON written to stdout.
- Exit code 2 with stderr blocks `PreToolUse` execution (legacy mechanism).

## Two Hook Mechanisms

Codex has two distinct hook systems:

### 1. Command Hooks (Claude-compatible stdin/stdout style)

These match the Claude Code hook model: command configured in `hooks.json`,
payload sent on stdin, response read from stdout.

Supported event names: `SessionStart`, `PreToolUse`, `PostToolUse`,
`UserPromptSubmit`, `Stop`

### 2. Notify Hooks (fire-and-forget)

These use a separate `HookPayload` wire format. The `notify` config key
specifies an argv for fire-and-forget invocations. Event types: `after_agent`,
`after_tool_use`.

This is the mechanism used by `atm-hook-relay.py` — the notify hook receives
a full payload via stdin with `hook_event.event_type` as the discriminator.

## Codex Extensions vs Claude

Codex command hooks diverge from Claude Code in these ways:

| Field | Claude Code | Codex | Notes |
| --- | --- | --- | --- |
| `turn_id` | absent | present (all turn-scoped events) | Codex extension |
| `model` | absent | present (all events) | Codex extension |
| `permission_mode` | absent | present (all events) | values: default, acceptEdits, plan, dontAsk, bypassPermissions |
| `transcript_path` | present (nullable) | present (nullable) | both |
| `SessionStart.source` | `init`, `compact` | `startup`, `resume` | different values, no `clear` |

## Common Input Fields (All Command Hooks)

```json
{
  "session_id": "<string>",
  "transcript_path": "<path or null>",
  "cwd": "<working directory>",
  "hook_event_name": "<string>",
  "model": "<model slug>",
  "permission_mode": "default" | "acceptEdits" | "plan" | "dontAsk" | "bypassPermissions"
}
```

Turn-scoped events also include:
```json
{
  "turn_id": "<string>"
}
```

## Common Output Fields (All Command Hooks)

```json
{
  "continue": true,
  "stopReason": "<optional string>",
  "suppressOutput": false,
  "systemMessage": "<optional string>"
}
```

`continue: false` stops execution. These are the `HookUniversalOutputWire` fields.

## Payload Schemas — Command Hooks

### SessionStart

Input:
```json
{
  "session_id": "<string>",
  "transcript_path": "<path or null>",
  "cwd": "<path>",
  "hook_event_name": "SessionStart",
  "model": "<string>",
  "permission_mode": "<string>",
  "source": "startup" | "resume"
}
```

No `turn_id` on SessionStart (not a turn-scoped event).

Output:
```json
{
  "continue": true,
  "stopReason": "<optional string>",
  "suppressOutput": false,
  "systemMessage": "<optional string>",
  "hookSpecificOutput": {
    "hookEventName": "SessionStart",
    "additionalContext": "<optional string injected into model context>"
  }
}
```

Plain text stdout is also accepted as `additionalContext`. JSON-like stdout that
fails to parse is an error (fails open).

**Source values**: `startup` (first launch), `resume` (resumed session). There
is no `clear` value in Codex. Compare with Claude's verified `init` and `compact`.

### PreToolUse

Input:
```json
{
  "session_id": "<string>",
  "turn_id": "<string>",
  "transcript_path": "<path or null>",
  "cwd": "<path>",
  "hook_event_name": "PreToolUse",
  "model": "<string>",
  "permission_mode": "<string>",
  "tool_name": "Bash",
  "tool_use_id": "<string>",
  "tool_input": {
    "command": "<shell command string>"
  }
}
```

**Note**: `tool_name` is currently hardcoded to `"Bash"` in all call sites. Only
Bash tool execution fires PreToolUse hooks.

Output (modern form):
```json
{
  "hookSpecificOutput": {
    "hookEventName": "PreToolUse",
    "permissionDecision": "allow" | "deny",
    "permissionDecisionReason": "<optional string>"
  }
}
```

Note: `"ask"` is defined in the enum but not supported — it fails with an error
rather than prompting the user.

Output (legacy deprecated form):
```json
{
  "decision": "block",
  "reason": "<optional string>"
}
```

The `"approve"` decision is also deprecated and fails with an error.

Exit code 2 with stderr also blocks execution (legacy mechanism, still supported).

### PostToolUse

Input:
```json
{
  "session_id": "<string>",
  "turn_id": "<string>",
  "transcript_path": "<path or null>",
  "cwd": "<path>",
  "hook_event_name": "PostToolUse",
  "model": "<string>",
  "permission_mode": "<string>",
  "tool_name": "Bash",
  "tool_use_id": "<string>",
  "tool_input": {
    "command": "<shell command string>"
  },
  "tool_response": {}
}
```

**Note**: `tool_name` is currently hardcoded to `"Bash"`.

Output:
```json
{
  "hookSpecificOutput": {
    "hookEventName": "PostToolUse",
    "additionalContext": "<optional string>",
    "updatedMCPToolOutput": {}
  }
}
```

`additionalContext` is injected into model context. `updatedMCPToolOutput`
replaces the MCP tool response.

Block form:
```json
{
  "decision": "block",
  "reason": "<string>"
}
```

### UserPromptSubmit

Input:
```json
{
  "session_id": "<string>",
  "turn_id": "<string>",
  "transcript_path": "<path or null>",
  "cwd": "<path>",
  "hook_event_name": "UserPromptSubmit",
  "model": "<string>",
  "permission_mode": "<string>",
  "prompt": "<user prompt string>"
}
```

Output:
```json
{
  "hookSpecificOutput": {
    "hookEventName": "UserPromptSubmit",
    "additionalContext": "<optional string>"
  }
}
```

Block form: `{ "decision": "block", "reason": "<string>" }` or `continue: false`.

### Stop

Input:
```json
{
  "session_id": "<string>",
  "turn_id": "<string>",
  "transcript_path": "<path or null>",
  "cwd": "<path>",
  "hook_event_name": "Stop",
  "model": "<string>",
  "permission_mode": "<string>",
  "stop_hook_active": true | false,
  "last_assistant_message": "<string or null>"
}
```

Output block form: `{ "decision": "block", "reason": "<string>" }` or `continue: false`.

## Payload Schemas — Notify Hooks (HookPayload)

These use a different wire format. The payload is sent on stdin to the `notify`
command configured in Codex config.

```json
{
  "session_id": "<ThreadId string>",
  "cwd": "<path>",
  "client": "<optional string>",
  "triggered_at": "<RFC 3339 timestamp>",
  "hook_event": {
    "event_type": "after_agent" | "after_tool_use",
    ...event-specific fields...
  }
}
```

### after_agent event

```json
{
  "hook_event": {
    "event_type": "after_agent",
    "thread_id": "<ThreadId string>",
    "turn_id": "<string>",
    "input_messages": ["<string>"],
    "last_assistant_message": "<string or null>"
  }
}
```

### after_tool_use event

```json
{
  "hook_event": {
    "event_type": "after_tool_use",
    "turn_id": "<string>",
    "call_id": "<string>",
    "tool_name": "<string>",
    "tool_kind": "function" | "custom" | "local_shell" | "mcp",
    "tool_input": {
      "input_type": "local_shell",
      "params": {
        "command": ["<argv>"],
        "workdir": "<optional string>",
        "timeout_ms": 60000,
        "sandbox_permissions": "<optional string>",
        "justification": null,
        "prefix_rule": null
      }
    },
    "executed": true,
    "success": true,
    "duration_ms": 42,
    "mutating": true,
    "sandbox": "<string>",
    "sandbox_policy": "<string>",
    "output_preview": "<string>"
  }
}
```

`tool_input` is tagged with `input_type`: `function`, `custom`, `local_shell`, or `mcp`.
For `mcp`: `{ "input_type": "mcp", "server": "<string>", "tool": "<string>", "arguments": "<json string>" }`.

Notify hooks are fire-and-forget — no response is read.

## ATM Relay Mapping

The `atm-hook-relay.py` script consumes the notify hook HookPayload format:

| Notify event | ATM event type |
| --- | --- |
| `after_agent` | `agent-turn-complete` (state relay) |
| (session lifecycle) | `session-start`, `session-end` (from Codex lifecycle events) |

The `hook_watcher.rs` in `agent-team-mail` consumes these JSONL events using the
following fields: `event_type`, `agent`, `team`, `thread-id`, `turn-id`,
`received-at`, `state`, `timestamp`, `idempotency-key`, `sessionId`,
`processId`.

## Mapping To sc-hooks HookType

| Codex event | sc-hooks analog | Gap |
| --- | --- | --- |
| `SessionStart` | `SessionStart` | source values differ (`startup/resume` vs Claude `init/compact`) |
| `PreToolUse` (Bash) | `PreToolUse/Bash` | Codex adds `turn_id`, `model`, `permission_mode` |
| `PostToolUse` (Bash) | `PostToolUse/Bash` | Codex adds `turn_id`, `model`, `tool_response` |
| `UserPromptSubmit` | `UserPromptSubmit` | Codex adds `turn_id`, `model` |
| `Stop` | `Stop` | Codex adds `turn_id`, `model`, `stop_hook_active`, `last_assistant_message` |
| notify `after_agent` | `Stop` (approximate) | different wire format entirely |
| notify `after_tool_use` | `PostToolUse` (approximate) | different wire format entirely |

## Key Differences From Claude Code

| Dimension | Claude Code | Codex |
| --- | --- | --- |
| Config format | `settings.json` hooks array | `hooks.json` object |
| Feature flag | always enabled | behind `Feature::CodexHooks` |
| Common extension fields | none | `turn_id`, `model`, `permission_mode` |
| SessionStart source | `init`, `compact` | `startup`, `resume` |
| Tool coverage | all tool types | Bash only (PreToolUse/PostToolUse) |
| Notify hooks | not present | `after_agent`, `after_tool_use` via separate HookPayload |
| Block mechanism | `decision: "block"` | `hookSpecificOutput.permissionDecision: "deny"` (modern) or `decision: "block"` (legacy) |
| `ask` decision | prompts user | not supported (fails with error) |
| Additional context | via `hookSpecificOutput` | same format |
| `PermissionRequest` | present | not present |
| `Notification` | present | not present |

## Design Implications For `schook`

- Codex uses `hooks.json` like Cursor, not `settings.json` like Claude or
  Gemini. A future Codex adapter must handle this config path difference.
- The `turn_id`, `model`, and `permission_mode` Codex extensions are not in the
  Claude contract; cross-platform hook code should treat them as optional.
- Bash-only tool coverage means Codex hooks cannot gate non-shell operations.
  Planning for Codex tool gating beyond Bash is premature.
- The notify hook `HookPayload` format is a separate contract from the
  command hook stdin format. Cross-platform normalization must handle both.
- `Feature::CodexHooks` must be confirmed enabled in any Codex test environment
  before hook tests will fire.
- SessionStart `source` enum values differ across all three platforms (Claude,
  Gemini, Codex). Any cross-platform session-start handler must treat the
  source as a platform-specific enum, not a shared one.
