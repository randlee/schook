# sc-hooks Project Plan

## 1. Purpose

This document translates the current requirements, traceability table, and
implementation gaps into an execution plan.

It is a derived planning document. It does not override:
- `docs/requirements.md` for release-facing behavior
- `docs/architecture.md` for current architecture
- `docs/protocol-contract.md` for the host/plugin wire contract
- `docs/implementation-gaps.md` for explicit open gaps

## 2. Planning Inputs

This plan is derived from:
- `docs/requirements.md`
- `docs/traceability.md`
- `docs/implementation-gaps.md`
- `docs/architecture.md`

Current open release-relevant drivers are:
- `GAP-001`: compliance harness coverage is below the release contract
- `GAP-002`: `long_running` contract is not aligned across host, SDK, docs, and tests
- `GAP-003`: source plugins are still scaffold/reference crates, not shipped runtime plugins
- `GAP-004`: no checked-in example `.sc-hooks/` runtime layout or setup guide
- `CLI-007`, `TMO-004`, `BND-002`, and `TST-007`: required-before-release items still open

Deferred rather than scheduled for this release plan:
- `GAP-006`
- `DEF-002`
- `DEF-004`

## 3. Current Snapshot

Already implemented and not future sprint work:
- plugin-only runtime resolution from `.sc-hooks/plugins/`
- current `sc-observability` integration at the `sc-hooks-cli` boundary
- observability output documented in `docs/observability-contract.md`
- removal of the old ad hoc logging path and builtin `log` handler path
- release-doc alignment for requirements, architecture, traceability, and gaps

Important planning rule:
- `sc-observability` remains a requirement, but it is already implemented
- therefore observability appears below as a completed baseline sprint, not as pending build work

## 4. Sprint Sequence

| Sprint | Status | Focus | Primary drivers | Depends on |
| --- | --- | --- | --- | --- |
| Sprint 0 | Completed | architecture and observability alignment | `OBS-001`, `OBS-002`, `OBS-006`, `OBS-007`, `OBS-008`, `GAP-005`, `GAP-007` | none |
| Sprint 1 | Planned | compliance harness hardening | `GAP-001`, `CLI-007`, `TST-007` | Sprint 0 |
| Sprint 2 | Planned | `long_running` contract alignment | `GAP-002`, `TMO-004` | Sprint 1 |
| Sprint 3 | Planned | runtime layout and setup proof | `GAP-004`, `CFG-001`, `RES-002`, `CLI-004` | Sprint 1 |
| Sprint 4 | Planned | plugin packaging and release honesty | `GAP-003`, `BND-002` | Sprint 3 |
| Sprint 5 | Planned | merge closeout and release gate | task `#370`, final QA/PR review | Sprints 1-4 |

## 5. Sprint Details

### Sprint 0: Architecture And Observability Alignment

Status:
- completed

Focus:
- remove confusing duplicate logging surfaces
- make `sc-observability` the only current observability path
- align docs to the plugin-only runtime

Deliverables:
- `sc-hooks-cli` emits dispatch events through `sc-observability`
- old in-repo logging code and builtin `log` path are removed
- `docs/observability-contract.md` owns current observability details
- requirements, architecture, traceability, and gaps reflect the implemented boundary

Acceptance criteria:
- `OBS-001`, `OBS-002`, `OBS-006`, `OBS-007`, and `OBS-008` are documented as implemented
- `GAP-005` and `GAP-007` are closed
- `cargo fmt --check --all` and `cargo test --workspace` pass

### Sprint 1: Compliance Harness Hardening

Status:
- planned

Focus:
- make the compliance harness prove the release contract directly
- remove duplicated compliance logic before more behavior is layered on top

Deliverables:
- expand `sc-hooks-test` to assert timeout behavior, invalid JSON, multiple JSON objects, async misuse, matcher validity, and absent-payload behavior
- refactor `sc-hooks-cli test` to delegate to one shared compliance engine
- update `docs/traceability.md` so `CLI-007` and `TST-007` point to direct assertions instead of partial checks

Acceptance criteria:
- `GAP-001` is closed
- `CLI-007` and `TST-007` move from gap to implemented
- duplicated compliance logic in `sc-hooks-cli/src/testing.rs` is retired or reduced to a thin wrapper

### Sprint 2: `long_running` Contract Alignment

Status:
- planned

Focus:
- define one release-grade `long_running` contract across host, SDK, docs, and tests

Deliverables:
- decide whether `sc-hooks-sdk::traits::LongRunning` and `AsyncContextSource` are real release surfaces or stale helpers to retire
- align manifest validation, timeout behavior, audit checks, and docs around one contract
- add end-to-end tests that prove the chosen behavior

Acceptance criteria:
- `GAP-002` is closed
- `TMO-004` moves from required-before-release to implemented
- requirements, architecture, traceability, and SDK surface all describe the same `long_running` behavior

### Sprint 3: Runtime Layout And Setup Proof

Status:
- planned

Focus:
- prove the expected `.sc-hooks/` runtime layout from a clean contributor starting point

Deliverables:
- add either a checked-in example `.sc-hooks/` tree or a setup guide that proves the same layout
- document how `.sc-hooks/config.toml`, `.sc-hooks/plugins/`, and install output fit together
- verify that contributor docs match the actual CLI behavior

Acceptance criteria:
- `GAP-004` is closed
- `CFG-001`, `RES-002`, and `CLI-004` no longer depend on a gap note for practical setup clarity
- a contributor can follow the checked-in example or guide without reading source code

### Sprint 4: Plugin Packaging And Release Honesty

Status:
- planned

Focus:
- keep plugin claims honest unless runtime installation, behavior, and tests exist

Deliverables:
- choose the release posture for each source crate under `plugins/`
- if a plugin is promoted as shipped behavior, add runtime installation guidance and direct behavior tests
- otherwise keep the crate clearly documented as scaffold/reference code

Acceptance criteria:
- `GAP-003` is closed
- `BND-002` is either satisfied for promoted plugins or avoided by keeping release claims scoped to scaffold/reference status only
- README and docs agree on the exact plugin inventory and maturity level

### Sprint 5: Merge Closeout And Release Gate

Status:
- planned

Focus:
- close known merge-time review items and freeze the release-doc set

Deliverables:
- resolve task `#370` items:
- `RV-001`: requirements section 2.1 plugin-count prose fix
- `RV-002`: retire `EXC-009` and `CLI-008` "0-10" claim at merge
- final doc/code consistency check
- final reviewer and QA handoff

Acceptance criteria:
- merge-time review items are closed
- no open blocker gaps remain for the chosen release scope
- branch handoff records exact validation commands and reviewer status

## 6. Out Of Scope For This Plan

These items stay deferred unless product direction changes:
- richer `fire` output beyond the current summary string
- finer-grained resolution-time exit codes
- SDK ergonomics beyond the current host-enforced contract
- production-ready bundled plugin behavior beyond whatever Sprint 4 explicitly promotes

## 7. Release Gate

The release plan is complete only when:
- all non-deferred blocker or important gaps are either closed or explicitly removed from release scope
- every required-before-release item is either implemented or intentionally cut from the release contract
- traceability rows for release claims point to real code and tests, not inference alone
- merge-time review items are closed
- reviewer and QA signoff are recorded on the final branch state
