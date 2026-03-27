# Claude Hook Harness

## Role

Claude is the first implementation provider for the hook harness.

This directory is the first execution target for:

- canned prompt definitions
- local Python capture hooks
- provider-specific validation models
- approved fixture snapshots
- formal run reports
- `pytest` tests

## Required First-Pass Capture Set

- `SessionStart`
- `SessionEnd`
- `PreToolUse(Bash)`
- `PostToolUse(Bash)`
- `PreToolUse(Task)`
- `PermissionRequest`
- `Stop`
- `Notification(idle_prompt)`

## Planned Internal Layout

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

## Rules

- use canned prompts only
- use local Python hooks only for capture
- do not promote fields into implementation until they are validated here
- write formal reports for every live capture run
- prefer long-term repeatability over broad one-off experiments

## Expected Outputs

Every Claude live-capture run should produce:

- raw captured payloads
- validation summary JSON
- drift report JSON
- markdown run report

No manual movement of these files should be part of the workflow.
