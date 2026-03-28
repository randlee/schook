# Claude Hook Harness

## Role

Claude is the first provider in the Sprint 9 harness-first sequence.

This directory owns the Claude-specific pieces required to:

- capture raw Claude hook payloads
- store capture artifacts automatically
- validate scaffold fixtures in `pytest`
- provide the first repeatable live-capture runner

## Directory Layout

```text
claude/
  prompts/
  hooks/
  models/
  schema/
  fixtures/
  captures/
  reports/
  scripts/
  tests/
```

## Current Phase Scope

This Phase 1 scaffold provides:

- local Python capture hooks for the required Claude hook surfaces
- canned prompt files for each required surface
- a shell runner for Claude capture sessions
- placeholder fixture/model validation tests that keep the harness executable in CI

This phase does not yet provide:

- promoted implementation-facing schema fields
- Pydantic models
- generated schema artifacts from real captured payloads
- accepted hook-runtime code

## Required Claude Capture Set

- `SessionStart`
- `SessionEnd`
- `PreCompact`
- `PreToolUse(Bash)`
- `PostToolUse(Bash)`
- logical teammate/background spawn surface
- `PermissionRequest`
- `Notification(idle_prompt)`
- `Stop`

## Current Live Haiku Capture Status

Captured:

- `SessionStart(source="startup")`
- `SessionStart(source="compact")`
- `SessionStart(source="resume")`
- `SessionEnd`
- `PreCompact`
- `PreToolUse(Bash)`
- `PostToolUse(Bash)`
- teammate/background spawn via `PreToolUse` with `tool_name = "Agent"`
- `PermissionRequest` with `tool_name = "Write"`
- `PermissionRequest` with `tool_name = "Bash"`
- `Stop`

Not yet captured:

- `SessionStart(source="clear")`
- `Notification`

Observed live notes:

- `Stop` is the reliable end-of-turn signal in current Haiku capture
- `SessionStart` alone did not emit `Stop` when no turn occurred
- an automated `/clear` PTY attempt did not yield `source = "clear"`; it
  produced a new `startup` session instead
- `Notification` remained uncaptured after repeated long-idle runs with
  `matcher = ""`, including the bounded follow-up probe on this branch

## Runner Usage

Use [run-capture.sh](/Users/randlee/Documents/github/schook-worktrees/feature-s9-harness-followup/test-harness/hooks/claude/scripts/run-capture.sh)
to launch a Claude session against one canned prompt while writing artifacts to
the harness tree.

Example:

```bash
test-harness/hooks/claude/scripts/run-capture.sh pretooluse-bash
```

Or via the provider wrapper:

```bash
test-harness/hooks/scripts/run-capture.sh claude pretooluse-bash
```

Behavior:

- writes a temporary Claude settings file with absolute hook command paths
- points all hook scripts at `test-harness/hooks/claude/captures/raw/`
- runs `claude --print` with the selected canned prompt
- leaves raw payload files in the harness capture directory

For manual interactive runs, use
[prepare-manual-launch.sh](/Users/randlee/Documents/github/schook-worktrees/feature-s9-harness-followup/test-harness/hooks/claude/scripts/prepare-manual-launch.sh).
It prints a temporary settings file path and the exact `claude` command to run
with harness-local hooks.

Recommended manual rerun flow:

```bash
cd /Users/randlee/Documents/github/schook-worktrees/feature-s9-harness-followup
CLAUDE_MODEL=haiku test-harness/hooks/claude/scripts/prepare-manual-launch.sh
```

Then run the printed `claude --model haiku --setting-sources local --settings ...`
command exactly.

For the interactive helper used during this live pass:

```bash
cd /Users/randlee/Documents/github/schook-worktrees/feature-s9-harness-followup
uv run --with pexpect \
  test-harness/hooks/claude/scripts/run-interactive-capture.py \
  notification
```

Interactive surface notes from this capture pass:

- `PermissionRequest` was easiest to reproduce by asking Claude to write a file
  and leaving the request blocked at the permission prompt
- `SessionStart(source="resume")` was captured by ending a prompt-driven
  session and then resuming that exact `session_id` with `claude --resume`
- `PreCompact` plus post-compact `SessionStart(source="compact")` were captured
  only after launching Claude with harness-local settings and running
  `/compact` manually
- an automated `/clear` probe did not yield `source = "clear"`; treat clear as
  a manual follow-up capture until proven otherwise
- `Notification` remained unresolved even after switching to `matcher = ""`
  and waiting more than 60 seconds

The runner is best-effort in Phase 1. The CI requirement for this phase is the
fixture-validation layer under `pytest`, not a live provider launch.

## Capture Rules

- capture hooks must write raw JSON automatically
- no manual copying or hand-moving of payloads is part of the workflow
- fixture snapshots stay separate from raw capture output
- implementation may not rely on Claude fields until later phases validate them
