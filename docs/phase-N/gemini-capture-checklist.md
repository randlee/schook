# Gemini Capture Checklist

Status:
- `FROZEN`

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
- session continuation checks
  - `--resume latest`
- control semantics inventory
  - blocking vs non-blocking
  - exit-code handling
  - stdout/stderr contract
