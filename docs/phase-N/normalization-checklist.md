# Normalization Checklist

Status:
- `COMPLETE`

Purpose:
- freeze the cross-provider normalization review matrix for `N.3`

Evidence baseline used in this sprint:
- Claude approved fixtures under `test-harness/hooks/claude/fixtures/approved/`
- Codex approved fixtures under `test-harness/hooks/codex/fixtures/approved/`
- Gemini approved fixtures under `test-harness/hooks/gemini/fixtures/approved/`
- provider docs:
  - `docs/hook-api/claude-hook-api.md`
  - `docs/hook-api/codex-hook-api.md`
  - `docs/hook-api/gemini-hook-api.md`

Field-family review matrix:

| Field family | Claude baseline | Codex evidence | Gemini evidence | `N.3` result | Notes |
| --- | --- | --- | --- | --- | --- |
| `session_id` | captured on all direct lifecycle and tool surfaces | captured on `SessionStart` and `PreToolUse` | captured on all approved surfaces | `canonical candidate` | promote as semantic identifier newtype candidate |
| `transcript_path` | captured on approved direct surfaces | captured on `SessionStart` and `PreToolUse` | captured on all approved surfaces | `canonical candidate` | canonical path-like context only; not identity |
| `cwd` | captured on all approved surfaces, including drift scenarios | captured on `SessionStart`, `PreToolUse`, and `notify` | captured on all approved surfaces | `canonical candidate` | feeds canonical current-dir context, not immutable root by itself |
| direct hook name | `hook_event_name` on all direct surfaces | `hook_event_name` on direct surfaces; `notify` uses `type` instead | `hook_event_name` on all approved surfaces | `provider-local` | no single canonical event-name field yet |
| lifecycle `source` | `startup`, `compact`, `resume`, `clear` observed | `startup` observed | `startup`, `resume` observed | `unresolved` | values and lifecycle semantics are not yet compatible enough to canonicalize |
| shell tool command | `tool_input.command` on Bash tool surfaces | `tool_input.command` on `PreToolUse` | `tool_input.command` on `BeforeTool` / `AfterTool` | `canonical candidate` | valid only for normalized shell-like tool surfaces |
| raw tool name | `Bash`, `Agent` | `Bash` | `run_shell_command` | `provider-local` | raw names do not align across providers |
| turn/thread/tool-use ids | no compatible Claude equivalent across the reviewed baseline | `thread-id`, `turn-id`, `tool_use_id` captured | no comparable turn/thread id in approved Gemini fixtures | `provider-local` | keep as provider identifiers pending future proof |
| model name | `model` on `SessionStart` | `model` on `SessionStart` and `PreToolUse` | absent from approved Gemini payloads | `provider-local` | not eligible for canonical mapping yet |
| post-response / turn-complete surface | `Stop` | `notify` (`agent-turn-complete`) | `AfterAgent` | `unresolved` | same lifecycle family, incompatible payload shape and control semantics |
| session-end reason | `SessionEnd.reason` optional | no approved Codex end surface | `SessionEnd.reason = "exit"` | `provider-local` | insufficient compatible evidence for canonical promotion |
| provider root env/project dir | `CLAUDE_PROJECT_DIR` stable root signal in hook env | `CODEX_THREAD_ID` stale and non-canonical | `GEMINI_PROJECT_DIR` / `GEMINI_CWD` captured | `provider-local` | provider adapters must own root recovery separately |

Canonical mapping candidates frozen for `N.3`:

| Canonical concept | Candidate type | Provider source fields | Notes |
| --- | --- | --- | --- |
| `session_id` | `SessionId` newtype | Claude `session_id`; Codex `session_id`; Gemini `session_id` | semantic identifier; not a display-only string |
| `ai_current_dir` | path-like string | Claude `cwd`; Codex `cwd`; Gemini `cwd` | current directory snapshot only |
| `transcript_path` | path-like string | Claude `transcript_path`; Codex `transcript_path`; Gemini `transcript_path` | provider transcript path; not a stable identity key |
| `shell_command` | string | Claude `tool_input.command`; Codex `tool_input.command`; Gemini `tool_input.command` | valid only on normalized shell-command surfaces |

Canonical shape examples:

```json
{
  "canonical_field": "session_id",
  "type": "SessionId",
  "providers": {
    "claude": "payload.session_id",
    "codex": "payload.session_id",
    "gemini": "payload.session_id"
  }
}
```

```json
{
  "canonical_field": "ai_current_dir",
  "type": "string",
  "providers": {
    "claude": "payload.cwd",
    "codex": "payload.cwd",
    "gemini": "payload.cwd"
  },
  "note": "Current-directory snapshot only; immutable root remains provider-specific."
}
```

```json
{
  "canonical_field": "shell_command",
  "type": "string",
  "providers": {
    "claude": "payload.tool_input.command",
    "codex": "payload.tool_input.command",
    "gemini": "payload.tool_input.command"
  },
  "note": "Applies only to shell-like tool surfaces after the provider-specific tool surface is normalized."
}
```

Explicit do-not-map items for later runtime work:
- do not map raw `tool_name` strings directly into a canonical enum from this sprint
- do not map raw lifecycle `source` values into a canonical session-source enum yet
- do not treat Codex `thread-id`, `turn-id`, or `tool_use_id` as generic cross-provider identifiers
- do not treat `CLAUDE_PROJECT_DIR`, `GEMINI_PROJECT_DIR`, `GEMINI_CWD`, or `CODEX_THREAD_ID` as one shared root/session contract
- do not promote Gemini `tool_response.returnDisplay`, `prompt`, `prompt_response`, or `stop_hook_active` into canonical fields
- do not promote Claude `permission_suggestions`, `PreCompact`-only fields, or `WorktreeCreate` / `WorktreeRemove` into the generic provider contract
- do not promote Codex `notify.client`, `notify.type`, `notify.input-messages`, or `notify.last-assistant-message` into canonical fields
