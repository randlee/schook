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
- CLI human-readable output except for the contract-tested console-sink summary
  line described below
- spans, metrics, or OTLP export

## 2. Ownership Boundary

Implements:
- `OBS-006`
- `OBS-007`
- `OBS-008`

- `sc-hooks-cli` owns logger creation, event emission, flush, and shutdown
- the implementation uses `sc-observability` and `sc-observability-types`
- `sc-hooks-core`, `sc-hooks-sdk`, and `sc-hooks-test` do not own logger state

The OBS-007/OBS-008 violation corrected in this pass was:
- `default_logger_config()` and env-flag sink routing had drifted into
  `sc-hooks-core`
- the scaffold/reference `agent-session-foundation` crate had gained direct
  `sc-observability` dependencies and its own logger construction path

Current restored boundary:
- `sc-hooks-cli` owns logger config, sink routing, event emission, flush, and
  shutdown
- `sc-hooks-core` exports `OBSERVABILITY_ROOT` and `OBSERVABILITY_LOG_PATH` only
  as shared path literals so the CLI, contract tests, and related docs agree on
  file locations without re-encoding them in multiple places
- scaffold/reference plugin crates do not own `sc-observability`

## 3. File Layout

Implements:
- `OBS-002`

Current default file sink path:

```text
.sc-hooks/observability/sc-hooks/logs/sc-hooks.log.jsonl
```

This path comes from `LoggerConfig::default_for(ServiceName::new("sc-hooks"), ".sc-hooks/observability")`.

## 3.1 Sink Routing Environment Variables

The host currently supports two observability sink toggles:

| Variable | Default | Accepted true values | Accepted false values | Purpose |
| --- | --- | --- | --- | --- |
| `SC_HOOKS_ENABLE_CONSOLE_SINK` | `false` | `1`, `true`, `yes`, `on` | `0`, `false`, `no`, `off` | Enables the human-readable console sink for live operator/debugging output |
| `SC_HOOKS_ENABLE_FILE_SINK` | `true` | `1`, `true`, `yes`, `on` | `0`, `false`, `no`, `off` | Enables the durable JSONL file sink |

Current behavior:
- unrecognized values are ignored
- the host emits a warning to `stderr` describing the accepted values
- both sinks can be enabled at the same time
- the file sink remains the canonical structured contract even when the console
  sink is enabled
- the file sink can be intentionally disabled for an operator/debugging session
  with `SC_HOOKS_ENABLE_FILE_SINK=0`

## 4. Event Shape

Implements:
- `OBS-001`
- `OBS-002`

Each line is one serialized `sc_observability_types::LogEvent`.

Current observability emission uses:
- `service = "sc-hooks"`
- `target = "hook"`
- `action = "dispatch.complete"` for normal dispatch completion
- `action = "session.root_divergence"` when inbound `CLAUDE_PROJECT_DIR` diverges from immutable `ai_root_dir`
- `outcome = "proceed" | "block" | "error"`
- `identity.pid = <current process id>`

The `fields` object for `dispatch.complete` currently carries:
- `hook`
- `event` when present
- `matcher`
- `mode`
- `handlers`
- `results`
- `total_ms`
- `exit`
- `ai_notification` when present

The `fields` object for `session.root_divergence` currently carries:
- `immutable_root`
- `observed`
- `session_id`
- `hook_event`

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
- if a handler reports a root-divergence notice, `sc-hooks` also emits one `session.root_divergence` event before the enclosing `dispatch.complete` event
- `session.root_divergence` emits with `level = Error`
- if no handlers match, `sc-hooks` emits no observability event
- if observability emission fails during dispatch completion or `session.root_divergence` emission, `sc-hooks` falls back to `stderr` with `sc-hooks: failed emitting dispatch observability event: ...` instead of silently swallowing the failure
- async aggregate output to stdout is unchanged and remains separate from observability emission
- runtime plugin/protocol failures still map to the existing CLI exit-code contract

## 7. Console Sink Expansion

Post-file-sink observability expansion:
- console-sink coverage is now the first completed post-file-sink observability
  expansion
- the file sink remains the baseline durable contract and canonical structured
  record surface
- the console sink is the operator/debugging surface for live dispatch review
  and background-agent monitoring

Current relationship between sinks:
- both sinks are driven from the same dispatch-complete `LogEvent`
- the file sink preserves the full structured JSON event, including `fields`
- the console sink intentionally renders a concise human-readable line from the
  same event, so it preserves the same top-level dispatch semantics (`level`,
  `target`, `action`, message/outcome) while not repeating the full structured
  field payload inline

Current console sink line format:
- `<timestamp> <LEVEL> <target> <action> <message>`
- the `message` currently includes `hook`, `event`, `mode`, handler count, and
  `outcome`

## 8. Non-Goals

Environment controls:
- `SC_HOOKS_ENABLE_CONSOLE_SINK`
  - accepted values: `1`, `true`, `yes`, `on`, `0`, `false`, `no`, `off`
  - default: off
  - purpose: enable the operator-facing console sink in addition to the file sink
- `SC_HOOKS_ENABLE_FILE_SINK`
  - accepted values: `1`, `true`, `yes`, `on`, `0`, `false`, `no`, `off`
  - default: on
  - purpose: control durable JSONL file emission under the resolved observability root
- both flags are evaluated by `sc-hooks-cli` at logger initialization time and use the same resolved observability root configuration

Related deferred boundary:
- `OBS-009` promotes env-flag sink toggles only; config-file sink routing,
  traces, metrics, and OTLP export remain outside the current release baseline

Current `schook` observability does not yet provide:
- configurable sink routing from `.sc-hooks/config.toml`
- console sink customization beyond the contract-tested default summary line
- traces
- metrics
- OTLP export
