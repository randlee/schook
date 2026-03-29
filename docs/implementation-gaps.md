# sc-hooks Implementation Gaps

This document tracks gaps between the current codebase and the release-standard documentation set.

## Gap Closure Tracker

| Gap | Severity | Owner area | Verification method | Early retire / replace candidates |
| --- | --- | --- | --- | --- |
| DEF-001 | deferred | docs, plugin source crates, release packaging | deferred table, README, architecture, and gaps all keep scaffold plugins out of the shipped baseline | none until a plugin is promoted with runtime proof |
| DEF-002 | deferred | `sc-hooks-cli`, docs | `fire` docs and implementation continue to describe summary-string output only | none until a structured fire report is intentionally designed |
| DEF-003 | deferred | `sc-hooks-sdk`, docs | SDK docs, requirements, and gaps keep richer `LongRunning` ergonomics deferred beyond the current manifest-driven host contract | see `GAP-002` |
| GAP-006 | deferred | `sc-hooks-cli`, `sc-hooks-core` | Exit-code tests and docs agree on any future split | none until the exit taxonomy changes |
| GAP-008 | deferred | docs, `sc-hooks-cli` | Requirements, architecture, and gaps all state that builtin handler resolution is intentionally out of scope for the current release | none until the product intentionally restores builtins |
| GAP-009 | deferred | docs, `sc-hooks-cli` | Requirements, architecture, observability docs, and gaps all state that `[logging]` config was intentionally removed during the `sc-observability` migration | none until sink configuration is intentionally restored |
| GAP-010 | resolved in this pass | `sc-hooks-cli`, docs | host-level observability contract tests prove success, block, invalid-json error, timeout, and file-sink path behavior through the real `sc-hooks-cli` binary | broader sink/monitoring coverage remains tracked under `DEF-008` |
| DEF-007 | deferred | docs, `sc-hooks-sdk` | requirements and protocol contract keep the extended payload-condition operator set out of the release-facing contract until explicitly promoted | none until the operator set is elevated into the release contract |
| DEF-008 | deferred | docs, `sc-hooks-cli`, `sc-observability` integration | requirements, architecture, observability docs, and gaps keep richer observability validation beyond the current file-sink dispatch contract explicitly planned but not release-blocking, with console-sink coverage named as the first follow-up | none until console/custom sink coverage and multi-hook smoke correlation are intentionally promoted |

## Resolved In This Pass

- Sprint 8 RBP follow-up closed the last documented best-practices review residue by deleting dead condition-validation code in `sc-hooks-sdk/src/conditions.rs`, promoting the already-implemented audit findings `AUD-005` and `AUD-009` into the requirements/traceability set, and documenting the dispatch stderr fallback when observability emission fails.
- `GAP-001` resolved by expanding `sc-hooks-test` with shared host-dispatch contract scenarios and proving them through the actual `sc-hooks-cli` binary in `sc-hooks-cli/tests/compliance_host.rs`.
- `GAP-002` resolved by making `long_running` a sync-only manifest/runtime contract, aligning timeout handling and handler discovery with that rule, and keeping SDK runner defaults explicitly non-normative.
- `GAP-003` resolved by freezing every current `plugins/` source crate as scaffold/reference only in release-facing docs and plugin Cargo metadata.
- `GAP-004` resolved by checking in `examples/runtime-layout/.sc-hooks/`, documenting it as the canonical contributor setup path, and proving it with `sc-hooks-cli/tests/runtime_layout_example.rs`.
- `GAP-005` resolved by removing the mixed ad hoc logger surfaces and emitting one `sc-observability` `LogEvent` shape only.
- `GAP-007` resolved by adopting the external `sc-observability` workspace referenced by `sc-hooks-cli/Cargo.toml` at `../../../sc-observability/...` and making that boundary current architecture.
- `GAP-010` resolved by adding `sc-hooks-cli/tests/observability_contract.rs`, which drives the real `sc-hooks-cli` binary through success, block, invalid-json error, and timeout dispatches, then asserts the emitted `sc-observability` JSONL contract and file-sink path.
- `OBS-003` and `OBS-004` are retired requirement IDs from earlier ad hoc logging drafts; the current observability contract is represented by `OBS-001`, `OBS-002`, `OBS-005`, `OBS-006`, `OBS-007`, and `OBS-008`, with the migration closures recorded under `GAP-005` and `GAP-007`.
- Task `#370` was a Sprint 6 merge-review tracker, not a release-facing requirement or gap ID. It was retired by freeze commit `cdce7b1` when `docs/project-plan.md` replaced the specific stale text `Current open release-relevant drivers are: merge-time review residue tracked under task #370` with `none; release-facing blocker and important gaps are closed for the chosen scope`, and replaced the Sprint 6 driver text `task #370, final QA/PR review` with `final reviewer/QA handoff`.

## Resolved Gaps

### GAP-001: Compliance Harness Overclaims Coverage (Resolved In Sprint 2)

- Severity: `blocker`
- Source: `CLI-007`, `TST-007`
- Owner area:
  - `sc-hooks-test`, `sc-hooks-cli`
- Current behavior:
  - `sc-hooks-cli test` continues to delegate to the shared `sc-hooks-test` compliance engine instead of maintaining a second implementation.
  - `sc-hooks-test::compliance::run_contract_behavior_suite` now asserts timeout, invalid stdout, multi-object stdout warnings, async block misuse, matcher filtering, and absent-payload handling through the real `sc-hooks-cli` dispatch path.
- Expected behavior:
  - The reusable compliance harness directly verifies the behaviors the release docs promise, including async misuse, timeout behavior, invalid JSON, multi-object stdout handling, matcher filtering, and real absent-payload behavior.
- Verification method:
  - direct compliance assertions for timeout, invalid output, async misuse, matcher validity, absent-payload behavior, and multi-object stdout warnings
- Recommended fix path:
  - Keep `sc-hooks-cli test` as a thin presentation layer over the shared compliance engine while the host-path contract suite remains in `sc-hooks-test`.
- Early retire / replace candidates:
  - duplicate compliance logic in `sc-hooks-cli/src/testing.rs` remains retired after Sprint 1
  - the duplicate absent-payload pseudo-check in `sc-hooks-test/src/compliance.rs` stays removed; host-path absent-payload proof now replaces it

### GAP-002: SDK Surface Does Not Yet Match Host Reality Cleanly (Resolved In Sprint 3)

- Severity: `important`
- Source: `TMO-004`
- Owner area:
  - `sc-hooks-sdk`, `sc-hooks-cli`, docs
- Current behavior:
  - The host treats `long_running` as a sync-only manifest contract.
  - Sync `long_running=true` removes the default sync timeout when no explicit `timeout_ms` override is set.
  - Async manifests using `long_running=true` are rejected during manifest validation, resolution, and audit.
  - the stale `sc-hooks-sdk::traits::LongRunning` and `AsyncContextSource` surfaces are retired so the SDK no longer implies a richer contract than the host actually guarantees today.
  - `sc-hooks-sdk::runner::PluginRunner` also includes convenience behavior such as treating empty stdin as `{}`, which is useful for authoring but is not itself the release-defining host contract.
  - the SDK remains a thin authoring surface: manifest helpers can express the valid sync-only contract, but runner fallback defaults are still convenience behavior rather than host guarantees.
- Expected behavior:
  - The docs, SDK convenience surface, and tests agree on one release-grade posture: the host contract is manifest-driven and sync-only for `long_running`, while runner conveniences stay out of the normative release contract.
- Verification method:
  - one end-to-end SDK posture proven across manifest validation, runtime behavior, docs, and tests
- Recommended fix path:
  - Keep the SDK posture narrow and document runner-helper limits anywhere the SDK is presented as an authoring path so convenience defaults are not mistaken for host guarantees.
- Early retire / replace candidates:
  - `sc-hooks-sdk::traits::LongRunning` remains retired after Sprint 1
  - `sc-hooks-sdk::traits::AsyncContextSource` remains retired after Sprint 1
  - any SDK helper behavior that reads like contract-defining runtime semantics without corresponding host guarantees

## GAP-003: Bundled Plugin Readiness Was Previously Overstated (Resolved In Sprint 5)

- Severity: `important`
- Source: `BND-001`, `BND-002`
- Owner area:
  - docs, plugin source crates, release packaging
- Current behavior:
  - Source crates under `plugins/` respond to `--manifest`, read stdin, and return `{\"action\":\"proceed\"}`.
  - Runtime plugin discovery does not read from `plugins/`; it reads from `.sc-hooks/plugins/`.
  - README, architecture, requirements, and plugin `Cargo.toml` metadata now mark every current source crate as scaffold/reference only and explicitly not shipped runtime functionality.
- Expected behavior:
  - The docs must describe these crates as scaffolds or reference implementations until they ship real behavior, installation guidance, and direct tests.
- Verification method:
  - supported-plugin claims match runtime installation, behavior, and tests
- Recommended fix path:
  - Keep the docs and Cargo metadata honest now; later either promote specific plugins to supported runtime artifacts with install/runtime proof and direct behavior tests, or keep them reference-only.
- Early retire / replace candidates:
  - old "bundled plugin" language in contributor-facing docs and release notes

## GAP-004: No Checked-In Example Runtime Layout (Resolved In Sprint 4)

- Severity: `important`
- Source: `CFG-001`, `RES-002`, `CLI-004`
- Owner area:
  - docs, examples/setup, `sc-hooks-cli`
- Current behavior:
  - The host expects `.sc-hooks/config.toml` and `.sc-hooks/plugins/`.
  - The repository now includes a checked contributor/runtime example at `examples/runtime-layout/.sc-hooks/` with a validating `guard-paths` plugin and a host-level test that audits and runs it successfully.
- Expected behavior:
  - Contributors have a minimal documented example config and runtime plugin layout that can be copied or inspected without reading source code.
- Verification method:
  - a checked-in example or setup guide proves the expected `.sc-hooks/` runtime layout
- Recommended fix path:
  - Keep the checked example and runtime-layout test updated whenever install/runtime assumptions change.

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

### DEF-008: Broader Observability Monitoring Coverage Stays Deferred

- Current behavior:
  - the current release baseline proves the file-sink `LogEvent` contract under
    real dispatch for success, block, invalid-json error, and timeout paths
  - `sc-hooks-core` currently exports `OBSERVABILITY_ROOT` and
    `OBSERVABILITY_LOG_PATH` as shared path literals for agreement between the
    CLI, tests, and docs; this is an accepted OBS-007 boundary tension because
    the constants do not own sink wiring, logger lifecycle, or event emission
  - the next planned observability follow-up is console-sink coverage under
    real dispatch because console logs are the most useful immediate debugging
    surface for live/background-agent interaction tracing
  - the baseline does not yet prove:
    - console-sink behavior under `sc-observability`
    - custom sink registration paths
    - multi-hook sequence correlation / exactly-once smoke monitoring across a
      longer lifecycle
    - operator-facing monitoring flows such as background-agent log watching
- Exit condition:
  - requirements, architecture, observability docs, and tests intentionally
    promote richer monitoring coverage, starting with console-sink behavior and
    then extending to custom sinks and multi-hook smoke correlation

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
