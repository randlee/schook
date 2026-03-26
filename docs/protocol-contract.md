# sc-hooks Protocol Contract

## 1. Scope

This document defines the current host/plugin JSON contract for `contract_version = 1`.

It covers:
- manifest JSON emitted by `--manifest`
- runtime stdin JSON written by the host
- runtime stdout JSON read by the host
- stdout/stderr handling rules

It does not define:
- CLI-level human-readable output
- JSONL logging records

## 2. Manifest Contract

### 2.1 Required Fields

| Field | Type | Notes |
| --- | --- | --- |
| `contract_version` | integer | Must be `<= HOST_CONTRACT_VERSION` |
| `name` | string | Non-empty |
| `mode` | `"sync"` or `"async"` | Serialized `DispatchMode` |
| `hooks` | string array | Non-empty |
| `matchers` | string array | Non-empty |
| `requires` | object | Map of metadata field paths to field requirements |

### 2.2 Optional Fields

| Field | Type | Current meaning |
| --- | --- | --- |
| `optional` | object | Additional metadata field requirements |
| `payload_conditions` | array | Runtime payload filters |
| `timeout_ms` | integer | Positive timeout override in milliseconds |
| `long_running` | boolean | Host timeout behavior modifier |
| `response_time` | object | Async bucketing hint with `min_ms` and `max_ms` |
| `sandbox` | object | Sandbox declarations used by audit |
| `description` | string | Required when `long_running = true` |

### 2.3 Field Requirement Shape

Each `requires` or `optional` entry uses:

```json
{
  "type": "string|number|integer|boolean|object|array|any",
  "validate": "non_empty|dir_exists|file_exists|path_resolves|positive_int|one_of:a,b,c"
}
```

The `validate` field is optional.

Wire-contract note:
- `type` and `validate` are serialized string values in the public contract
- the host parses those strings into internal Rust enums such as `FieldType` and `ValidationRule`
- those enum names are not themselves part of the public plugin protocol

### 2.4 Payload Condition Shape

Each payload condition uses:

```json
{
  "path": "tool_input.command",
  "op": "contains",
  "value": "atm"
}
```

Supported operators:
- `exists`
- `not_exists`
- `equals`
- `not_equals`
- `contains`
- `starts_with`
- `matches`
- `one_of`
- `regex`

## 3. Host Input Contract

The host writes one JSON object to plugin stdin.

### 3.1 Always-Present Fields

```json
{
  "hook": {
    "type": "PreToolUse",
    "event": "Write"
  }
}
```

`hook.type` is always present.

`hook.event` is present only when the invocation has an event.

### 3.2 Filtered Metadata

The host copies into stdin only the metadata paths declared by the manifest:
- every required field in `requires`
- every present optional field in `optional`

Fields not declared by the manifest are omitted.

### 3.3 Payload Passthrough

If the host invocation received a payload, the host inserts:

```json
{
  "payload": { "...": "..." }
}
```

If no payload exists, `payload` is omitted.

The host does not send `payload: null`.

## 4. Plugin Output Contract

### 4.1 Runtime Output Shape

The host parses stdout as a `HookResult`:

```json
{
  "action": "proceed|block|error",
  "reason": "optional block reason",
  "message": "optional error message",
  "additionalContext": "optional async/user context",
  "systemMessage": "optional async/system message"
}
```

### 4.2 Sync Rules

Sync plugins may return:
- `action = "proceed"`
- `action = "block"`
- `action = "error"`

### 4.3 Async Rules

Async plugins are still parsed as `HookResult`.

Allowed behavior:
- `action = "proceed"` with optional `additionalContext`
- `action = "proceed"` with optional `systemMessage`
- `action = "error"`

Rejected behavior:
- `action = "block"` is a protocol error at runtime for async plugins

The SDK's `AsyncResult` helper converts to a proceed-style `HookResult`.

## 5. stdout And stderr Rules

### 5.1 stdout

- stdout must contain at least one valid JSON object
- the host parses only the first JSON object
- if more than one JSON object is present, the host continues using the first object and records a warning
- empty stdout is an error
- invalid JSON is an error

### 5.2 stderr

- stderr is not part of the protocol payload
- when present, stderr may be captured into logs and error reports
- stderr does not rescue invalid stdout

### 5.3 Exit Status

- non-zero exit status is treated as plugin failure
- successful exit is required even when returning `action = "error"`

## 6. Contract Version Rule

Current host contract version is `1`.

Compatibility rule:
- plugin `contract_version` must be less than or equal to the host contract version

There is no current v2 wire contract in this repository.

## 7. Environment Contract For External Plugins

The host also exports these environment variables for external plugin processes:

| Variable | Meaning |
| --- | --- |
| `SC_HOOK_TYPE` | Hook type string |
| `SC_HOOK_EVENT` | Hook event string when present |
| `SC_HOOK_METADATA` | Filesystem path to assembled metadata JSON |

These variables are convenience context, not a replacement for stdin JSON.

`SC_HOOK_METADATA` lifecycle rules:
- the host creates the metadata file before plugin invocation
- the host owns cleanup of the file after dispatch scope exits
- plugins should treat the path as read-only and ephemeral
- callers should not treat `SC_HOOK_METADATA` as a durable state file contract

## 8. Failure Classification Notes

For the current host:
- handler resolution failures and manifest-load failures are CLI resolution failures
- missing or invalid required metadata fields are CLI validation failures
- runtime protocol violations are plugin errors

Those categories affect exit-code mapping, but the wire contract itself remains JSON-based.
