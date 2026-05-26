# opencode Hook API

## Purpose

This document records the current opencode provider evidence that matters to
the maintained `sc-hooks` harness. It is intentionally separate from the
Claude, Codex, and Gemini documents because the current evidence is still
public-doc-backed approved-reference material rather than repo-owned live hook
capture.

## Current Source Of Truth

- public OpenCode plugins docs `https://opencode.ai/docs/plugins`
- the same page's event catalog and notification example proving the retained
  `session.idle` surface
- retained approved-reference harness fixtures under
  `test-harness/hooks/opencode/fixtures/approved/`

This document only promotes facts that are directly visible from those
sources.

## Model Entry Point

Current opencode payload validation entrypoint:

```python
def validate_opencode_hook_payload(payload: Any) -> OpencodeHookPayload
```

## Platform Rules

- `opencode` is not currently installed on this machine
- current planning treats opencode as a harness-only provider target, not as
  part of the normalized runtime implementation baseline
- public opencode event names may be documented here before implementation, but
  they do not become runtime inputs until later approved live harness capture
- opencode-targeting runtime work remains deferred until a later explicitly
  approved follow-on phase

## Current Public Event Baseline

Current publicly documented event names visible on the OpenCode plugins docs
page include:

- `command.executed`
- `file.edited`
- `file.watcher.updated`
- `installation.updated`
- `lsp.client.diagnostics`
- `lsp.updated`
- `message.part.removed`
- `message.part.updated`
- `message.removed`
- `message.updated`
- `permission.asked`
- `permission.replied`
- `server.connected`
- `session.created`
- `session.compacted`
- `session.deleted`
- `session.diff`
- `session.error`
- `session.idle`
- `session.status`
- `session.updated`
- `todo.updated`
- `shell.env`
- `tool.execute.after`
- `tool.execute.before`
- `tui.prompt.append`
- `tui.command.execute`
- `tui.toast.show`
- `experimental.session.compacting`

For the current harness-only follow-on scope, only one retained opencode
surface is approved:

- `session.idle`

The other public event names remain documented provider references until later
approved harness evidence promotes them.

## Retained Harness-Backed Surface

Current retained harness-backed opencode surface:

- `session.idle`

That retained surface is represented by:

- `test-harness/hooks/opencode/fixtures/approved/manifest.json`
- `test-harness/hooks/opencode/fixtures/approved/session-idle.json`
- `test-harness/hooks/opencode/fixtures/approved/session-idle.env.json`
- `test_harness/hooks/opencode/models/payloads.py`

Retained harness disposition:

- the current opencode harness is maintained
- the retained `session.idle` surface is `approved-reference`, not live-captured
- runtime normalization remains deferred

## Approved Reference Payload Shape

Current retained `session.idle` payload fields approved in the harness model:

| Field | Type | Notes |
| --- | --- | --- |
| `event.type` | string | retained literal value `"session.idle"` |

The same docs page also proves the plugin event callback shape:

- `event: async ({ event }) => { ... }`
- the notification example branches on `event.type === "session.idle"`

No additional stdin fields are promoted yet beyond that retained approved
shape.

## Planning Rules For `sc-hooks`

- do not invent opencode runtime fields beyond the retained approved-reference
  fixture
- do not write `sc-hooks` runtime code against public opencode event names that
  have not yet been promoted through later approved harness evidence
- use the current public event names as planning inputs only
- require later approved live fixture capture before any opencode-targeting
  runtime crate is implemented

## Current Platform Gaps

- no local `opencode` CLI is installed on this machine
- no live-captured opencode hook payload fixtures exist in this repo yet
- only one approved-reference opencode validation model currently exists:
  - `session.idle`
- no verified live provider-specific stdin schema has been captured yet for the
  broader public opencode event family

## Design Implications For `sc-hooks`

- treat opencode support as a maintained harness-only provider target, not as
  part of the current runtime implementation baseline
- keep the retained `session.idle` model narrow and explicitly doc-backed until
  later live capture authorizes broader opencode promotion
- any future opencode runtime path needs a separate explicit approval because
  the current `HKR-018` closure covers harness/doc-model support only
