# sc-hooks Logging Contract

## 1. Scope

`sc-hooks` currently writes two JSONL record shapes to the configured hook log path.

This is the exact current behavior:
- builtin `log` records
- dispatch records

Consumers must handle both shapes.

## 2. Log Destination

Default path:

```text
.sc-hooks/logs/hooks.jsonl
```

The path is configurable through `[logging].hook_log`.

## 3. Builtin Log Record Shape

When the builtin `log` handler executes, it appends:

```json
{
  "ts_millis": 1774480000000,
  "hook": "PreToolUse",
  "event": "Write",
  "mode": "sync",
  "handler": "log",
  "action": "proceed"
}
```

Current fields:

| Field | Type | Notes |
| --- | --- | --- |
| `ts_millis` | integer | Milliseconds since epoch |
| `hook` | string | Hook name |
| `event` | string or null | Event when present |
| `mode` | `"sync"` or `"async"` | Dispatch mode |
| `handler` | string | Always `log` for this record shape |
| `action` | string | Currently `proceed` |

## 4. Dispatch Record Shape

When at least one handler executes, the host appends:

```json
{
  "ts": "2026-03-26T00:00:00Z",
  "ts_millis": 1774480000000,
  "hook": "PreToolUse",
  "event": "Write",
  "matcher": "Write",
  "mode": "sync",
  "handlers": ["guard-paths"],
  "results": [
    {
      "handler": "guard-paths",
      "action": "proceed",
      "ms": 2
    }
  ],
  "total_ms": 2,
  "exit": 0,
  "ai_notification": null,
  "level": "info"
}
```

### 4.1 Top-Level Fields

| Field | Type | Required |
| --- | --- | --- |
| `ts` | string | yes |
| `ts_millis` | integer | yes |
| `hook` | string | yes |
| `event` | string or null | no |
| `matcher` | string | yes |
| `mode` | `"sync"` or `"async"` | yes |
| `handlers` | string array | yes |
| `results` | array | yes |
| `total_ms` | integer | yes |
| `exit` | integer | yes |
| `ai_notification` | string or null | no |
| `level` | `"debug"|"info"|"warn"|"error"` | yes |

### 4.2 `results[]` Fields

Each result item uses:

| Field | Type | Required |
| --- | --- | --- |
| `handler` | string | yes |
| `action` | string | yes |
| `ms` | integer | yes |
| `error_type` | string or null | no |
| `stderr` | string or null | no |
| `warning` | string or null | no |
| `disabled` | boolean or null | no |

## 5. Behavioral Rules

### 5.1 Zero-Match Behavior

If no handlers match, the host does not append a dispatch record.

### 5.2 Error Behavior

Error records may include:
- `error_type`
- captured `stderr`
- `disabled = true`
- `ai_notification`

### 5.3 Timeout Behavior

Sync timeout:
- appends a dispatch record with an error result
- returns timeout failure

Async timeout:
- appends a dispatch record
- preserves overall async host success
- carries `ai_notification` describing the timeout

### 5.4 Multiple JSON Objects

If a plugin prints multiple JSON objects, the host:
- uses the first object
- records a result-level warning

## 6. Consumer Guidance

Consumers must branch on record shape:

- if the object has `results`, parse it as a dispatch record
- if the object has `handler` and no `results`, parse it as a builtin log record

There is currently no explicit record discriminator field.

## 7. Known Contract Gap

Because both record shapes share the same file without a discriminator, downstream log consumers need shape-based parsing. This is tracked in `docs/implementation-gaps.md`.
