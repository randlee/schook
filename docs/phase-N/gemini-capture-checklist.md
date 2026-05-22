# Gemini Capture Checklist

Status:
- `PENDING`

Purpose:
- freeze the Gemini hook-surface capture matrix for `N.2`

Named hook-surface entries for freeze gate:
- `preTool` — named candidate direct hook surface for tool-pre interception
- `sessionStart` — named candidate lifecycle surface if exposed by local Gemini
  hooks runtime

Planned coverage:
- every locally exercisable `gemini hooks` surface
- lifecycle or resume surfaces if present
- hook-process environment snapshots
- payload field inventory
- full hook env var inventory
- capture-run CLI version and hook registration details
- output-format interaction checks
- control semantics inventory
