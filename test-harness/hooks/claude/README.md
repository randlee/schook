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

- local Python capture hooks for the eight required Claude hook surfaces
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
- `PreToolUse(Bash)`
- `PostToolUse(Bash)`
- `PreToolUse(Task)`
- `PermissionRequest`
- `Stop`
- `Notification(idle_prompt)`

## Runner Usage

Use [run-capture.sh](/Users/randlee/Documents/github/schook-worktrees/feature-s9-harness-build/test-harness/hooks/claude/scripts/run-capture.sh)
to launch a Claude session against one canned prompt while writing artifacts to
the harness tree.

Example:

```bash
test-harness/hooks/claude/scripts/run-capture.sh pretooluse-bash
```

Behavior:

- writes a temporary Claude settings file with absolute hook command paths
- points all hook scripts at `test-harness/hooks/claude/captures/raw/`
- runs `claude --print` with the selected canned prompt
- leaves raw payload files in the harness capture directory

The runner is best-effort in Phase 1. The CI requirement for this phase is the
fixture-validation layer under `pytest`, not a live provider launch.

## Capture Rules

- capture hooks must write raw JSON automatically
- no manual copying or hand-moving of payloads is part of the workflow
- fixture snapshots stay separate from raw capture output
- implementation may not rely on Claude fields until later phases validate them
