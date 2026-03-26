# sc-hooks Implementation Gaps

This document tracks gaps between the current codebase and the release-standard documentation set.

## Gap Closure Tracker

| Gap | Severity | Owner area | Verification method | Early retire / replace candidates |
| --- | --- | --- | --- | --- |
| GAP-001 | blocker | `sc-hooks-test`, `sc-hooks-cli` | Direct compliance assertions for timeout, invalid output, async misuse, matcher validity, and absent-payload behavior | retire duplicated compliance logic in `sc-hooks-cli/src/testing.rs`; remove the duplicate absent-payload pseudo-check in `sc-hooks-test/src/compliance.rs` |
| GAP-002 | important | `sc-hooks-sdk`, `sc-hooks-cli`, docs | One end-to-end SDK posture proven across manifest validation, runtime behavior, docs, and tests | retire or replace public-looking SDK traits and document runner-helper limits unless they become real release-contract surfaces |
| GAP-003 | important | docs, plugin source crates, release packaging | Supported-plugin claims match runtime installation, behavior, and tests | retire old "bundled plugin" language before promoting any source crate to shipped behavior |
| GAP-004 | important | docs, examples/setup, `sc-hooks-cli` | A checked-in example or setup guide proves the expected `.sc-hooks/` runtime layout | none yet |
| DEF-001 | deferred | docs, plugin source crates, release packaging | deferred table, README, architecture, and gaps all keep scaffold plugins out of the shipped baseline | none until a plugin is promoted with runtime proof |
| DEF-002 | deferred | `sc-hooks-cli`, docs | `fire` docs and implementation continue to describe summary-string output only | none until a structured fire report is intentionally designed |
| DEF-003 | deferred | `sc-hooks-sdk`, docs | SDK docs, requirements, and gaps keep richer `LongRunning` ergonomics deferred beyond the current manifest-driven host contract | see `GAP-002` |
| GAP-006 | deferred | `sc-hooks-cli`, `sc-hooks-core` | Exit-code tests and docs agree on any future split | none until the exit taxonomy changes |
| GAP-008 | deferred | docs, `sc-hooks-cli` | Requirements, architecture, and gaps all state that builtin handler resolution is intentionally out of scope for the current release | none until the product intentionally restores builtins |
| GAP-009 | deferred | docs, `sc-hooks-cli` | Requirements, architecture, observability docs, and gaps all state that `[logging]` config was intentionally removed during the `sc-observability` migration | none until sink configuration is intentionally restored |
| DEF-007 | deferred | docs, `sc-hooks-sdk` | requirements and protocol contract keep the extended payload-condition operator set out of the release-facing contract until explicitly promoted | none until the operator set is elevated into the release contract |

## Resolved In This Pass

- `GAP-005` resolved by removing the mixed ad hoc logger surfaces and emitting one `sc-observability` `LogEvent` shape only.
- `GAP-007` resolved by adopting the external `sc-observability` workspace referenced by `sc-hooks-cli/Cargo.toml` at `../../../sc-observability/...` and making that boundary current architecture.
- `OBS-003` and `OBS-004` are retired requirement IDs from earlier ad hoc logging drafts; the current observability contract is represented by `OBS-001`, `OBS-002`, `OBS-005`, `OBS-006`, `OBS-007`, and `OBS-008`, with the migration closures recorded under `GAP-005` and `GAP-007`.

## GAP-001: Compliance Harness Overclaims Coverage

- Severity: `blocker`
- Source: `CLI-007`, `TST-007`
- Owner area:
  - `sc-hooks-test`, `sc-hooks-cli`
- Current behavior:
  - `sc-hooks-cli test` now delegates to the shared `sc-hooks-test` compliance engine instead of maintaining a second implementation.
  - the surviving compliance engine verifies manifest loading, basic contract compatibility, simple matcher checks, positive timeout shape, and minimal JSON output.
- Expected behavior:
  - The reusable compliance harness should directly verify the behaviors the release docs promise, including async misuse, timeout behavior, invalid JSON, multi-object stdout handling, and real absent-payload behavior.
- Verification method:
  - direct compliance assertions for timeout, invalid output, async misuse, matcher validity, and absent-payload behavior
- Recommended fix path:
  - Expand `sc-hooks-test` first, then align `docs/traceability.md` with direct assertions.
  - Keep `sc-hooks-cli test` as a thin presentation layer over the shared compliance engine.
- Early retire / replace candidates:
  - duplicate compliance logic in `sc-hooks-cli/src/testing.rs` is retired in this sprint
  - the duplicate absent-payload pseudo-check in `sc-hooks-test/src/compliance.rs` is retired in this sprint

## GAP-002: SDK Surface Does Not Yet Match Host Reality Cleanly

- Severity: `important`
- Source: `TMO-004`
- Owner area:
  - `sc-hooks-sdk`, `sc-hooks-cli`, docs
- Current behavior:
  - The host honors manifest fields `long_running`, `timeout_ms`, and `description`.
  - Audit rejects `long_running=true` for async handlers and requires a non-empty description.
  - the stale `sc-hooks-sdk::traits::LongRunning` and `AsyncContextSource` surfaces are retired so the SDK no longer implies a richer contract than the host actually guarantees today.
  - `sc-hooks-sdk::runner::PluginRunner` also includes convenience behavior such as treating empty stdin as `{}`, which is useful for authoring but is not itself the release-defining host contract.
- Expected behavior:
  - The docs, SDK convenience surface, and tests should agree on one release-grade SDK posture: either thin authoring conveniences with clearly documented limits, or a fuller public contract that is actually proven end to end.
- Verification method:
  - one end-to-end SDK posture proven across manifest validation, runtime behavior, docs, and tests
- Recommended fix path:
  - Treat `long_running` as a host manifest feature for now and keep the SDK posture narrow until Sprint 3 aligns the end-to-end contract.
  - Document runner-helper limits anywhere the SDK is presented as an authoring path so convenience defaults are not mistaken for host guarantees.
- Early retire / replace candidates:
  - `sc-hooks-sdk::traits::LongRunning` is retired in this sprint
  - `sc-hooks-sdk::traits::AsyncContextSource` is retired in this sprint
  - any SDK helper behavior that reads like contract-defining runtime semantics without corresponding host guarantees

## GAP-003: Bundled Plugin Readiness Was Previously Overstated

- Severity: `important`
- Source: `BND-001`, `BND-002`
- Owner area:
  - docs, plugin source crates, release packaging
- Current behavior:
  - Source crates under `plugins/` respond to `--manifest`, read stdin, and return `{\"action\":\"proceed\"}`.
  - Runtime plugin discovery does not read from `plugins/`; it reads from `.sc-hooks/plugins/`.
- Expected behavior:
  - The docs must describe these crates as scaffolds or reference implementations until they ship real behavior, installation guidance, and direct tests.
- Verification method:
  - supported-plugin claims match runtime installation, behavior, and tests
- Recommended fix path:
  - Keep the docs honest now; later either promote specific plugins to supported runtime artifacts or move them to an examples-only posture.
- Early retire / replace candidates:
  - old "bundled plugin" language in contributor-facing docs and release notes

## GAP-004: No Checked-In Example Runtime Layout

- Severity: `important`
- Source: `CFG-001`, `RES-002`, `CLI-004`
- Owner area:
  - docs, examples/setup, `sc-hooks-cli`
- Current behavior:
  - The host expects `.sc-hooks/config.toml` and `.sc-hooks/plugins/`, but the repository does not currently include a checked-in example runtime layout.
- Expected behavior:
  - Contributors should have a minimal documented example config and runtime plugin layout.
- Verification method:
  - a checked-in example or setup guide proves the expected `.sc-hooks/` runtime layout
- Recommended fix path:
  - Add a minimal example `.sc-hooks/` tree or a clearly linked setup guide before release.

## Deferred Item Acknowledgments

### DEF-001: Production-Ready Bundled Plugins Stay Deferred

- Current behavior:
  - source crates under `plugins/` remain scaffold/reference implementations
- Exit condition:
  - real runtime behavior, installation guidance, and direct tests exist for any plugin promoted as shipped behavior

### DEF-002: Richer `fire` Diagnostics Stay Deferred

- Current behavior:
  - `sc-hooks fire` returns a short summary string rather than a structured diagnostics report
- Exit condition:
  - a stable structured `fire` output format is implemented and tested

### DEF-003: Richer SDK `LongRunning` Ergonomics Stay Deferred

- Current behavior:
  - richer SDK-level `LongRunning` ergonomics remain subordinate to the current manifest-driven host contract
- Exit condition:
  - the SDK, docs, and tests agree on a stable public contract beyond the current host behavior

### DEF-007: Extended Payload-Condition Operators Stay Deferred

- Current behavior:
  - code may accept operators beyond the `PLC-002` set, but those extra operators are not part of the release-facing contract
- Exit condition:
  - requirements, protocol docs, and tests are updated together for the expanded operator set

## GAP-006: Exit-Code Taxonomy Is Coarse Around Resolution-Time Manifest Failures

- Severity: `deferred`
- Source: `EXC-004`, `DEF-004`
- Owner area:
  - `sc-hooks-cli`, `sc-hooks-core`
- Current behavior:
  - unresolved handlers and manifest-load or manifest-compatibility failures all map to the same resolution exit code (`4`)
- Expected behavior:
  - if the project wants finer-grained operational diagnosis, manifest incompatibility may eventually deserve a dedicated exit code
- Verification method:
  - exit-code tests and docs agree on any future split
- Recommended fix path:
  - keep the current behavior documented honestly unless and until the codebase introduces a new exit-code split

## GAP-008: Builtin Handler Resolution Is Intentionally Out Of Scope

- Severity: `deferred`
- Source: `RES-001`, `DEF-005`
- Owner area:
  - docs, `sc-hooks-cli`
- Current behavior:
  - the runtime resolves configured handler names only through `.sc-hooks/plugins/`
  - there is no builtin resolution path in the dispatcher
- Expected behavior:
  - the docs should state explicitly that builtin handler resolution was removed from the current release baseline and is deferred unless the product intentionally restores it
- Verification method:
  - requirements, architecture, and gaps all state that builtin handler resolution is intentionally out of scope for the current release
- Recommended fix path:
  - keep the plugin-only runtime explicit unless the product intentionally reintroduces builtins with a documented precedence and lifecycle model

## GAP-009: `[logging]` Config Was Intentionally Removed During Observability Migration

- Severity: `deferred`
- Source: `OBS-002`, `DEF-006`
- Owner area:
  - docs, `sc-hooks-cli`
- Current behavior:
  - the CLI no longer supports a `[logging]` section in `.sc-hooks/config.toml`
  - observability output is routed through the fixed `sc-observability` CLI boundary instead of config-driven sink wiring
- Expected behavior:
  - the docs should state explicitly that `[logging]` config was intentionally removed from the current release baseline during the `sc-observability` migration
- Verification method:
  - requirements, architecture, observability docs, and gaps all state that `[logging]` config was intentionally removed during the `sc-observability` migration
- Recommended fix path:
  - keep sink routing fixed at the CLI boundary unless the product intentionally restores supported configuration keys and their contract
