# Claude Haiku Capture Report

## Scope

This report records the first live Claude Haiku harness pass in
`feature-s9-haiku-harness-testing`.

Worktree:

- `/Users/randlee/Documents/github/schook-worktrees/feature-s9-haiku-harness-testing`

Provider:

- `claude --model haiku`

## Captured Raw Surfaces

Representative raw captures retained under `captures/raw/`:

- `20260327T063151.567307Z-session-start.json`
  - `SessionStart`
  - `source = "startup"`
- `20260327T071254.001282Z-session-start.json`
  - `SessionStart`
  - `source = "compact"`
- `20260327T071243.458795Z-pre-compact.json`
  - `PreCompact`
  - `trigger = "manual"`
- `20260327T054546.720877Z-pretooluse-bash.json`
  - `PreToolUse`
  - `tool_name = "Bash"`
- `20260327T054546.843011Z-posttooluse-bash.json`
  - `PostToolUse`
  - `tool_name = "Bash"`
- `20260327T054744.357048Z-pretooluse-agent.json`
  - `PreToolUse`
  - `tool_name = "Agent"`
- `20260327T063236.772257Z-permission-request.json`
  - `PermissionRequest`
  - `tool_name = "Write"`
- `20260327T064124.716131Z-permission-request.json`
  - `PermissionRequest`
  - `tool_name = "Bash"`
- `20260327T064748.256536Z-stop.json`
  - `Stop`
- `20260327T071354.207249Z-session-end.json`
  - `SessionEnd`
  - `reason = "prompt_input_exit"`

Approved fixture snapshots were copied into `fixtures/approved/` with stable
names.

## Key Verified Findings

- `SessionStart.source` is `startup` on fresh start and `compact` after
  `/compact`.
- `PreCompact` is a real surface and includes `trigger` plus
  `custom_instructions`.
- teammate/background spawn currently arrives as `PreToolUse` with
  `tool_name = "Agent"` in Haiku capture.
- `PermissionRequest` was captured for both `Write` and `Bash`.
- `Stop` is the reliable observed end-of-turn signal.
- `SessionStart` by itself did not emit `Stop` when no turn occurred.

## Unresolved Surface

`Notification` was not captured in this environment.

What was tried:

- manual interactive launches with harness-local settings
- `Notification` with `matcher = "idle_prompt"`
- `Notification` with `matcher = ""`
- repeated long-idle waits after startup
- repeated long-idle waits after completed turns
- repeated long-idle waits after permission and compact scenarios

Observed result:

- no `notification` payload was produced in the harness capture directory

## Interpretation

- `Notification` remains wired and documented
- `Notification` should remain unresolved/deferred in the current plan until
  it is reproduced with a live payload
- the rest of the Claude-first design can now proceed from captured evidence
