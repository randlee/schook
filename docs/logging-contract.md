# sc-hooks Logging Contract

## 1. Scope

Owning requirement IDs:
- `OBS-001`
- `OBS-002`
- `OBS-005`
- `OBS-006`
- `OBS-007`
- `OBS-008`
- `OBS-009` (`Env override ownership and the layered `[observability]` surface are defined in `docs/observability-contract.md` §3.1; SC-LOG-S4 cross-reference`)

Scope note:
- `DEF-010`, `DEF-011`, `DEF-012`, `DEF-013`, `DEF-017`, and `DEF-017a` are
  owned by `docs/observability-contract.md` and are intentionally excluded from
  this contract's owning-ID list
- `session.root_divergence` is owned by `docs/observability-contract.md`; this
  document covers the `dispatch.complete` log family only

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
- future observability sinks beyond the current file sink baseline and the
  contract-tested default console sink
- full-audit debug-field semantics; those live in
  `docs/observability-contract.md`

Consistency note:
- fallback stderr wording for observability-emission failures is owned by
  `docs/observability-contract.md`; this document intentionally does not repeat
  the exact fallback string

## 1.1 Environment Controls

- `[observability].mode`
  - repo-local accepted values: `off`, `standard`, `full`
  - global accepted values: `off`, `standard`
  - when resolved to `off`, no durable dispatch log line is written and the
    sink env flags below do not re-enable structured logging
- `SC_HOOKS_ENABLE_CONSOLE_SINK`
  - accepted values: `1`, `true`, `yes`, `on`, `0`, `false`, `no`, `off`
  - default: off
  - enables console-sink emission for operator/debugging workflows when the
    resolved mode is not `off`
- `SC_HOOKS_ENABLE_FILE_SINK`
  - accepted values: `1`, `true`, `yes`, `on`, `0`, `false`, `no`, `off`
  - default: on
  - controls durable JSONL file emission beneath the resolved observability root
    when the resolved mode is not `off`
- when both are enabled, console and file sinks emit the same dispatch
  semantics while differing only in presentation/rendering

Important current reality:
- the current implementation does not emit the old ad hoc `DispatchLogEntry`
  record shape
- each log line is one `sc_observability_types::LogEvent`
- the file sink is the canonical structured contract; the console sink is a
  human-readable rendering of the same dispatch event for operator/debugging use

## 1.2 Console Sink Relationship

- the default console sink renders one human-readable line per qualifying
  dispatch
- it preserves the same dispatch semantics as the file sink for `level`,
  `target`, `action`, and message/outcome
- it does not inline the full structured `fields` payload; consumers that need
  exact `fields` data must continue to use the JSONL file sink

## 2. File And Line Model

Implements:
- `OBS-001`
- `OBS-002`

Current default file path:

```text
.sc-hooks/observability/logs/sc-hooks.log.jsonl
```

Current write model:
- the file is newline-delimited JSON
- each line is one complete dispatch log record
- no line is written when the resolved `[observability].mode` is `off`
- if no handlers execute, no line is written
- if a pre-dispatch failure triggers the standard-mode degraded stderr signal,
  no JSONL line is written because `dispatch.complete` was never emitted

## 2.1 Sink Routing Environment Variables

The current host supports these sink-routing toggles:

| Variable | Default | Accepted true values | Accepted false values | Effect |
| --- | --- | --- | --- | --- |
| `SC_HOOKS_ENABLE_CONSOLE_SINK` | `false` | `1`, `true`, `yes`, `on` | `0`, `false`, `no`, `off` | Enables the human-readable console sink alongside normal dispatch execution |
| `SC_HOOKS_ENABLE_FILE_SINK` | `true` | `1`, `true`, `yes`, `on` | `0`, `false`, `no`, `off` | Enables the JSONL file sink at the contract path above |

Current rules:
- resolved `[observability].mode = "off"` suppresses durable dispatch logging
  before the env-flag sink toggles are considered
- both sinks may be enabled simultaneously
- the file sink remains the canonical structured logging surface
- invalid values fall back to the documented default and emit a warning to
  `stderr`
- standard-mode degraded stderr signals for pre-dispatch failures are owned by
  `docs/observability-contract.md`; they are intentionally outside the JSONL
  dispatch-log envelope contract described here

## 3. Top-Level Record Envelope

Implements:
- `OBS-001`
- `OBS-002`

Each line is one serialized `sc_observability_types::LogEvent`.

Current dispatch records use:

| Field | Type | Current value or rule |
| --- | --- | --- |
| `version` | string | current `OBSERVATION_ENVELOPE_VERSION` |
| `timestamp` | timestamp string | emitted at dispatch completion time |
| `level` | string | current values are `Info`, `Warn`, or `Error` |
| `service` | string | always `sc-hooks` |
| `target` | string | always `hook` |
| `action` | string | always `dispatch.complete` for the dispatch-log records covered by this contract |
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

Implements:
- `OBS-001`
- `OBS-002`

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

Implements:
- `OBS-005`

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

Implements:
- `OBS-001`
- `OBS-005`

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
- `outcome = "error"` and `exit = 2` for invalid JSON, non-zero exit, spawn,
  stdin/stdout/stderr, wait, and `action = "error"` plugin-failure paths
- `outcome = "error"` and `exit = 6` for sync timeout
- `results[*].action = "error"`
- `results[*].error_type` identifies the failure class
- `results[*].disabled = true` is present when the handler was disabled
- `ai_notification` is usually present

Exit-taxonomy owner:
- `EXC-002`
- `EXC-006`

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
- the top-level `level` becomes `Error`

Current implementation note:
- `sc-hooks-cli/src/observability.rs` computes `Info` only when every result
  has no `error_type`, no `warning`, and `disabled != true`
- a warning therefore prevents `Info`, and a non-blocking dispatch with no
  other special case falls through to `Error`

### 6.7 Invalid Trailing Garbage After First JSON Object

If a plugin prints a valid first JSON object followed by invalid trailing
output:
- the host treats the dispatch as `invalid_json`
- the handler is disabled
- the invocation fails like any other runtime protocol violation

## 7. AI-Notification Rules

Implements:
- `OBS-001`
- `OBS-005`

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

Related owning IDs:
- `OBS-001`
- `OBS-002`
- `OBS-005`

Downstream consumers should:
- parse each line as a `LogEvent`
- branch on `service = "sc-hooks"` and `action = "dispatch.complete"`
- read dispatch-specific fields from the `fields` object
- treat `results[*].error_type` as the durable per-handler failure indicator
- not rely on `outcome` or `exit` alone for async failure detection

## 9. Cross-References

Boundary owner IDs:
- `OBS-006`
- `OBS-007`
- `OBS-008`

- ownership boundary and file-path rationale: `docs/observability-contract.md`
- plugin wire contract: `docs/protocol-contract.md`
- normative release-facing behavior: `docs/requirements.md`
