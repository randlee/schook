# sc-hooks Observability Contract

## 1. Scope

`sc-hooks` currently emits structured observability events through the sibling
`../sc-observability` workspace.

This document defines the current JSONL file output owned by `sc-hooks-cli`.

It does not define:
- plugin stdin/stdout JSON
- CLI human-readable output
- spans, metrics, or OTLP export

## 2. Ownership Boundary

- `sc-hooks-cli` owns logger creation, event emission, flush, and shutdown
- the implementation uses `sc-observability` and `sc-observability-types`
- `sc-hooks-core`, `sc-hooks-sdk`, and `sc-hooks-test` do not own logger state

## 3. File Layout

Current default file sink path:

```text
.sc-hooks/observability/sc-hooks/logs/sc-hooks.log.jsonl
```

This path comes from `LoggerConfig::default_for(ServiceName::new("sc-hooks"), ".sc-hooks/observability")`.

## 4. Event Shape

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
- `mode`
- `handlers`
- `results`
- `total_ms`
- `exit`
- `ai_notification` when present

## 5. Handler Result Shape

`fields.results` is an array of per-handler records with:
- `handler`
- `action`
- `ms`
- `error_type` when present
- `stderr` when present
- `warning` when present
- `disabled` when present

This is the required place where dispatch-level error detail now lives.

## 6. Emission Rules

- if at least one handler executes, `sc-hooks` emits one dispatch-complete event
- if no handlers match, `sc-hooks` emits no observability event
- async aggregate output to stdout is unchanged and remains separate from observability emission
- runtime plugin/protocol failures still map to the existing CLI exit-code contract

## 7. Non-Goals

Current `schook` observability does not yet provide:
- configurable sink routing from `.sc-hooks/config.toml`
- console sink customization
- traces
- metrics
- OTLP export
