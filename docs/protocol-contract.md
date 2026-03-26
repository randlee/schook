# sc-hooks Protocol Contract

## 1. Scope

This document defines the current host/plugin contract for `sc-hooks`.

It covers:
- manifest JSON emitted by `plugin --manifest`
- runtime stdin JSON written by the host to the plugin
- runtime stdout JSON read by the host from the plugin
- runtime stderr and exit-status rules
- environment variables exported for external plugins
- current protocol-violation handling

It does not define:
- human-readable CLI output
- `sc-observability` event payloads
- release planning or gap prioritization

Current observability output is documented separately in
`docs/observability-contract.md`.

## 2. Versioning And Compatibility

Current host contract version:

```json
1
```

Current compatibility rule:
- a plugin is accepted when `plugin.contract_version <= host_contract_version`
- the current host contract version is `1`
- there is no `v2` contract defined in this repository

Change rule:
- a breaking wire-contract change requires a higher host contract version
- a non-breaking change must preserve compatibility with plugins declaring an
  older supported version

Current host behavior is coarse:
- the host checks only the integer compatibility rule above
- it does not implement a richer compatibility negotiation protocol

## 3. Manifest Contract

### 3.1 Manifest Command

The host loads manifests by executing:

```text
<plugin-path> --manifest
```

Requirements:
- the command must exit successfully
- stdout must be one valid manifest JSON object
- stderr is not part of the manifest payload

Differences from runtime dispatch:
- manifest loading parses stdout as one full JSON document
- runtime dispatch parses only the first JSON object from stdout

## 3.2 Manifest JSON Schema

Required fields:

| Field | Type | Rules |
| --- | --- | --- |
| `contract_version` | integer | must be `<= 1` |
| `name` | string | non-empty after trim |
| `mode` | `"sync"` or `"async"` | dispatch mode |
| `hooks` | string array | must contain at least one hook |
| `matchers` | string array | must contain at least one matcher |
| `requires` | object | map of metadata paths to field requirements |

Optional fields:

| Field | Type | Current meaning |
| --- | --- | --- |
| `optional` | object | optional metadata fields to copy when present |
| `payload_conditions` | array | payload filters evaluated before plugin spawn |
| `timeout_ms` | integer | timeout override; must be greater than zero when set |
| `long_running` | boolean | if true, default timeout becomes `none` unless `timeout_ms` is set |
| `response_time` | object | async bucketing hint with `min_ms` and `max_ms` |
| `sandbox` | object | audit-only sandbox declaration |
| `description` | string | required and non-empty when `long_running = true` |

### 3.3 Field Requirement Schema

Each entry under `requires` or `optional` has this wire shape:

```json
{
  "type": "string",
  "validate": "non_empty"
}
```

Fields:

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `type` | string | yes | one of `string`, `number`, `integer`, `boolean`, `object`, `array`, `any` |
| `validate` | string | no | current values listed below |

Current validation strings:
- `non_empty`
- `dir_exists`
- `file_exists`
- `path_resolves`
- `positive_int`
- `one_of:<value1>,<value2>,...`

Wire-contract rule:
- these are string values in the public contract
- internal Rust enums such as `FieldType` and `ValidationRule` are not
  themselves part of the plugin wire contract

### 3.4 Payload Condition Schema

Each payload condition has this wire shape:

```json
{
  "path": "tool_input.command",
  "op": "contains",
  "value": "atm"
}
```

Fields:

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `path` | string | yes | dot-delimited path inside the payload object |
| `op` | string | yes | operator name |
| `value` | any JSON value | depends on operator | omitted for `exists` and `not_exists` |

Current operators accepted by code:
- `exists`
- `not_exists`
- `equals`
- `not_equals`
- `contains`
- `not_contains`
- `starts_with`
- `matches`
- `one_of`
- `regex`
- `gt`
- `lt`
- `gte`
- `lte`

Current validation rules:
- `path` must not be empty and must not contain empty segments
- `one_of` requires an array of strings
- `matches` and `regex` require a string value
- `gt`, `lt`, `gte`, and `lte` require a numeric value

Release-facing note:
- `docs/requirements.md` only elevates a subset of these operators into the
  release contract today
- the full list above is the current implementation behavior

### 3.5 Hook And Matcher Semantics

Manifest hook and matcher strings are interpreted by the host event taxonomy.

Current rules:
- `PreToolUse` and `PostToolUse` use tool-event matchers such as `Write`,
  `Read`, `Edit`, and `*`
- `Notification` accepts `idle_prompt` and `*`; unknown values are warnings
- lifecycle-style hooks such as `SessionEnd`, `SessionStart`, `PreCompact`,
  `PostCompact`, `TeammateIdle`, `PermissionRequest`, and `Stop` only allow `*`
- invalid matcher combinations fail resolution/audit
- unknown tool or notification matchers are currently warnings rather than hard
  protocol failures

### 3.6 Manifest Validation Rules

Current manifest validation rejects:
- empty `name`
- empty `hooks`
- empty `matchers`
- incompatible `contract_version`
- `timeout_ms = 0`
- `response_time.min_ms > response_time.max_ms`
- `long_running = true` with missing or empty `description`
- unknown field-validation rules
- invalid payload-condition structure

## 4. Host-To-Plugin Runtime Input

### 4.1 Invocation Model

For runtime execution, the host spawns the plugin executable with no extra
subcommand and writes one JSON object to stdin.

### 4.2 Runtime Input Shape

Minimal runtime input:

```json
{
  "hook": {
    "type": "PreToolUse"
  }
}
```

With event and filtered metadata:

```json
{
  "team": {
    "name": "schook"
  },
  "repo": {
    "branch": "feature/s7-doc-repair"
  },
  "hook": {
    "type": "PreToolUse",
    "event": "Write"
  },
  "payload": {
    "tool_input": {
      "command": "git status"
    }
  }
}
```

### 4.3 Metadata Filtering Rules

The host copies only manifest-declared metadata paths into stdin:
- every path in `requires`
- every path in `optional` when that metadata is present

Filtering behavior:
- undeclared metadata is omitted
- dot-delimited paths become nested JSON objects in stdin
- required metadata is validated before spawn
- optional metadata is also validated when present

Failure behavior before spawn:
- a missing required field is a validation failure
- an invalid required field is a validation failure
- an invalid optional field that is present is also a validation failure

Those failures stop execution before the plugin process is spawned.

### 4.4 Hook Object Rules

The host always inserts:

```json
{
  "hook": {
    "type": "<hook-name>"
  }
}
```

Rules:
- `hook.type` is always present
- `hook.event` is present only when the invocation has an event

### 4.5 Payload Passthrough Rules

If the host invocation received a payload, it inserts:

```json
{
  "payload": { "...": "..." }
}
```

Rules:
- payload is copied through as JSON
- if no payload exists, `payload` is omitted
- the host does not send `payload: null`
- the host does not send `payload: {}`

## 5. Environment Contract

The host exports these environment variables for external plugin processes:

| Variable | Meaning |
| --- | --- |
| `SC_HOOK_TYPE` | hook type string |
| `SC_HOOK_EVENT` | hook event string when present |
| `SC_HOOK_METADATA` | filesystem path to assembled metadata JSON |

Rules:
- these variables are convenience context, not a replacement for stdin JSON
- the host creates `SC_HOOK_METADATA` before dispatch
- the host owns cleanup of the metadata file after dispatch scope exits
- plugins should treat `SC_HOOK_METADATA` as read-only and ephemeral
- plugins must not treat `SC_HOOK_METADATA` as durable state

## 6. Plugin-To-Host Runtime Output

### 6.1 Common Result Schema

The host parses plugin stdout into this `HookResult` shape:

```json
{
  "action": "proceed",
  "reason": null,
  "message": null,
  "additionalContext": null,
  "systemMessage": null
}
```

Field meanings:

| Field | Type | Required | Current meaning |
| --- | --- | --- | --- |
| `action` | `"proceed" \| "block" \| "error"` | yes | required outcome |
| `reason` | string or null | no | sync block reason |
| `message` | string or null | no | plugin-provided error detail |
| `additionalContext` | string or null | no | async aggregate user context |
| `systemMessage` | string or null | no | async aggregate system message |

### 6.2 Sync Result Semantics

Sync-mode plugins may return:
- `action = "proceed"`
- `action = "block"`
- `action = "error"`

Current host behavior:
- sync `block` short-circuits the chain
- missing `reason` on sync block is allowed; the host substitutes
  `"plugin blocked without reason"`
- sync `error` disables the plugin for the session and fails the invocation

### 6.3 Async Result Semantics

Async-mode plugins use the same JSON schema, but with stricter behavior:
- `action = "proceed"` is allowed
- `additionalContext` is collected and later joined with `\n---\n`
- `systemMessage` is collected and later joined with `\n`
- `action = "error"` disables the plugin for the session and is treated as a
  plugin failure
- `action = "block"` is a protocol violation for async plugins

Current host handling of async `block`:
- the plugin is disabled for the session
- the host records the failure in observability output
- the async host invocation continues and emits an AI-facing system message

### 6.4 Host Async Aggregate Output

After all async plugins complete, the host may print one aggregate JSON object
to its own stdout:

```json
{
  "additionalContext": "joined text or null",
  "systemMessage": "joined text or null"
}
```

This is host output, not plugin output.

## 7. stdout, stderr, And Exit Status Rules

### 7.1 Runtime stdout Rules

Current runtime stdout parsing rules:
- stdout must contain at least one valid JSON object
- the host parses only the first JSON object
- if additional output follows the first JSON object, the host keeps the first
  object and records a warning
- empty stdout is a failure
- invalid JSON is a failure

### 7.2 Runtime stderr Rules

Current stderr rules:
- stderr is not part of the protocol payload
- stderr may be captured into observability fields and error reports
- stderr does not rescue invalid stdout

### 7.3 Exit Status Rules

Current exit-status rules:
- non-zero exit status is a plugin failure
- zero exit is still required when returning `action = "error"`
- a plugin cannot signal `action = "error"` by process exit status alone

## 8. Protocol Violation And Failure Behavior

Current host behavior on runtime failures:

| Failure class | Current host behavior |
| --- | --- |
| spawn failure | plugin is disabled for the session; invocation fails |
| stdin write failure | plugin is disabled for the session; invocation fails |
| stdout read failure | plugin is disabled for the session; invocation fails |
| stderr read failure | plugin is disabled for the session; invocation fails |
| non-zero exit | plugin is disabled for the session; invocation fails |
| invalid or empty stdout JSON | plugin is disabled for the session; invocation fails |
| async `action = "block"` | plugin is disabled for the session; async invocation continues with warning/system message |
| `action = "error"` | plugin is disabled for the session; invocation fails |
| timeout in sync mode | plugin is disabled for the session; invocation fails |
| timeout in async mode | plugin is disabled for the session; async invocation continues with warning/system message |

Failure classification notes:
- resolution-time handler or manifest-load failures are not runtime protocol
  payload failures; they are CLI resolution failures
- missing or invalid metadata fields are validation failures that happen before
  runtime spawn
- runtime protocol violations map to plugin failures in the current CLI
  taxonomy

## 9. Current Gaps And Stability Notes

Known release-relevant gaps around this contract:
- compliance coverage is still incomplete for some runtime behaviors; see
  `GAP-001`
- the `long_running` contract is not yet aligned end to end across host, SDK,
  docs, and tests; see `GAP-002`
- strict stdout rejection after the first JSON object is not current behavior;
  see `GAP-010`

This document describes current behavior exactly where code is clear. It does
not elevate every code path into a release guarantee unless
`docs/requirements.md` does so as well.
