# Gemini Findings Ledger

Status:
- `COMPLETE`

Purpose:
- authoritative findings ledger for `N.2` Gemini schema capture

| id | surface | finding | severity | disposition | notes |
| --- | --- | --- | --- | --- | --- |
| `GEM-001` | `SessionStart` | direct raw capture confirmed fields `session_id`, `transcript_path`, `cwd`, `hook_event_name`, `timestamp`, `source` | `info` | `captured` | approved fixtures: `session-start-startup.json`, `session-start-resume.json` |
| `GEM-002` | `SessionEnd` | direct raw capture confirmed fields `session_id`, `transcript_path`, `cwd`, `hook_event_name`, `timestamp`, `reason` | `info` | `captured` | approved fixture: `session-end.json` |
| `GEM-003` | `BeforeAgent` | headless `-p` mode fires `BeforeAgent` before model execution with `prompt` included in payload | `info` | `captured` | approved fixture: `before-agent.json` |
| `GEM-004` | `BeforeTool` | tool-pre hook fires with `tool_name = "run_shell_command"` and structured `tool_input` | `info` | `captured` | approved fixture: `before-tool.json`; do not map `tool_name` to Claude/Codex names before `N.3` |
| `GEM-005` | `AfterTool` | tool-post hook fires with `tool_response.llmContent` and `tool_response.returnDisplay` | `info` | `captured` | approved fixture: `after-tool.json`; `returnDisplay` stays provider-local unless another provider proves compatible semantics |
| `GEM-006` | `AfterAgent` | post-agent hook fires with `prompt`, `prompt_response`, and `stop_hook_active` | `info` | `captured` | approved fixture: `after-agent.json` |
| `GEM-007` | `resume` | `--resume latest` produces `SessionStart.source = "resume"` with a fresh hook fire | `info` | `captured` | repo-owned evidence: `session-start-resume.json` |
| `GEM-008` | `output-format` | no hook-observable payload-key or env-key difference across `text`, `json`, and `stream-json` for `SessionStart` and `AfterAgent` | `info` | `captured` | CLI output differs, hook stdin/env schema does not |
| `GEM-009` | `registration-path` | user-scope `~/.gemini/settings.json` is the verified local registration path; `gemini hooks --help` exposing only `migrate` is not the runtime contract | `important` | `captured` | capture workflow uses isolated temporary `HOME` with copied auth files |
| `GEM-010` | `workspace-settings` | isolated probes did not demonstrate clean workspace `.gemini/settings.json` hook activation | `important` | `confirmed-not-exercisable` | keep workspace-scope registration unresolved for `N.3` |
| `GEM-011` | `control-semantics` | `SessionStart` non-zero exit warned but did not stop the turn; `BeforeAgent` non-zero exit blocked the turn; `BeforeTool` / `AfterTool` non-zero exits blocked tool progression | `important` | `captured` | `AfterAgent` and `SessionEnd` remain post-response/post-exit surfaces; their isolated non-zero exit path was partially confounded by Gemini quota exhaustion |
| `GEM-012` | `newtype-candidates` | semantic identifier fields observed: `session_id`, `GEMINI_SESSION_ID`, `GEMINI_PROJECT_DIR`, `GEMINI_CWD`, `hook_event_name`, `tool_name` | `important` | `captured` | `session_id`, `GEMINI_SESSION_ID`, `hook_event_name`, and `tool_name` are newtype candidates for later Rust normalization; path-like fields remain plain strings until `N.3` |
