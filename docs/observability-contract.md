# sc-hooks Observability Contract

## 1. Scope

Owning requirement IDs:
- `OBS-001`
- `OBS-002`
- `OBS-005`
- `OBS-006`
- `OBS-007`
- `OBS-008`

`sc-hooks` currently emits structured observability events through the external
`sc-observability` workspace referenced by `sc-hooks-cli/Cargo.toml` at
`../../../sc-observability/...`.

This document defines the current JSONL file output owned by `sc-hooks-cli`.

It does not define:
- plugin stdin/stdout JSON
- CLI human-readable output
- spans, metrics, or OTLP export

## 2. Ownership Boundary

Implements:
- `OBS-006`
- `OBS-007`
- `OBS-008`

- `sc-hooks-cli` owns logger creation, event emission, flush, and shutdown
- the implementation uses `sc-observability` and `sc-observability-types`
- `sc-hooks-core`, `sc-hooks-sdk`, and `sc-hooks-test` do not own logger state

Current OBS-007 boundary tension:
- `sc-hooks-core` exports `OBSERVABILITY_ROOT` and `OBSERVABILITY_LOG_PATH` so the
  CLI, contract tests, and related docs can share the same resolved file-sink
  path without re-encoding literals in multiple places
- this is a narrow path-coordination exception only; logger configuration,
  sink lifecycle, and event emission remain owned by `sc-hooks-cli`

## 3. File Layout

Implements:
- `OBS-002`

Current default file sink path:

```text
.sc-hooks/observability/sc-hooks/logs/sc-hooks.log.jsonl
```

This path comes from `LoggerConfig::default_for(ServiceName::new("sc-hooks"), ".sc-hooks/observability")`.

## 4. Event Shape

Implements:
- `OBS-001`
- `OBS-002`

Each line is one serialized `sc_observability_types::LogEvent`.

Current dispatch emission uses:
- `service = "sc-hooks"`
- `target = "hook"`
- `action = "dispatch.complete"`
- `outcome = "proceed" | "block" | "error"`
- `identity.pid = <current process id>`

The `fields` object currently carries:
- `hook`
- `event` when present
- `matcher`
- `mode`
- `handlers`
- `results`
- `total_ms`
- `exit`
- `ai_notification` when present

## 5. Handler Result Shape

Implements:
- `OBS-005`

`fields.results` is an array of per-handler records with:
- `handler`
- `action`
- `ms`
- `error_type` when present
- `stderr` when present
- `warning` when present
- `disabled` when present

This is the required place where dispatch-level error detail now lives.

Current `matcher` rule:
- when an event exists, `matcher` is the event string
- when no event exists, `matcher` is `"*"`

## 6. Emission Rules

Implements:
- `OBS-001`
- `OBS-005`

- if at least one handler executes, `sc-hooks` emits one dispatch-complete event
- if no handlers match, `sc-hooks` emits no observability event
- if observability emission fails during dispatch completion, `sc-hooks` falls back to `stderr` with `sc-hooks: failed emitting dispatch observability event: ...` instead of silently swallowing the failure
- async aggregate output to stdout is unchanged and remains separate from observability emission
- runtime plugin/protocol failures still map to the existing CLI exit-code contract

## 7. Non-Goals

Related deferred boundary:
- no current requirement ID promotes configurable sink routing, traces, metrics,
  or OTLP export into the release baseline

Current `schook` observability does not yet provide:
- configurable sink routing from `.sc-hooks/config.toml`
- console sink customization
- traces
- metrics
- OTLP export
