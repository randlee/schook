# sc-hooks Observability Contract

## 1. Scope

Owning requirement IDs:
- `OBS-001`
- `OBS-002`
- `OBS-005`
- `OBS-006`
- `OBS-007`
- `OBS-008`
- `OBS-009` (`Added in S9-BONUS`; traceability: `docs/traceability.md`)

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

Sealed sink-boundary rule:
- the sink-registration and sink-selection boundary remains sealed inside
  `sc-hooks-cli`
- this contract does not expose a public sink-plugin API, trait-extension
  surface, or lower-crate logger lifecycle hook
- `ADR-SHK-003` and `OBS-007` continue to require that lower crates stay
  sink-agnostic even as layered config and future audit profiles expand

## 3. File Layout

Implements:
- `OBS-002`

Current default file sink path:

```text
.sc-hooks/observability/sc-hooks/logs/sc-hooks.log.jsonl
```

This path comes from `LoggerConfig::default_for(ServiceName::new("sc-hooks"), ".sc-hooks/observability")`.

## 3.1 Environment Override Surface

The host currently supports layered `[observability]` config plus operator
environment overrides.

Resolved mode rules:

- repo-local `.sc-hooks/config.toml` may set `[observability].mode` to `off`,
  `standard`, or `full`
- global `~/.sc-hooks/config.toml` may set `[observability].mode` to `off` or
  `standard` only
- environment overrides are applied after built-in defaults, global config, and
  repo-local config
- the sink env flags below are evaluated only when the resolved mode is not
  `off`

| Variable | Default | Accepted values | Overrides | Notes |
| --- | --- | --- | --- | --- |
| `SC_HOOKS_OBSERVABILITY_MODE` | `standard` | `off`, `standard`, `full` | `[observability].mode` | `full` remains invalid when it comes from global config alone; env may enable it for an operator session |
| `SC_HOOKS_AUDIT_PROFILE` | `lean` | `lean`, `debug` | `[observability].full_profile` | applies only when mode resolves to `full` |
| `SC_HOOKS_AUDIT_PATH` | `.sc-hooks/audit` | any path | `[observability].path` | relative paths remain repo-root relative |
| `SC_HOOKS_AUDIT_MAX_RUNS` | `10` | non-negative integer | `[observability].retain_runs` | retention-count override for run pruning |
| `SC_HOOKS_AUDIT_MAX_AGE_DAYS` | `14` | non-negative integer | `[observability].retain_days` | age-cap override for run pruning |
| `SC_HOOKS_AUDIT_REDACTION` | `strict` | `strict`, `permissive` | `[observability].redaction` | redaction policy remains local/operator owned |
| `SC_HOOKS_AUDIT_CAPTURE_PAYLOADS` | `false` | `1`, `true`, `yes`, `on`, `0`, `false`, `no`, `off` | `[observability].capture_payloads` | payload capture remains separate from mode/profile selection |
| `SC_HOOKS_AUDIT_CAPTURE_STDIO` | `summary` | `none`, `summary`, `bounded` | `[observability].capture_stdio` | stdio capture detail remains bounded by profile rules |
| `SC_HOOKS_ENABLE_CONSOLE_SINK` | `false` | `1`, `true`, `yes`, `on`, `0`, `false`, `no`, `off` | sink toggle only | enables the human-readable console sink for live operator/debugging output |
| `SC_HOOKS_ENABLE_FILE_SINK` | `true` | `1`, `true`, `yes`, `on`, `0`, `false`, `no`, `off` | sink toggle only | enables the durable JSONL file sink |

Current behavior:
- when resolved `[observability].mode = "off"`, the host suppresses durable
  structured sink emission regardless of the sink env flags
- unrecognized values are ignored
- the host emits a warning to `stderr` describing the accepted values
- both sinks can be enabled at the same time
- the file sink remains the canonical structured contract even when the console
  sink is enabled
- the file sink can be intentionally disabled for an operator/debugging session
  with `SC_HOOKS_ENABLE_FILE_SINK=0`

Formal amendment note:
- `OBS-009` was added in `S9-BONUS` to promote these env-flag sink toggles into
  the release-facing observability contract; see `docs/traceability.md`.

## 3.2 `[observability]` Key Surface

The layered config surface is frozen to the keys below.

| Key | Type | Default | Global config | Repo-local config | Env mapping |
| --- | --- | --- | --- | --- | --- |
| `mode` | string enum | `standard` | yes: `off`, `standard` only | yes: `off`, `standard`, `full` | `SC_HOOKS_OBSERVABILITY_MODE` |
| `full_profile` | string enum | `lean` | no | yes | `SC_HOOKS_AUDIT_PROFILE` |
| `path` | path | `.sc-hooks/audit` | no | yes | `SC_HOOKS_AUDIT_PATH` |
| `console_mirror` | boolean | `false` | yes | yes | none |
| `retain_runs` | unsigned integer | `10` | yes | yes | `SC_HOOKS_AUDIT_MAX_RUNS` |
| `retain_days` | unsigned integer | `14` | yes | yes | `SC_HOOKS_AUDIT_MAX_AGE_DAYS` |
| `redaction` | string enum | `strict` | yes | yes | `SC_HOOKS_AUDIT_REDACTION` |
| `capture_payloads` | boolean | `false` | no | yes | `SC_HOOKS_AUDIT_CAPTURE_PAYLOADS` |
| `capture_stdio` | string enum | `summary` | no | yes | `SC_HOOKS_AUDIT_CAPTURE_STDIO` |

Key-surface rules:

- unknown `[observability]` keys are rejected during config parsing
- global config owns shared defaults only; repo-local config owns `full`
  activation and payload/detail controls
- there is no separate `[logging]` section in the committed contract
- sink env toggles remain outside the TOML key surface because they are
  operator-session overrides rather than persisted config keys

## 3.3 Full Audit Lean File Layout

When `[observability].mode = "full"`, `sc-hooks-cli` also writes the lean audit
contract to the configured audit root.

Default layout:

```text
.sc-hooks/audit/runs/<run-id>/meta.json
.sc-hooks/audit/runs/<run-id>/events.jsonl
```

Path rules:

- the default root is `.sc-hooks/audit`
- repo-local config may override the root with a relative or absolute path
- relative paths resolve from the immutable project root / `ai_root_dir`
- each CLI invocation uses one run-scoped directory; there is no shared hot
  audit file across runs

`meta.json` currently carries:

- `schema_version`
- `service`
- `run_id`
- `invocation_id`
- `profile`
- `started_at`
- `project_root`
- `pid`

## 4. Standard Event Shape

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

Amendment note (`BND-001a`, `S9-HP5`):
- the documented `DispatchEventEmitted` field inventory was expanded in
  `S9-BONUS` to freeze the currently emitted `dispatch.complete` field set
  above rather than leaving the event payload partially implied by code/tests

The `fields` object for `session.root_divergence` currently carries:
- `immutable_root`
- `observed`
- `session_id`
- `hook_event`

## 4.1 Full Audit Lean Record Shape

Each line in `events.jsonl` is one lean audit JSON object.

Current contract note:

- `session_id` is not part of the committed lean record shape in `SC-LOG-S4`
- any future session-correlation field must be added in code, tests, and this
  contract together rather than implied by planning text alone

Current mandatory lean fields are:

- `schema_version`
- `timestamp`
- `service`
- `run_id`
- `invocation_id`
- `name`
- `hook`
- `mode`
- `profile`
- `project_root`
- `pid`
- `outcome`

Current conditional lean fields are:

- `hook_event`
- `current_dir` when it differs from `project_root`
- `stage` for pre-dispatch failure records
- `handler_chain` and `handler_count` for completed dispatch records
- `total_ms`
- `exit`
- `error`
- `ai_notification`
- `degraded`

## 4.2 Full Audit Debug Mandatory Fields

The `debug` profile extends the lean record shape; it does not replace it.

The mandatory `debug`-only field set is frozen to this closed enumeration
before debug-profile implementation begins:

- `config_source_summary`
- `config_layer_resolution`
- `decision_trace_summary`
- `handler_stderr_excerpt`
- `handler_stdout_excerpt`
- `redaction_actions`
- `payload_capture_state`

Rules:

- these fields are in addition to all mandatory `lean` fields
- payload excerpts remain gated behind separate payload-capture controls
- machine-readable bounded output remains mandatory even when `debug` is active

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
- if no handlers match, `sc-hooks` emits no standard `dispatch.complete` event
- if the resolved `[observability].mode` is `off`, `sc-hooks` suppresses
  durable structured observability emission while still allowing direct stderr
  warnings and degraded-path notices
- if `standard` mode is active and resolution, metadata preparation, dispatch
  preflight, or plugin-input preparation fails before `dispatch.complete`,
  `sc-hooks` emits one degraded stderr line of the form
  `sc-hooks: standard observability degraded before dispatch.complete: stage=<stage> hook=<hook> event=<event-or-*> mode=<mode> error=<err>`
- if `full` mode is active, `sc-hooks` also appends:
  - `hook.invocation.received` at invocation start
  - `hook.invocation.zero_match` for zero-match fast paths
  - `hook.invocation.failed_pre_dispatch` for resolution, metadata preparation, dispatch preflight, and plugin-input preparation failures
  - `hook.dispatch.completed` when a handler-executing dispatch completes
- if observability emission fails during dispatch completion or `session.root_divergence` emission, `sc-hooks` falls back to `stderr` with `sc-hooks: failed emitting observability event: ...` instead of silently swallowing the failure
- if full-audit append or run-file preparation fails, `sc-hooks` falls back to
  `stderr` with `sc-hooks: full audit degraded: ...`
- async aggregate output to stdout is unchanged and remains separate from observability emission
- runtime plugin/protocol failures still map to the existing CLI exit-code contract

Formal amendment note (`SC-OBS-INTEGRATION-1-FIX-R1`):
- the previous documented fallback text was
  `sc-hooks: failed emitting dispatch observability event: ...`
- the current fallback text is
  `sc-hooks: failed emitting observability event: {err}`
- rationale: the fallback helper is shared by both `dispatch.complete` and
  `session.root_divergence` emission paths, so the older "dispatch" wording was
  too narrow and could mislabel a root-divergence emission failure

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

Correction note:
- sections `3.1` and `3.2` document current supported behavior, not a non-goal
- the non-goal is broader config-file sink routing or console-sink
  customization beyond the layered config and operator overrides documented above

Current supported controls (not non-goals):
- the full persisted `[observability]` key surface is frozen in section `3.2`
- the full operator override surface is frozen in section `3.1`
- sink toggles are still evaluated by `sc-hooks-cli` at logger initialization
  time and use the same resolved observability root configuration

Related deferred boundary:
- `OBS-009` promotes env-flag sink toggles only; config-file sink routing,
  traces, metrics, and OTLP export remain outside the current release baseline

Current `sc-hooks` observability does not yet provide:
- configurable third-party or exporter sink routing beyond the supported
  `[observability]` mode surface and the two env-flag sink toggles
- console sink customization beyond the contract-tested default summary line
- traces
- metrics
- OTLP export
