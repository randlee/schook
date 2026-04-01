# Cursor Agent Hook API

## Purpose

This document records the verified Cursor Agent hook surfaces, payload schemas,
and differences from the Claude Code / sc-hooks model.

Source of truth inputs:

- `/Applications/Cursor.app/Contents/Resources/app/out/vs/workbench/workbench.desktop.main.js`
  — hook payload call sites extracted from the Cursor app bundle
- `/Users/randlee/.local/bin/cursor-agent --help` — CLI flag reference
- User-level hooks config: `~/.cursor/hooks.json`
- Project-level hooks config: `.cursor/hooks.json`
- Enterprise hooks config: `/etc/cursor/hooks.json`

## Platform Rules

- Cursor uses `hooks.json` (not `settings.json`) at user, project, and
  enterprise scopes. All matching hooks from all locations run.
- Hook scripts are invoked with the event payload on stdin as JSON.
- Control hooks respond via JSON written to stdout.
- Informational hooks: stdout output is ignored; no response expected.
- `cursor-agent --print` (headless/scriptable mode) — whether hooks fire in
  this mode is **not yet verified**; must be confirmed by harness capture.

## CLI Invocation

Binary: `/Users/randlee/.local/bin/cursor-agent`

```
cursor-agent [options] [prompt]

Key flags:
  --print              Headless/scriptable mode (non-interactive)
  --trust              Trust workspace without prompting (headless only)
  --force / --yolo     Auto-approve all actions
  --model <model>      Model to use (e.g. sonnet-4, gpt-5)
  --mode plan|ask      Read-only / Q&A mode
  --workspace <path>   Workspace directory
  --worktree [name]    Isolated git worktree at ~/.cursor/worktrees/<repo>/<name>
  --output-format      text | json | stream-json (--print only)
  --approve-mcps       Auto-approve all MCP servers
```

## Confirmed Hook Types

From the Cursor app bundle (`uee` enum in workbench.desktop.main.js):

| Hook event | Control | Description |
| --- | --- | --- |
| `beforeShellExecution` | allow / deny / ask | Fires before any shell command |
| `beforeMCPExecution` | allow / deny / ask | Fires before any MCP tool call |
| `beforeReadFile` | deny only | Fires before file contents sent to LLM |
| `afterFileEdit` | informational | Fires after Cursor modifies a file |
| `beforeSubmitPrompt` | informational (unverified) | Not in IDE bundle enum — may be cursor-agent only; verify |
| `stop` | informational | Fires when task completes |

## Common Fields (All Hooks)

Present on every hook payload:

```json
{
  "conversation_id": "<string>",
  "generation_id": "<string>",
  "hook_event_name": "<string>",
  "workspace_roots": ["<path>", ...]
}
```

## Payload Schemas (Verified From Bundle)

### beforeShellExecution

Call site confirmed in workbench.desktop.main.js:

```json
{
  "conversation_id": "<string>",
  "generation_id": "<string>",
  "hook_event_name": "beforeShellExecution",
  "workspace_roots": ["<path>"],
  "command": "<shell command string>",
  "cwd": "<working directory>"
}
```

Response (stdout JSON):
```json
{
  "permission": "allow" | "deny" | "ask",
  "userMessage": "<optional: shown to user>",
  "agentMessage": "<optional: shown to agent>",
  "continue": true | false
}
```

### beforeMCPExecution

Call site confirmed in workbench.desktop.main.js:

```json
{
  "conversation_id": "<string>",
  "generation_id": "<string>",
  "hook_event_name": "beforeMCPExecution",
  "workspace_roots": ["<path>"],
  "tool_name": "<MCP tool name>",
  "tool_input": "<JSON string of tool parameters>",
  "command": "<MCP server command>"
}
```

Note: server `name` and `url` fields also present — exact field names to verify
via harness capture.

Response: same `permission` / `userMessage` / `agentMessage` / `continue` format.

### beforeReadFile

Call site confirmed in workbench.desktop.main.js:

```json
{
  "conversation_id": "<string>",
  "generation_id": "<string>",
  "hook_event_name": "beforeReadFile",
  "workspace_roots": ["<path>"],
  "content": "<file content string>",
  "filePath": "<relative workspace path>"
}
```

**Note**: `filePath` is camelCase — inconsistent with `file_path` (snake_case) used
in `afterFileEdit`. This is a Cursor API inconsistency; handle both in any
cross-platform normalization layer.

Response: `{ "permission": "deny" }` to block the read. Redaction mechanism
(returning modified content) is not yet confirmed — verify via harness.

### afterFileEdit

Call site confirmed in workbench.desktop.main.js:

```json
{
  "conversation_id": "<string>",
  "generation_id": "<string>",
  "hook_event_name": "afterFileEdit",
  "workspace_roots": ["<path>"],
  "file_path": "<path>",
  "edits": [
    { "old_string": "<before>", "new_string": "<after>" }
  ]
}
```

Informational only — no response respected.

### stop

Payload fields from user documentation (bundle verification pending):

```json
{
  "conversation_id": "<string>",
  "generation_id": "<string>",
  "hook_event_name": "stop",
  "workspace_roots": ["<path>"],
  "status": "completed" | "aborted" | "error"
}
```

Informational only.

## Mapping To sc-hooks HookType

| Cursor event | sc-hooks analog | Gap |
| --- | --- | --- |
| `beforeShellExecution` | `PreToolUse` / `Bash` (`run_shell_command`) | tool name differs; payload has `command`+`cwd` not wrapped in `tool_input` |
| `beforeMCPExecution` | `PreToolUse` / `Task` (approximate) | no direct MCP hook type in sc-hooks today |
| `beforeReadFile` | `PreToolUse` / `Read` | `filePath` camelCase vs `file_path`; `content` pre-loaded (Claude does not pre-load) |
| `afterFileEdit` | `PostToolUse` / `Edit` or `Write` | `edits` array format differs from sc-hooks payload |
| `beforeSubmitPrompt` | no current analog | would require new `HookType` variant |
| `stop` | `Stop` | closest match; `status` field maps cleanly |

## Key Differences From Claude Code

| Dimension | Claude Code | Cursor Agent |
| --- | --- | --- |
| Config format | `settings.json` hooks array | `hooks.json` object |
| Hook granularity | `PreToolUse` generic + matcher | Per-tool-type events |
| Session lifecycle | `SessionStart` / `SessionEnd` | Not confirmed |
| Response format | `proceed` / `block` + `reason` | `allow` / `deny` / `ask` + messages |
| Content pre-loading | No — hook fires, plugin reads path | Yes — `beforeReadFile` receives full content |
| Sub-agent gating | `PreToolUse/Task` | No confirmed equivalent |
| `continue` field | Not present | `continue: false` can stop agent loop |

## Headless Testing Strategy

To verify hooks fire in `--print` mode:

```bash
# Create a minimal hooks.json with a logging hook
cat > ~/.cursor/hooks.json << 'EOF'
{
  "beforeShellExecution": [
    {
      "command": "python3 -c \"import sys,json; d=json.load(sys.stdin); open('/tmp/cursor-hook-capture.json','w').write(json.dumps(d,indent=2)); print('{\\\"permission\\\":\\\"allow\\\"}')\""
    }
  ]
}
EOF

# Run cursor-agent with a task that triggers a shell command
cursor-agent --print --trust --yolo "run: echo hello"

# Check if hook fired
cat /tmp/cursor-hook-capture.json
```

This is the minimum capture scenario for the test harness.
