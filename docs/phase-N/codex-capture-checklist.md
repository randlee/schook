# Codex Capture Checklist

Status:
- `PENDING`

Purpose:
- freeze the Codex hook-surface capture matrix for `N.1`

Planned hook-surface coverage:
- `notify` — `direct hook surface (harness-capturable)`
- `PreToolUse` — `direct hook surface (harness-capturable)`
- `SessionStart` — `relay-synthetic (confirmed-not-exercisable)`; source:
  `docs/hook-api/codex-hook-api.md` relay evidence from `hook_watcher.rs`
- resume/restart continuity — exercised through direct `notify` /
  `PreToolUse` capture runs plus session-record correlation checks

Planned non-surface evidence:
- hook-process environment snapshots
- payload field inventory
- full hook env var inventory
- capture-run CLI version and hook registration details
- control semantics inventory
