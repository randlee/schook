# Codex Capture Checklist

Status:
- `FROZEN`

Purpose:
- freeze the Codex hook-surface capture matrix for `N.1`
- this planning-branch matrix is a template only; `N.1` execution must create
  a dedicated checklist-freeze commit before the first harness code commit, and
  carrying this file forward from the planning branch alone does not satisfy
  the separate-commit acceptance gate

Planned hook-surface coverage:
- `notify` — `direct hook surface (harness-capturable)`
- `PreToolUse` — `direct hook surface (harness-capturable)`
- `Stop` — `direct hook surface (harness-capturable)`; verify whether live
  Codex exec ever invokes `hooks.json` `Stop`, and if not record
  `confirmed-not-exercisable` from harness-owned evidence
- `SessionStart` — `relay-synthetic (confirmed-not-exercisable)`; source:
  `docs/hook-api/codex-hook-api.md` relay evidence from `hook_watcher.rs`
- `resume` — `relay-synthetic (confirmed-not-exercisable)`; reason: resume
  does not expose a distinct hook surface and is observed through direct
  `notify` / `PreToolUse` capture runs plus session-record correlation checks
- `fork` — `direct hook surface (harness-capturable)`; verify whether the
  forked Codex process preserves `notify` / `PreToolUse` capture visibility
- `--cd` — `direct hook surface (harness-capturable)`; verify root/current-dir
  behavior by launching capture runs with an explicit startup directory change
- restart continuity — `relay-synthetic (confirmed-not-exercisable)`; reason:
  restart does not expose a distinct hook surface and is observed through
  direct `notify` / `PreToolUse` capture runs plus session-record correlation
  checks

Planned non-surface evidence:
- hook-process environment snapshots
- payload field inventory
- full hook env var inventory
- capture-run CLI version and hook registration details
- control semantics inventory
