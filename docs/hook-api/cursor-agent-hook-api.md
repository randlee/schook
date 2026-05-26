# Cursor Agent Hook API

## Purpose

This document records the current Cursor Agent provider evidence that matters
to the maintained `sc-hooks` harness. It is intentionally separate from the
Claude and Codex documents because the current evidence still comes from a
different combination of local CLI behavior and public Cursor hook
documentation rather than from repo-owned live hook capture.

## Current Source Of Truth

- local CLI help from `cursor-agent --help`
- local Cursor CLI state under `$HOME/.cursor/`
- public Cursor docs page `https://cursor.com/docs/hooks`

This document only promotes facts that are directly visible from those sources
or from the retained approved-reference Cursor harness fixtures landed in
`Q.6` / `Q.7`.

## Model Entry Point

Current Cursor Agent payload validation entrypoint:

```python
def validate_cursor_agent_hook_payload(payload: Any) -> CursorAgentHookPayload
```

## Platform Rules

- `cursor-agent` is a real installed CLI on this machine
- current planning treats Cursor Agent as a harness-only provider target, not
  as part of the initial Claude ATM implementation baseline
- public Cursor hook names may be documented here before implementation, but
  they do not become implementation inputs until the harness captures them
- Cursor-targeting runtime work remains deferred until a later explicitly
  approved follow-on pass

## Path And Environment Rules

- local Cursor CLI state currently lives under `$HOME/.cursor/`
- `$HOME/.cursor/cli-config.json` is a verified local configuration
  input
- `$HOME/.cursor/hooks.json` is not currently present on this machine
- current working directory must not be treated as a stable hook identity or
  provider-contract signal
- provider-specific path and environment assumptions must be captured by the
  harness before implementation relies on them

## Current Local Runtime Baseline

Current locally verified CLI behavior:

- `cursor-agent` is installed and runnable
- headless/CLI usage supports:
  - `--print`
  - `--output-format text | json | stream-json`
  - `--mode plan | ask`
  - `--resume`
  - `--continue`
  - `--workspace`
  - `--worktree`

Current locally verified config state:

- `$HOME/.cursor/cli-config.json` exists
- `$HOME/.cursor/hooks.json` does not currently exist on this machine

That means `sc-hooks` can treat Cursor Agent as an installed provider with a
current CLI/runtime surface, but not as a provider whose local hook config and
stdin payloads have already been captured live in this repo.

## Retained Harness-Backed Surface

Current retained harness-backed Cursor surface:

- `stop`

That retained surface is represented by:

- `test-harness/hooks/cursor-agent/fixtures/approved/manifest.json`
- `test-harness/hooks/cursor-agent/fixtures/approved/stop.json`
- `test-harness/hooks/cursor-agent/fixtures/approved/stop.env.json`
- `test_harness/hooks/cursor_agent/models/payloads.py`

Retained harness disposition:

- the current Cursor harness is maintained
- the retained `stop` surface is `approved-reference`, not live-captured
- runtime normalization remains deferred

## Current Public Hook Baseline

Current publicly documented hook/event names visible on the Cursor hooks page
include:

- `beforeShellExecution`
- `beforeMCPExecution`
- `beforeReadFile`
- `afterFileEdit`
- `stop`
- `sessionStart`
- `sessionEnd`
- `preCompact`
- `subagentStart`
- `subagentStop`
- `beforeSubmitPrompt`
- `afterAgentResponse`
- `afterAgentThought`

For the current harness-only follow-on scope, the broader public Cursor hook
set still includes:

- controllable hooks:
  - `beforeShellExecution`
  - `beforeMCPExecution`
  - `beforeReadFile`
- informational hooks:
  - `afterFileEdit`
  - `stop`

Only `stop` is currently retained in the approved manifest. The other public
hook names remain documented provider references until later approved harness
evidence promotes them.

## Verified Public Schema Fragments

The current Cursor hooks page also shows these currently documented field or
config names:

- common configuration keys:
  - `failClosed`
  - `matcher`
- example request/response fields:
  - `command`
  - `permission`
- documented hook payload fields in the current page content:
  - `transcript_path`
  - `user_email`
  - `is_parallel_worker`
  - `git_branch`
  - `duration_ms`
  - `message_count`
  - `tool_call_count`
  - `loop_count`
  - `modified_files`
  - `agent_transcript_path`
  - `is_first_compaction`

These are verified as names currently present in Cursor's public hook docs.
Only the `stop` fixture fields currently promoted into the Cursor harness model
are treated as approved-reference contract inputs; the rest are still doc-only
until later approved harness evidence promotes them.

## Approved Reference Payload Shape

Current retained `stop` payload fields approved in the harness model:

| Field | Type | Notes |
| --- | --- | --- |
| `hook_event_name` | string | `"stop"` |
| `transcript_path` | string | public-doc-backed transcript path field |
| `git_branch` | string | public-doc-backed |
| `duration_ms` | integer | public-doc-backed |
| `message_count` | integer | public-doc-backed |
| `tool_call_count` | integer | public-doc-backed |
| `loop_count` | integer | public-doc-backed |
| `modified_files` | array of string | public-doc-backed |

## Planning Rules For `sc-hooks`

- do not assume the full Cursor IDE hook schema is identical to the
  `cursor-agent` CLI runtime without live capture evidence
- do not write `sc-hooks` code against Cursor field names that have only been
  seen in public docs and not yet captured by the harness
- use the current public hook names as planning inputs only
- require live fixture capture before any Cursor-targeting runtime crate is
  implemented

## Current Platform Gaps

- no live-captured `cursor-agent` hook payload fixtures exist in this repo yet
- no current local `hooks.json` is configured on this machine
- only one approved-reference Cursor validation model currently exists:
  - `stop`
- no verified live provider-specific stdin schema has been captured yet for:
  - `beforeShellExecution`
  - `beforeMCPExecution`
  - `beforeReadFile`
  - `afterFileEdit`
  - `stop`

## Design Implications For `sc-hooks`

- treat Cursor hook support as a maintained harness-only provider target, not
  as part of the Claude implementation baseline
- use the schema-capture harness to prove the actual `cursor-agent` hook
  payloads before any runtime crate depends on them
- keep the retained `stop` model narrow and explicitly doc-backed until live
  capture authorizes broader Cursor promotion
