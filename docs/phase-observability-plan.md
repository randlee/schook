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
- global config may set defaults and future exporter wiring, but does not
  enable `full` audit by itself
- `full` audit is a repo-local or operator action
- full audit failures and normal observability failures never affect hook
  execution outcomes

## 3. Mode Model

Planned observability modes:

- `off`
  - disables durable observability sinks
  - does not suppress direct stderr warnings or failures
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
- future exporter and OTel defaults
- default sink selection for `off` or `standard`
- default retention and rotation policy
- global naming and path conventions where needed

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

Recommended default layout:

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

### 5.3 Machine-Readable Live Streaming

Human console output is not the machine contract.

If live machine-readable processing is required, it should use a separate
structured sink, not the human-readable console formatter.

Planned sink split:

- human console sink: operator-facing
- structured live stream: harness-facing
- durable audit files: canonical source of truth for `full`

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

`lean` profile fields:

- invocation ID
- session ID when present
- hook name
- hook event name
- mode
- project root
- current dir when relevant
- pid
- handler chain summary
- outcome
- timing
- degraded-path flags

`debug` profile adds richer diagnostics such as:

- handler stderr or stdout summaries
- config-source and layer notes
- more decision-point detail
- optional payload excerpts only when a separate capture flag allows them

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

This phase should freeze an explicit redaction policy before `debug` profile
ships.

## 8. Retention And Rotation

Recommended defaults:

- prune by run directory, not by line slicing a shared hot file
- keep a bounded number of recent runs
- apply an age cap
- apply a size-aware cap for `debug` profile where needed
- perform pruning best-effort only

Pruning failure rules:

- pruning must never fail hook execution
- pruning failure should surface as a warning or degraded audit signal only

## 9. Performance And Scale

The target is production-grade operation with at least 50 simultaneous agents
on the same repo root.

Design consequences:

- no single shared hot audit file for `full`
- run-scoped sharding for durable audit output
- bounded retention
- no blocking dependency on exporter availability
- non-blocking failure semantics for audit and observability

Required validation before the phase closes:

- concurrency soak coverage
- corruption-free multi-agent writes
- bounded retention behavior under repeated runs
- stable hook outcomes when sinks or pruning fail

## 10. Planned Sprint Sequence

| Phase | Focus | Primary outcomes |
| --- | --- | --- |
| Observability Phase 0 | naming cleanup and namespace freeze | converge on `sc-hooks`, `hooks`, `.sc-hooks/` before more public surface is added |
| Observability Phase 1 | layered config foundation | freeze global/local/env precedence and mode resolution |
| Observability Phase 2 | standard coverage for all hook events | extend lower-volume observability to all hook event types |
| Observability Phase 3 | full audit lean profile | durable run-scoped accounting for evals and harnesses |
| Observability Phase 4 | full audit debug profile and redaction | richer troubleshooting output without losing safety controls |
| Observability Phase 5 | structured live stream and exporter defaults | separate machine stream from human console and wire global exporter defaults |
| Observability Phase 6 | concurrency and production hardening | prove 50+ simultaneous agents and operational safety |

## 11. Sprint Highlights

### Observability Phase 0

- file the naming cleanup as release-blocking
- replace `schook` drift in docs and public-facing surfaces
- freeze `sc-hooks` plus `hooks` alias before adding more config or sink names

### Observability Phase 1

- implement layered config loading
- freeze which keys are global-only vs local-only
- add mode resolution for `off`, `standard`, `full`

### Observability Phase 2

- extend `standard` observability coverage beyond current dispatch-complete
  events
- keep current lower-volume posture for zero-match fast paths unless `full` is
  enabled
- update contracts and integration tests together

### Observability Phase 3

- add `full` lean profile
- emit run-scoped durable audit files under `.sc-hooks/audit/`
- support explicit local path override for evals and harnesses

### Observability Phase 4

- add `debug` profile
- freeze redaction policy
- keep raw payload capture behind an additional explicit opt-in

### Observability Phase 5

- add structured machine-readable streaming if still required
- wire global exporter defaults and future OTel configuration without
  auto-enabling `full`

### Observability Phase 6

- prove 50+ simultaneous agents
- harden retention and pruning
- confirm all degraded paths remain non-blocking

## 12. Open Questions To Freeze Before Code

- exact global config schema for exporter and transport keys
- whether global config can impose explicit ceilings beyond the current
  "cannot enable `full`" rule
- exact field inventory for `lean` vs `debug`
- whether absolute audit paths need additional repo-local restrictions
- whether structured live streaming is needed in Phase 5 or can remain deferred
