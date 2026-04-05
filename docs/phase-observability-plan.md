# sc-hooks Observability Phase Plan

## 1. Purpose

This planning document freezes the next observability phase before code work
begins. It is a planning artifact only.

Terminology note:

- `full` audit in this document means observability-grade hook-attempt logging
  and durable audit output
- it does not rename or replace the existing `sc-hooks audit` command, which
  remains the static-analysis command surface unless a later CLI plan changes it

Authoritative current-state behavior remains in:

- `docs/requirements.md`
- `docs/architecture.md`
- `docs/sc-hooks-cli/architecture.md`
- `docs/sc-hooks-core/architecture.md`
- `docs/sc-hooks-sdk/architecture.md`
- `docs/project-plan.md`

## 2. Fixed Planning Decisions

- canonical product/runtime/binary/docs name converges on `sc-hooks`
- convenience CLI alias is `hooks`
- filesystem/config namespace stays `.sc-hooks/`
- global user config path is `~/.sc-hooks/config.toml`
- repo-local config path remains `.sc-hooks/config.toml`
- config precedence is:
  - built-in defaults
  - global config
  - repo-local config
  - environment overrides
- global config may set defaults, but does not enable `full` audit by itself
- `full` audit is a repo-local or operator action
- full audit failures and normal observability failures never affect hook
  execution outcomes
- durable audit JSONL is the canonical machine-readable contract for the
  committed phase
- a dedicated live structured stream plus exporter or OTel transport work is
  explicit follow-on scope after the committed phase closes
- stress and soak validation belong in integration and phase-end QA coverage,
  not in flaky or time-consuming unit tests

## 3. Mode Model

Planned observability modes:

- `off`
  - disables durable observability sinks for both `standard` and `full`
  - suppresses the durable structured dispatch records that would otherwise be
    emitted under `OBS-001`
  - does not suppress direct stderr warnings, degraded-path notices, or hard
    failures visible to the operator
- `standard`
  - keeps the current operational `sc-observability` posture
  - lower-volume records centered on normal runtime behavior
- `full`
  - enables audit-grade accounting for hook attempts and degraded paths
  - remains local-only in practice, even when global config supplies defaults

Planned `full` profiles:

- `lean`
  - optimized for evals and harness runs
  - stable machine-readable accounting with bounded volume
- `debug`
  - optimized for local troubleshooting
  - richer diagnostics, still redaction-aware by default

Raw payload capture is not part of baseline `full`. It remains a separate
explicit opt-in layered on top of the selected profile.

## 4. Config Layering Model

### 4.1 Global Config

Global config at `~/.sc-hooks/config.toml` owns:

- observability defaults
- default sink selection for `off` or `standard`
- default retention and rotation policy
- global naming and path conventions where needed
- optional default console-mirror behavior for operator sessions

Global config does not own:

- repo-specific plugin settings
- repo-specific audit policy details
- implicit activation of `full` audit mode

### 4.2 Repo-Local Config

Repo-local `.sc-hooks/config.toml` owns:

- plugin-specific settings
- repo-specific observability policy
- explicit `full` audit activation
- repo-local audit destination overrides
- repo-local redaction and capture settings inside the approved contract

### 4.3 Environment Overrides

Environment overrides remain temporary operator controls only.

They are intended for:

- harness runs
- local debugging
- CI or one-off diagnostic toggles

They are not the primary long-term contract surface.

### 4.4 Planned Config Surface

This phase resolves `DEF-006` by introducing `[observability]` as the planned
config surface instead of restoring `[logging]`.

Global `~/.sc-hooks/config.toml`:

```toml
[observability]
mode = "standard"            # allowed: "off", "standard"
console_mirror = false
retain_runs = 10
retain_days = 14
redaction = "strict"
```

Repo-local `.sc-hooks/config.toml`:

```toml
[observability]
mode = "standard"            # allowed: "off", "standard", "full"
full_profile = "lean"        # allowed: "lean", "debug"
path = ".sc-hooks/audit"
console_mirror = false
retain_runs = 10
retain_days = 14
redaction = "strict"
capture_payloads = false
capture_stdio = "summary"    # allowed: "none", "summary", "bounded"
```

Environment overrides:

- `SC_HOOKS_OBSERVABILITY_MODE`
- `SC_HOOKS_AUDIT_PROFILE`
- `SC_HOOKS_AUDIT_PATH`
- `SC_HOOKS_AUDIT_MAX_RUNS`
- `SC_HOOKS_AUDIT_MAX_AGE_DAYS`
- `SC_HOOKS_AUDIT_REDACTION`
- `SC_HOOKS_AUDIT_CAPTURE_PAYLOADS`
- `SC_HOOKS_AUDIT_CAPTURE_STDIO`
- existing `SC_HOOKS_ENABLE_FILE_SINK` and `SC_HOOKS_ENABLE_CONSOLE_SINK`
  remain operator-facing toggles for the standard sink family

Phase rules:

- `full` from global config alone is invalid
- repo-local config may enable `full`
- environment overrides may enable `full` for an operator session
- relative audit paths resolve from immutable `ai_root_dir`
- absolute audit paths are allowed only in repo-local config or environment
  overrides

### 4.5 Crate Boundary Rule

Per `ADR-SHK-003` and requirement `OBS-007`:

- `sc-hooks-cli` owns observability config loading, mode resolution, sink
  selection, and degradation policy
- `sc-hooks-core` may define sink-agnostic typed IDs or correlation metadata,
  but remains sink-agnostic and does not own logger lifecycle
- this boundary remains in force for layered config, `off | standard | full`
  mode resolution, audit-path handling, and future deferred exporter follow-on
  work

## 5. Output Model

### 5.1 Standard Operational Logging

`standard` mode keeps the current `sc-observability` boundary:

- service-scoped structured logs
- file sink beneath `.sc-hooks/observability/`
- optional human console sink

This remains the lower-volume operational surface.

### 5.2 Full Audit Logging

`full` mode adds a separate durable audit surface.

Default root:

```text
.sc-hooks/audit/
```

Required default layout:

```text
.sc-hooks/audit/runs/<run-id>/events.jsonl
.sc-hooks/audit/runs/<run-id>/meta.json
```

Reasoning:

- avoids a single hot shared audit file
- makes pruning and retention deterministic
- scales better for 50+ simultaneous agents
- keeps eval or harness runs self-contained

Path rules:

- relative paths resolve from the immutable project root / `ai_root_dir`
- absolute paths are allowed only from repo-local config or environment
  overrides, not as a global forced path

### 5.3 Machine-Readable Boundary

Human console output is not the machine contract.

Committed sink split:

- human console sink: operator-facing
- durable audit files: canonical source of truth for `full`

Structured live streaming is not part of the committed phase acceptance gate.

Sealed sink-boundary rule:

- the committed phase uses a sealed internal sink-registration boundary inside
  `sc-hooks-cli`
- this phase does not introduce a public sink-plugin API or trait-extension
  surface for third-party sinks

## 6. Event Model

### 6.1 Standard

`standard` mode continues to focus on operational records:

- dispatch-complete outcomes
- root divergence
- handler results
- timing and exit data

It keeps the current lower-volume posture.

### 6.2 Full

`full` mode must record hook-attempt accounting, including:

- hook invocation receipt
- zero-match outcomes
- pre-dispatch failures
- resolution failures
- dispatch start and dispatch end
- degraded observability paths such as fallback-to-stderr
- root-divergence sequencing when present

Implemented and contract-frozen `full` event names in `SC-LOG-S4`:

- `hook.invocation.received`
- `hook.invocation.zero_match`
- `hook.dispatch.completed`
- `hook.invocation.failed_pre_dispatch`

Planned future event names with explicit later-phase assignment:

- `hook.invocation.resolved` in `SC-LOG-S6`
- `hook.dispatch.started` in `SC-LOG-S6`
- `hook.observability.degraded` in `SC-LOG-S6`
- `hook.session.root_divergence` in `SC-LOG-S6`

`lean` profile fields:

- timestamp
- service
- run ID
- invocation ID
- hook name
- hook event name
- mode
- full profile
- project root
- current dir when relevant
- pid
- handler chain summary
- outcome
- timing
- degraded-path flags

Lean-field note:

- `session_id` is intentionally not part of the committed `SC-LOG-S4` lean
  schema; add it only through a later contract-and-implementation amendment if
  session correlation becomes a required audit field

`debug` mandatory fields, in addition to all `lean` fields, are:

- config-source summary
- config-layer resolution summary
- decision-point trace summary
- bounded per-handler stderr excerpt
- bounded per-handler stdout excerpt
- redaction action markers
- payload-capture state marker

`debug` may add optional payload excerpts only when a separate capture flag
allows them.

Field-naming note:

- the human-readable labels above are planning shorthand only
- the authoritative serialized JSON key names live in
  `docs/observability-contract.md` section `4.2`

## 7. Redaction Model

Default recommendation:

- strict redaction by default
- summarize rather than copy when data can be sensitive
- raw capture remains separate explicit opt-in

Never log verbatim by default:

- auth tokens
- API keys
- bearer headers
- cookies
- full environment dumps

Summarize by default:

- prompts
- tool arguments
- plugin stdin payloads
- plugin stdout and stderr beyond bounded excerpts

Redaction levels:

- `strict`
  - default
  - never records raw payloads
  - always summarizes prompts, tool args, stdin, and oversized stdio
- `permissive`
  - still masks known secret patterns
  - allows raw payloads or larger excerpts only when the separate capture flags
    are enabled

## 8. Retention And Rotation

Recommended defaults:

- prune by run directory, not by line slicing a shared hot file
- keep a bounded number of recent runs
- apply an age cap
- apply a size-aware cap for `debug` profile where needed
- perform pruning best-effort only

Committed defaults:

- retain the newest 10 runs
- prune runs older than 14 days
- in `debug` profile, stop bounded stdio capture after 1 MiB per handler result

Pruning failure rules:

- pruning must never fail hook execution
- pruning failure should surface as a warning or degraded audit signal only

## 9. Performance And Scale

The target is production-grade operation with at least 50 simultaneous agents
on the same repo root.

Basis for the 50-agent target:

- planned ATM multi-agent operation on a shared repo root
- eval and harness fan-out runs that need simultaneous audit capture without
  corrupting output or creating one hot shared file

Design consequences:

- no single shared hot audit file for `full`
- run-scoped sharding for durable audit output
- bounded retention
- no blocking dependency on exporter availability
- non-blocking failure semantics for audit and observability

Implementation notes:

- keep mode and profile transitions explicit through internal typestates such as
  `ObservabilityMode` and `FullAuditProfile`
- use semantic newtypes such as `RunId`, `AuditPath`, `RetentionCount`, and
  `RetentionAge` at the CLI boundary
- keep sink fan-out behind a sealed internal registration boundary
- keep degraded logging behavior inside a dedicated `ObservabilityError` family
  so recovery paths remain testable without leaking sink policy into lower crates
- `ObservabilityMode`, `FullAuditProfile`, and `ObservabilityError` are planned
  future types for this phase; they are distinct from the current
  `ObservabilityInitError` already implemented in `sc-hooks-cli`

Required validation before the phase closes:

- concurrency soak coverage
- corruption-free multi-agent writes
- bounded retention behavior under repeated runs
- stable hook outcomes when sinks or pruning fail

## 10. Planned Sprint Sequence

| Phase | Focus | Primary outcomes |
| --- | --- | --- |
| `SC-LOG-S1` / Observability Phase 0 | naming cleanup and namespace freeze | converge on `sc-hooks`, `hooks`, `.sc-hooks/` before more public surface is added |
| `SC-LOG-S2` / Observability Phase 1 | layered config foundation | freeze global/local/env precedence, config schema, and mode resolution |
| `SC-LOG-S3` / Observability Phase 2 | standard coverage for all hook events | extend lower-volume observability to all hook event types |
| `SC-LOG-S4` / Observability Phase 3 | full audit lean profile | durable run-scoped accounting for evals and harnesses |
| `SC-LOG-S5` / Observability Phase 4 | full audit debug profile and redaction | richer troubleshooting output without losing safety controls |
| `SC-LOG-S6` / Observability Phase 5 | retention, pruning, and degraded-path hardening | keep audit durable, bounded, and non-blocking under repeated runs |
| `SC-LOG-S7` / Observability Phase 6 | concurrency and production hardening | prove 50+ simultaneous agents and operational safety |

## 11. Phase Deliverables And Exit Gates

### `SC-LOG-S1` / Observability Phase 0

- file the naming cleanup as release-blocking
- replace `schook` drift in docs and public-facing surfaces
- freeze `sc-hooks` plus `hooks` alias before adding more config or sink names

Exit gate:

- requirements, architecture, project plan, and traceability all use
  `sc-hooks` as the canonical name and `hooks` as alias-only language
- no new public-facing plan text introduces `.schook/`, `hook`, or mixed binary
  naming

### `SC-LOG-S2` / Observability Phase 1

- implement layered config loading
- freeze which keys are global-only vs local-only
- add mode resolution for `off`, `standard`, and `full`
- amend `CFG-002` to add `[observability]`
- freeze the environment-override names and precedence rules

Exit gate:

- `[observability]` is the only planned config surface for observability
- `full` is rejected from global config alone
- config tests prove built-in < global < local < env precedence
- `DEF-006` is closed and both `docs/requirements.md` and `docs/architecture.md`
  are amended to retire `[logging]` in favor of `[observability]`
- the sealed internal sink-boundary rule is documented and preserved at the
  `sc-hooks-cli` boundary

### `SC-LOG-S3` / Observability Phase 2

- extend `standard` observability coverage beyond current dispatch-complete
  events
- keep current lower-volume posture for zero-match fast paths unless `full` is
  enabled
- update contracts and integration tests together

Exit gate:

- every hook invocation that reaches runtime processing emits the documented
  standard-mode operational record or degraded signal when `standard` is active
- `standard` still avoids full zero-match accounting unless `full` is active

### `SC-LOG-S4` / Observability Phase 3

- add `full` lean profile
- emit run-scoped durable audit files under `.sc-hooks/audit/`
- support explicit local path override for evals and harnesses

Exit gate:

- `full` lean profile writes JSONL under run-scoped directories
- zero-match and pre-dispatch failure paths are accounted for in `full`
- integration tests, not unit-test loops, prove the durable file contract
- degraded append-failure fallback coverage is explicitly deferred to
  `SC-LOG-S6` as planned requirement `DEF-017a`, where degraded-path hardening
  closes logger-init, emit, append, and prune failure proof together
- the closed `debug` mandatory field list is frozen in docs before debug-profile
  implementation begins
- the sealed internal sink-boundary rule is documented and preserved at the
  `sc-hooks-cli` boundary

### `SC-LOG-S5` / Observability Phase 4

- add `debug` profile
- freeze redaction policy
- keep raw payload capture behind an additional explicit opt-in

Exit gate:

- `strict` redaction is default
- `permissive` never bypasses explicit payload-capture flags
- debug output remains bounded and machine-readable
- the sealed internal sink-boundary rule still holds; debug does not introduce
  a public sink-extension API

### `SC-LOG-S6` / Observability Phase 5

- harden pruning and bounded retention
- prove logger-init, emit, append, and prune failures stay non-blocking
- preserve file-backed audit JSONL as the canonical machine-readable source
- close deferred `DEF-017a` degraded pre-dispatch audit fallback proof

Exit gate:

- pruning keeps the newest 10 runs and the 14-day age cap by default
- logger-init, emit, append, and prune failures never change hook execution outcomes
- long-term integration coverage forces append and emit failures, keeps hook
  exits unchanged, and asserts the documented degraded fallback text
- no committed phase behavior depends on live structured streaming or exporter
  availability

### `SC-LOG-S7` / Observability Phase 6

- prove 50+ simultaneous agents
- harden retention and pruning
- confirm all degraded paths remain non-blocking
- keep the concurrency proof in integration or soak coverage rather than the
  fast unit-test suite
- S4 merge-forward closure note: audit code paths rely on `AUD-008` and
  `AUD-011` being present and unique; there is no runtime dedup set involved

Exit gate:

- soak or integration runs prove 50+ simultaneous agents can write without
  corruption or shared-file contention
- the long-term QA path is documented separately from normal unit-test suites
- phase-close evidence includes load-run results and degraded-path checks

Phase-close evidence on the `SC-LOG-S7` branch:

- soak record: commit `fe66778` on 2026-04-05 kept the 64-agent
  `CONC-001` integration proof green, with all 64 of 64 concurrent host
  invocations producing one valid run-scoped audit directory each under a
  shared audit root without JSON corruption
- `cargo +1.94.1 test --workspace` includes
  `full_mode_concurrent_agents_shard_runs_without_corruption`, which runs 64
  concurrent host invocations against one shared audit root and verifies one
  valid run-scoped audit directory per agent without JSON corruption
- `cargo +1.94.1 test --workspace --release` keeps the degraded-path checks
  green for:
  - `standard_mode_logger_init_failure_is_non_blocking`
  - `standard_mode_emit_failure_is_non_blocking`
  - `full_mode_append_failure_is_non_blocking`
  - `full_mode_prune_failure_is_non_blocking`

## 12. Out Of Scope For The Committed Phase

- live structured streaming as a separate runtime sink
- remote exporter, spans, metrics, OTLP transport, or broader OTel adoption
- changing the static `sc-hooks audit` CLI into a runtime audit viewer
