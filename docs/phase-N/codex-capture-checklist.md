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
- `notify` — `direct hook surface (captured)`
- `PreToolUse` — `direct hook surface (captured)`
- `Stop` — `direct hook surface (confirmed-not-exercisable)`; a direct `Stop`
  hook command was configured on 2026-05-22 and did not fire in local
  `codex exec`
- `SessionStart` — `direct hook surface (captured)`; repo-owned raw stdin/env
  fixtures now exist and supersede the earlier relay-only planning assumption
- `resume` — `relay-synthetic (confirmed-not-exercisable)`; noninteractive
  `codex resume <session-id> <prompt>` exits with `Error: stdin is not a terminal`
- `fork` — `direct hook surface (confirmed-not-exercisable)`; noninteractive
  `codex fork <session-id> <prompt>` exits with `Error: stdin is not a terminal`
- `--cd` — `direct hook surface scenario (captured)`; approved `cwd-drift`
  fixtures prove startup-directory drift through `SessionStart`, `PreToolUse`,
  and `notify`
- restart continuity — `relay-synthetic (confirmed-not-exercisable)`; no
  distinct restart hook surface was exposed in the local harness

Planned non-surface evidence:
- hook-process environment snapshots
- payload field inventory
- full hook env var inventory
- capture-run CLI version and hook registration details
- control semantics inventory
