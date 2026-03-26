# sc-hooks Logging Contract

## 1. Scope

This document defines the current JSONL dispatch-log contract for downstream
consumers.

It covers:
- the current log file path used by `sc-hooks`
- the top-level JSON record envelope written per dispatch
- the dispatch-specific `fields` payload
- per-handler result records
- outcome-specific logging behavior
- AI-notification logging rules

It does not define:
- plugin stdin/stdout protocol details
- CLI human-readable output
- future observability sinks beyond the current file sink

Important current reality:
- the current implementation does not emit the old ad hoc `DispatchLogEntry`
  record shape
- each log line is one `sc_observability_types::LogEvent`

## 2. File And Line Model

Current default file path:

```text
.sc-hooks/observability/sc-hooks/logs/sc-hooks.log.jsonl
```

Current write model:
- the file is newline-delimited JSON
- each line is one complete dispatch log record
- if no handlers execute, no line is written

## 3. Top-Level Record Envelope

Each line is one serialized `sc_observability_types::LogEvent`.

Current dispatch records use:

| Field | Type | Current value or rule |
| --- | --- | --- |
| `version` | string | current `OBSERVATION_ENVELOPE_VERSION` |
| `timestamp` | timestamp string | emitted at dispatch completion time |
| `level` | string | current values are `Info`, `Warn`, or `Error` |
| `service` | string | always `sc-hooks` |
| `target` | string | always `hook` |
| `action` | string | always `dispatch.complete` |
| `message` | string | currently always present |
| `identity.hostname` | string or null | currently `null` |
| `identity.pid` | integer or null | current process id |
| `trace` | object or null | currently `null` |
| `request_id` | string or null | currently `null` |
| `correlation_id` | string or null | currently `null` |
| `outcome` | string or null | current values are `proceed`, `block`, or `error` |
| `diagnostic` | object or null | currently `null` |
| `state_transition` | object or null | currently `null` |
| `fields` | object | dispatch-specific payload defined below |

Current `message` format:

```text
dispatch hook=<hook> event=<event-or-*> mode=<sync-or-async> handlers=<count> outcome=<outcome>
```

## 4. Dispatch Fields Payload

The `fields` object currently carries:

| Field | Type | Required | Rule |
| --- | --- | --- | --- |
| `hook` | string | yes | hook name |
| `event` | string | no | present only when an event exists |
| `matcher` | string | yes | event name when present, otherwise `*` |
| `mode` | `"sync"` or `"async"` | yes | dispatch mode |
| `handlers` | string array | yes | configured handler chain for this dispatch |
| `results` | array | yes | per-handler results |
| `total_ms` | integer | yes | total elapsed dispatch time in milliseconds |
| `exit` | integer | yes | exit code the host associates with this dispatch record |
| `ai_notification` | string | no | present only when the host generated an AI-facing notification |

Important consumer note:
- `handlers` is the configured chain for the invocation
- `results` contains only handlers that actually reached a logged result path

## 5. Handler Result Record

`fields.results` is an array of these records:

| Field | Type | Required | Rule |
| --- | --- | --- | --- |
| `handler` | string | yes | handler name |
| `action` | string | yes | current values are `proceed`, `block`, or `error` |
| `ms` | integer | yes | elapsed handler time in milliseconds |
| `error_type` | string | no | present for runtime error paths |
| `stderr` | string | no | present when stderr or error detail was captured |
| `warning` | string | no | present for non-fatal parse warnings such as multiple JSON objects |
| `disabled` | boolean | no | present and true when the handler was disabled for the session |

Current `error_type` values emitted by code:
- `spawn_error`
- `stdin_write_failed`
- `timeout`
- `wait_failed`
- `stdout_read_failed`
- `stderr_read_failed`
- `non_zero_exit`
- `invalid_json`
- `async_block`
- `action_error`

## 6. Logging Behavior By Outcome

### 6.1 Sync Proceed

Current behavior:
- `outcome = "proceed"`
- `exit = 0`
- `ai_notification` is absent
- the final handler result has `action = "proceed"`

Current level rule:
- `level = "Info"` only when there are no warnings, no `error_type`, and no
  `disabled = true` entries

### 6.2 Sync Block

Current behavior:
- `outcome = "block"`
- `exit = 1`
- `level = "Warn"`
- `ai_notification` is absent
- the blocking handler result has `action = "block"`

### 6.3 Sync Error Paths

Current behavior for sync runtime failures such as timeout, invalid JSON,
non-zero exit, `action = "error"`, and process I/O failures:
- `level = "Error"`
- `outcome = "error"` for non-timeout failures
- `outcome = "error"` and `exit = 6` for sync timeout
- `results[*].action = "error"`
- `results[*].error_type` identifies the failure class
- `results[*].disabled = true` is present when the handler was disabled
- `ai_notification` is usually present

### 6.4 Async Proceed

Current behavior:
- `outcome = "proceed"`
- `exit = 0`
- `additionalContext` and `systemMessage` aggregation is separate host stdout,
  not part of the log fields payload

### 6.5 Async Timeout And Async Block

Current behavior:
- the handler result is logged as `action = "error"`
- `error_type` is `timeout` or `async_block`
- `disabled = true` is present
- `ai_notification` is present
- the top-level record still carries `exit = 0`
- the top-level record still carries `outcome = "proceed"`
- `level = "Error"`

Important current limitation:
- downstream consumers cannot infer async partial failure from `outcome` or
  `exit` alone
- consumers must inspect `level`, `results[*].error_type`, and
  `ai_notification`

### 6.6 Multiple JSON Objects

If a plugin prints multiple valid JSON objects:
- the host uses only the first object
- the relevant result record includes a `warning`
- the top-level record still carries `exit = 0`
- the top-level `level` becomes `Error` because any warning currently raises
  the dispatch level above `Info`

### 6.7 Trailing Garbage After First JSON Object

If a plugin prints a valid first JSON object followed by invalid trailing
output:
- the host still uses the first JSON object
- the relevant result record includes a `warning`
- the invocation does not fail solely because of the trailing output
- the top-level `level` becomes `Error` because any warning currently raises
  the dispatch level above `Info`

Important current limitation:
- strict stdout rejection after the first JSON object is not current behavior
- that stricter behavior remains deferred as `GAP-010`

## 7. AI-Notification Rules

`fields.ai_notification` is present only when the host generated a user-facing
or AI-facing remediation message.

Current emission cases include:
- spawn failure
- stdin write failure
- timeout
- wait failure
- stdout read failure
- stderr read failure
- invalid JSON
- non-zero exit
- async block
- `action = "error"`

Current non-emission cases include:
- normal proceed
- normal sync block
- multiple-JSON warning without another failure

## 8. Consumer Guidance

Downstream consumers should:
- parse each line as a `LogEvent`
- branch on `service = "sc-hooks"` and `action = "dispatch.complete"`
- read dispatch-specific fields from the `fields` object
- treat `results[*].error_type` as the durable per-handler failure indicator
- not rely on `outcome` or `exit` alone for async failure detection

## 9. Cross-References

- ownership boundary and file-path rationale: `docs/observability-contract.md`
- plugin wire contract: `docs/protocol-contract.md`
- normative release-facing behavior: `docs/requirements.md`
