# Gemini Capture Checklist

Status:
- `COMPLETE`

Purpose:
- freeze the Gemini hook-surface capture matrix for `N.2`

Named hook-surface entries for freeze gate:
- `SessionStart` — lifecycle entry surface observed in local Gemini settings and
  bundled CLI event mapping
- `SessionEnd` — lifecycle exit surface observed in local Gemini settings and
  bundled CLI event mapping
- `BeforeAgent` — prompt-submit candidate surface observed in bundled CLI event
  mapping
- `BeforeTool` — tool-pre candidate surface observed in bundled CLI event
  mapping
- `AfterTool` — tool-post candidate surface observed in bundled CLI event
  mapping
- `AfterAgent` — agent-stop candidate surface observed in local Gemini
  settings and bundled CLI event mapping

Planned coverage:
- every locally exercisable Gemini hook surface configurable through
  `.gemini/settings.json`
- lifecycle and resume surfaces if present
- per-surface disposition:
  - `captured`
  - `confirmed-not-exercisable`
- hook-process environment snapshots
- payload field inventory
- full hook env var inventory
- capture-run CLI version and hook registration details
- hook registration scope:
  - workspace `.gemini/settings.json`
  - user `~/.gemini/settings.json`
- output-format interaction checks
  - `text`
  - `json`
  - `stream-json`
- control semantics inventory
  - blocking vs non-blocking
  - exit-code handling
  - stdout/stderr contract

Final disposition:

| Surface | Status | Notes |
| --- | --- | --- |
| `SessionStart` | `COMPLETE` | captured for both `source = "startup"` and `source = "resume"` |
| `SessionEnd` | `COMPLETE` | captured with `reason = "exit"` |
| `BeforeAgent` | `COMPLETE` | captured in headless `-p` mode |
| `BeforeTool` | `COMPLETE` | captured with `tool_name = "run_shell_command"` |
| `AfterTool` | `COMPLETE` | captured with tool response payload |
| `AfterAgent` | `COMPLETE` | captured with prompt + prompt response fields |
| `--resume latest` continuity check | `COMPLETE` | verified `SessionStart.source = "resume"` |
| output-format check (`text` / `json` / `stream-json`) | `COMPLETE` | no hook-observable payload or env-key difference across tested formats |

Capture registration result:

| Scope | Result | Notes |
| --- | --- | --- |
| user `~/.gemini/settings.json` | `COMPLETE` | verified under an isolated temporary `HOME` with copied auth files |
| workspace `.gemini/settings.json` | `BLOCKED` | local isolated probes did not fire workspace-scoped hooks cleanly; not promoted into approved capture workflow |
