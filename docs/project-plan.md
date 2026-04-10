# sc-hooks Project Plan

## 1. Purpose

This document translates the current requirements, traceability table, and
implementation gaps into an execution plan.

It is a derived planning document. It does not override:
- `docs/requirements.md` for release-facing behavior
- `docs/architecture.md` for current architecture
- `docs/protocol-contract.md` for the host/plugin wire contract
- `docs/implementation-gaps.md` for archived gap context

## 2. Planning Inputs

This plan is derived from:
- `docs/requirements.md`
- `docs/traceability.md`
- `docs/implementation-gaps.md`
- `docs/architecture.md`

Current open release-relevant drivers are:
- naming cleanup before further public observability/global-config surface is
  added
- a multi-sprint observability phase that extends beyond the current
  dispatch-only file-sink baseline

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

Explicit follow-up after the current file-sink contract work:
- naming cleanup now precedes further observability surface growth so config,
  filesystem, binary, and service names converge on `sc-hooks`
- console-sink coverage is already part of the proved baseline and no longer
  stands alone as the next observability milestone
- the committed observability phase covers naming cleanup, layered config, full
  audit, redaction, retention, and production-load hardening
- future exporter or OTel defaults remain follow-on scope after that committed
  phase closes

Important planning rule:
- `sc-observability` remains a requirement, but it is already implemented
- therefore observability appears below first as a completed baseline sprint,
  not as pending build work
- the planned observability phase extends that baseline with naming cleanup,
  layered config, and audit-grade coverage; it does not reopen the original
  `sc-observability` adoption decision

## 4. Sprint Sequence

| Sprint | Status | Focus | Primary drivers | Depends on | Primary write scope |
| --- | --- | --- | --- | --- | --- |
| Sprint 0 | Completed | architecture and observability alignment | `OBS-001`, `OBS-002`, `OBS-006`, `OBS-007`, `OBS-008`, `GAP-005`, `GAP-007` | none | `sc-hooks-cli`, observability docs, release docs |
| Sprint 1 | In review | baseline alignment and code retirement | `GAP-001`, `GAP-002`, `GAP-003` | Sprint 0 | `sc-hooks-cli/src/testing.rs`, `sc-hooks-test`, `sc-hooks-sdk`, release docs |
| Sprint 2 | In review - fix-r1 pushed | compliance harness hardening | `GAP-001`, `CLI-007`, `TST-007` | Sprint 1 | `sc-hooks-test`, `sc-hooks-cli/src/testing.rs`, dispatch/runtime contract tests |
| Sprint 3 | In review | `long_running` contract alignment | `GAP-002`, `TMO-004` | Sprint 1 | `sc-hooks-sdk`, timeout/dispatch flow, requirements/architecture/traceability |
| Sprint 4 | In review | runtime layout and setup proof | `GAP-004`, `CFG-001`, `RES-002`, `CLI-004` | Sprint 2 | install/runtime layout docs, example `.sc-hooks/` tree, contributor path |
| Sprint 5 | In review | plugin packaging and release honesty | `GAP-003`, `BND-002` | Sprint 4 | `plugins/`, install/release docs, runtime packaging checks |
| Sprint 6 | In review | release freeze and final QA handoff | final reviewer/QA handoff | Sprints 2-5 | release docs, PR/review records, final cleanup |
| Sprint 8 | In review | Rust best-practices closeout | `AUD-005`, `AUD-009`, `OBS-005`, `SCHOOK-QA-001` | Sprint 6 | `sc-hooks-sdk`, `sc-hooks-cli`, release docs |
| Hook Phase 0 | In review | hook review baseline | `HKR-001`, `HKR-002`, `HKR-003`, `HKR-006`, `HKR-007` | Sprint 6 formally accepted | hook API docs, `docs/archive/plugin-plan-s9.md`, `docs/requirements.md`, `docs/architecture.md` |
| Hook Phase 1 | Planned | Claude schema harness | `HKR-002`, `HKR-005` | Hook Phase 0 | `test-harness/hooks/README.md`, `test-harness/hooks/claude/`, harness models, fixtures, reports |
| Hook Phase 2 | Planned | plan revision from captured Claude schema | `HKR-003` | Hook Phase 1 | `docs/archive/plugin-plan-s9.md`, `docs/hook-api/claude-hook-api.md`, readiness notes |
| Hook Phase 3 | Planned | session foundation and trait freeze | `HKR-004`, `HKR-008`, `HKR-009`, `HKR-012` | Hook Phase 2 | `sc-hooks-core`, `sc-hooks-sdk`, `plugins/agent-session-foundation`, same-PR architecture inventory update |
| Hook Phase 4 | Planned | generic spawn and tool gates | `HKR-010`, `HKR-011`, `HKR-013` | Hook Phase 3 | `plugins/agent-spawn-gates`, `plugins/tool-output-gates`, direct behavior tests |
| Hook Phase 5 | Planned | ATM extension behaviors | `HKR-010`, `HKR-011` | Hook Phase 3 | `plugins/atm-extension`, ATM relay and identity tests |
| Hook Phase 6 | Planned | post-Claude follow-on planning only | `HKR-006`, `HKR-007` | Hook Phase 5 plus separate approval | provider follow-on planning docs only |
| S10-VERSION-BUMP-1 | Completed | Claude version-bump detection | `TST-008` | Hook Phase 1 | `scripts/verify-claude-hook-api.py`, `test-harness/hooks/claude/fixtures/approved/manifest.json`, release docs |
| S11-DOC.1 | In review | README/usage guide release-doc alignment | `SCHOOK-QA-001`, `SCHOOK-QA-002`, `SCHOOK-QA-003`, `SCHOOK-QA-004`, `SCHOOK-QA-005` | none | `README.md`, `USAGE.md`, `docs/project-plan.md` |
| S12-PUB.1 | In review | workspace publish prep and release infrastructure | release packaging alignment | `develop` baseline | `crates/`, `release/`, `.github/workflows/`, `PUBLISHING.md`, release docs |
| `SC-LOG-S1` / Observability Phase 0 | Merged | naming cleanup and namespace freeze | release blocker #88, `DEF-019` | `develop` baseline | naming docs, binary/service references, config/runtime namespace decisions |
| `SC-LOG-S2` / Observability Phase 1 | Merged | layered config foundation | `DEF-010`, `DEF-011` | `SC-LOG-S1` | `sc-hooks-cli` config loading, requirements/architecture docs, config tests |
| `SC-LOG-S3` / Observability Phase 2 | Merged | standard observability coverage for all hook events | `DEF-011`, `DEF-017`, `HKR-009` | `SC-LOG-S2` | `sc-hooks-cli` hook runtime, observability tests, contract docs |
| `SC-LOG-S4` / Observability Phase 3 | Merged | full audit lean profile | `DEF-012`, `DEF-013`, `DEF-017` | `SC-LOG-S3` | audit writer, `.sc-hooks/audit/` layout, eval or harness integration tests |
| `SC-LOG-S5` / Observability Phase 4 | Merged | full audit debug profile and redaction controls | `DEF-013`, `DEF-014` | `SC-LOG-S4` | redaction policy, payload-capture gates, debug-profile tests |
| `SC-LOG-S6` / Observability Phase 5 | Merged | retention, pruning, and degraded-path hardening | `DEF-009`, `DEF-012`, `DEF-014`, `DEF-015` | `SC-LOG-S5` | retention pruning, degraded-path tests, operational docs |
| `SC-LOG-S7` / Observability Phase 6 | Completed | concurrency and production hardening | `DEF-016` | `SC-LOG-S6` | soak/load harness, operational validation, phase-close evidence |
| `SC-LOG-PHASE-END` | In review | PRR closeout and QA follow-up corrections | `BP-TS-001`, `BP-TS-002`, coverage and phase-end release-readiness findings | `SC-LOG-S7` | targeted runtime guards, coverage hardening, release/docs corrections |

## 5. Execution Controls

These rules exist to keep sprint work from drifting back into mixed designs:

1. Start each sprint by deleting or isolating obsolete code paths before adding new behavior on top.
2. Do not run parallel work in the same write scope. Parallel work is allowed only when file ownership is disjoint.
3. No sprint closes if the implementation still leaves two competing paths for the same release behavior.
4. Any code intentionally kept for compatibility must be named in the sprint deliverables and release gate, not left implicit.
5. Each sprint must update `docs/traceability.md` for any requirement or gap whose status changes.

## 6. Dependency And Parallelism Rules

- Sprint 1 is a cleanup sprint. It exists to remove false surfaces before capability work begins.
- Sprint 2 and Sprint 3 must not start until Sprint 1 closes, because both depend on a cleaned baseline with one owned implementation path per behavior.
- Sprint 4 depends on Sprint 2 because setup proof should reflect the surviving compliance/runtime path, not the pre-cleanup shape.
- Sprint 5 must not start until Sprint 4 freezes the expected runtime layout; otherwise plugin packaging claims drift from the documented install path.
- Sprint 6 is not feature work. It is only closeout, deletion of stale review notes, and final release gating.
- `SC-LOG-S1` must close before any later observability sprint, because naming choices feed the binary name, config keys, service identity, and on-disk audit paths.
- `SC-LOG-S2` must close before `SC-LOG-S3` through `SC-LOG-S7`, because mode resolution and layered config define which observability surfaces exist and where they are configured.
- `SC-LOG-S4` must close before `SC-LOG-S5`, because the debug profile is an extension of the lean audit profile rather than a separate sink family.
- `SC-LOG-S6` must close before `SC-LOG-S7`, because load and soak validation must target the final retention and degradation semantics rather than a partial design.

## 7. Pre-Sprint Kickoff Checklist

Before any sprint starts, record these items in the sprint handoff or working notes:

- exact requirement IDs and gap IDs in scope
- code or docs to delete before new behavior is added
- the single owning implementation path for the sprinted behavior
- the tests expected to fail before the sprint and pass after it
- the docs that must change in the same PR as the code
- the files or crates that define the sprint write scope

## 8. Remove/Replace Matrix

| Area | Current ambiguity or stale path | Planned action | Sprint | Verification |
| --- | --- | --- | --- | --- |
| Compliance flow | `sc-hooks-test/src/compliance.rs` and `sc-hooks-cli/src/testing.rs` both encode overlapping compliance logic | first freeze one owning path and delete pseudo-checks that do not prove real contract behavior; then expand the surviving engine | Sprint 1 then Sprint 2 | `CLI-007` and `TST-007` point to the same underlying checks |
| SDK public-looking surface | `sc-hooks-sdk/src/traits.rs` and `sc-hooks-sdk/src/runner.rs` can imply a richer or broader contract than the host actually guarantees | first decide keep-vs-retire posture, then align surviving SDK helpers and their documented limits with docs/tests | Sprint 1 then Sprint 3 | `GAP-002` and `TMO-004` close with one documented SDK posture |
| Instruction docs drift | derived onboarding/agent docs can repeat superseded rules such as builtin handler precedence | correct derived instructions immediately and treat source-of-truth docs as authoritative for runtime behavior | Sprint 1 | README, `CLAUDE.md`, and source-of-truth docs make the same runtime claims |
| Runtime setup guidance | source layout exists but contributor/runtime setup proof is incomplete | replace inference-only setup with a checked example or one canonical guide | Sprint 4 | `GAP-004` closes and a clean setup succeeds without source reading |
| Plugin release claims | source crates under `plugins/` are not uniformly shippable runtime plugins | first freeze scaffold/reference posture, then promote only with tests/install docs if desired | Sprint 1 then Sprint 5 | `GAP-003` and `BND-002` are resolved without mixed claims |
| Release handoff freeze | stale review-only notes can linger after implementation finishes even when the underlying work is done | remove stale review placeholders, confirm no open blocker gaps remain, and freeze one validation record for QA/review | Sprint 6 | final branch head has no stale review-only requirement notes |

## 9. Misalignment Coverage Signal

This planning pass is considered complete only when every high-risk
misalignment class is either already resolved or mapped into an explicit
sprint/gap path.

Current high-risk classes covered here:
- duplicate compliance source-of-truth logic
- SDK public-looking surface that can overstate the host contract
- derived instruction docs that can drift from source-of-truth docs
- scaffold plugin claims that can be mistaken for shipped runtime behavior
- runtime layout/setup assumptions that are not yet proven by an example or guide
- remaining merge-only review residue that can survive after the underlying issue is already resolved
- naming drift across repo, binary, service, and filesystem surfaces
- observability surface drift between current operational logging, planned full audit, and future exporter wiring
- hot-file or shared-sink contention risk for multi-agent audit runs
- accidental reuse of the human console sink as a machine-readable contract

Merge signal for this docs/planning pass:
- no additional high-risk misalignment class is known that is not already resolved or explicitly represented in this plan or `docs/implementation-gaps.md`

## 10. Sprint Details

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

### Planned Track: Observability Phase

Status:
- planned

Focus:
- freeze naming and config surfaces before expanding observability volume
- add a layered config model with global defaults at `~/.sc-hooks/config.toml`
  and repo-local overrides at `.sc-hooks/config.toml`
- preserve the current lower-volume `standard` dispatch log while adding an
  explicitly local-only `full` audit mode
- make the audit path durable, machine-readable, redaction-aware, and safe for
  50+ simultaneous agents

Phase-wide fixed decisions:
- canonical product/runtime/binary/docs name converges on `sc-hooks`
- convenience CLI alias is `hooks`
- filesystem namespace stays `.sc-hooks/`
- `full` audit is never enabled from global config alone
- observability or audit failure never changes hook execution behavior
- load and soak validation belongs in dedicated integration and phase-end QA
  coverage, not in flaky or long-running unit tests
- durable audit JSONL is the machine contract for the committed phase; a
  separate live structured stream remains out of scope for phase acceptance
- future exporter or OTel defaults are follow-on work after this committed
  phase, not part of its exit gate

Detailed design and sprint sequencing for this track lives in
`docs/phase-observability-plan.md`.

### Sprint 1: Baseline Alignment And Code Retirement (In Review)

Status:
- in review

Focus:
- remove false or confusing public-looking surfaces before feature work starts
- freeze the baseline so later sprints build on the shape we actually intend to keep

Write scope:
- `sc-hooks-test/src/compliance.rs`
- `sc-hooks-cli/src/testing.rs`
- `sc-hooks-sdk/src/traits.rs`
- release docs and implementation-gap notes tied to surviving surfaces

Early retire or replace:
- duplicate compliance paths that suggest two sources of truth
- pseudo-checks that do not prove the documented contract
- public-looking SDK traits that are not part of the real runtime contract
- SDK helper defaults that can be mistaken for host guarantees
- stale onboarding or agent instructions that repeat superseded runtime rules
- ambiguous plugin language that overstates scaffold crates as shipped behavior

Deliverables:
- decide the single owning compliance path and retire or reduce the duplicate path before expanding coverage
- decide whether `sc-hooks-sdk` traits and runner helpers are thin SDK conveniences to keep or stale public-looking surfaces to remove or narrow
- verify or explicitly gap any remaining release-facing observability claims that are still advisory-only
- align derived instruction docs such as `README.md` and `CLAUDE.md` to the current plugin-only runtime and JSON-defined public contract
- document SDK helper limits anywhere the repo presents `sc-hooks-sdk` as an authoring path
- freeze `plugins/` as scaffold/reference only unless and until a later sprint promotes a plugin with real runtime proof

Verification:
- surviving compliance path is named explicitly in code and docs
- removed or retained SDK helpers match the documented contract posture
- advisory observability claims are either code-cited or moved into explicit gaps
- derived instruction docs no longer contradict requirements or architecture
- release docs stop implying shipped plugin behavior where only scaffold code exists

Acceptance criteria:
- no duplicated source-of-truth surface remains for compliance behavior
- SDK posture is explicit instead of implied
- any remaining advisory-only observability claims are either verified or downgraded to documented gaps
- onboarding and agent instructions do not contradict the release docs
- docs and gaps describe one honest baseline for later sprint work

Definition of done:
- later sprints can build on one intended implementation path per behavior
- code scheduled for retirement is removed early or explicitly deferred
- no public-looking surface remains ambiguous about whether it is real contract or convenience only

QA checklist answers:
- Which requirement IDs or gap IDs changed status?
  Sprint 1 materially reduced `GAP-001`, `GAP-002`, and `GAP-003`, but none closed; `CLI-007`, `TST-007`, and `TMO-004` remain open and are carried forward with more precise post-Sprint-1 wording.
- What code was removed early rather than left in parallel?
  The duplicate compliance behavior in `sc-hooks-cli/src/testing.rs` was reduced to a presentation wrapper over `sc-hooks-test`, the duplicate absent-payload pseudo-check was retired from `sc-hooks-test/src/compliance.rs`, and the stale `LongRunning`/`AsyncContextSource` SDK traits were removed.
- Which files/crates were the owned write scope for the sprint?
  `sc-hooks-test/src/compliance.rs`, `sc-hooks-cli/src/testing.rs`, `sc-hooks-sdk/src/traits.rs`, `README.md`, `CLAUDE.md`, plugin `Cargo.toml` metadata, and the Sprint 1 traceability/gap-plan docs.
- What validation commands and direct tests proved the new contract?
  `cargo test --workspace`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo fmt --check --all` passed on the Sprint 1 branch; the direct proof added in this sprint is limited to the surviving shared compliance engine and its CLI delegation path.
- What follow-on work is blocked or unblocked by this sprint?
  Sprint 2 and Sprint 3 are unblocked because Sprint 1 removed the false duplicate compliance/source-of-truth surfaces; real contract-proof expansion and end-to-end `long_running` alignment still belong to those later sprints.

### Sprint 2: Compliance Harness Hardening (In Review)

Status:
- in review

Focus:
- make the compliance harness prove the release contract directly
- remove duplicated compliance logic before more behavior is layered on top

Write scope:
- `sc-hooks-test/src/compliance.rs`
- `sc-hooks-cli/src/testing.rs`
- dispatch/runtime contract tests and related traceability rows

Early retire or replace:
- duplicated compliance assertions split across CLI and test harness
- any indirect checks that only imply contract behavior instead of asserting it directly

Deliverables:
- expand `sc-hooks-test` to assert timeout behavior, invalid JSON, multiple JSON objects, async misuse, matcher validity, and absent-payload behavior
- refactor `sc-hooks-cli test` to delegate to one shared compliance engine
- update `docs/traceability.md` so `CLI-007` and `TST-007` point to direct assertions instead of partial checks

Verification:
- direct compliance fixtures cover the release-facing protocol branches
- `sc-hooks-cli test` and `sc-hooks-test` exercise the same underlying checks
- no duplicated behavior logic remains without an explicit wrapper-only justification

Acceptance criteria:
- `GAP-001` is closed
- `CLI-007` and `TST-007` move from gap to implemented
- duplicated compliance logic in `sc-hooks-cli/src/testing.rs` is retired or reduced to a thin wrapper

Definition of done:
- one compliance engine owns the contract checks
- stale duplicated compliance code is removed or reduced to glue only
- docs and traceability align to the surviving path
- validation and regression tests pass on the final sprint branch

QA checklist answers:
- Which requirement IDs or gap IDs changed status?
  Sprint 2 closes `GAP-001` and moves `CLI-007` and `TST-007` to implemented. `TMO-004` remains open for Sprint 3.
- What code was removed early rather than left in parallel?
  No duplicate compliance engine was reintroduced; Sprint 2 kept `sc-hooks-cli/src/testing.rs` as presentation-only glue and added the host-path contract suite to the shared `sc-hooks-test` surface instead of splitting behavior back into the CLI.
- Which files/crates were the owned write scope for the sprint?
  `sc-hooks-test/src/compliance.rs`, `sc-hooks-test/src/fixtures.rs`, `sc-hooks-cli/tests/compliance_host.rs`, and the Sprint 2 traceability/gap-plan docs.
- What validation commands and direct tests proved the new contract?
  `cargo test -p sc-hooks-test`, `cargo test -p sc-hooks-cli --test compliance_host`, and the final workspace validation prove direct host-path assertions for timeout, invalid stdout, multi-object warnings, async misuse, matcher filtering, and absent-payload behavior.
- What follow-on work is blocked or unblocked by this sprint?
  Sprint 4 is now unblocked on a real surviving compliance/runtime path, while Sprint 3 remains the next contract-alignment step for `long_running`.
Compatibility note:
- Sprint 2 and Sprint 4 both touch `sc-hooks-cli/tests/`, so later sprint work in that directory must merge-forward the latest Sprint 2 QA fixes before push to avoid regressing the shared host-path tests.

### Sprint 3: `long_running` And SDK Posture Alignment (In Review)

Status:
- in review

Focus:
- define one release-grade `long_running` and SDK posture across host, docs, and tests

Write scope:
- `sc-hooks-sdk/src/traits.rs`
- `sc-hooks-sdk/src/runner.rs`
- timeout/dispatch logic
- manifest validation, audit rules, and release docs

Early retire or replace:
- stale SDK traits that look public but are not part of the real host contract
- SDK runner behavior that could be mistaken for host-guaranteed runtime semantics
- split timeout behavior that differs between docs, host behavior, and SDK assumptions

Deliverables:
- decide whether `sc-hooks-sdk::traits::LongRunning` and `AsyncContextSource` are real release surfaces or stale helpers to retire
- align manifest validation, timeout behavior, audit checks, SDK helper limits, and docs around one contract
- add end-to-end tests that prove the chosen behavior

Verification:
- timeout behavior matches the chosen `long_running` contract in host, docs, and tests
- no public-looking SDK helper remains undocumented or behaviorally unproven

Acceptance criteria:
- `GAP-002` is closed
- `TMO-004` moves from required-before-release to implemented
- requirements, architecture, traceability, and SDK surface all describe the same `long_running` and SDK posture

Definition of done:
- one release-grade `long_running` contract exists
- retired SDK surfaces are removed early, not left as dead public-looking code
- contract behavior is proven by end-to-end tests and reflected in docs

QA checklist answers:
- Which requirement IDs or gap IDs changed status?
  Sprint 3 closes `GAP-002` and moves `TMO-004` from required-before-release to implemented. `CLI-007` cleanup from Sprint 2 is also reflected in the requirements baseline so the docs no longer lag the code.
- What code was removed early rather than left in parallel?
  The redundant audit-only async `long_running` rule was removed in favor of one manifest-validation path, and timeout/handler rendering now use the same sync-only `long_running` rule instead of keeping a split interpretation alive.
- Which files/crates were the owned write scope for the sprint?
  `sc-hooks-sdk/src/manifest.rs`, `sc-hooks-cli/src/timeout.rs`, `sc-hooks-cli/src/handlers.rs`, `sc-hooks-cli/src/audit.rs`, `sc-hooks-cli/tests/long_running_contract.rs`, and the Sprint 3 contract docs.
- What validation commands and direct tests proved the new contract?
  `cargo test -p sc-hooks-sdk manifest::tests::rejects_async_long_running_manifest`, `cargo test -p sc-hooks-cli --test long_running_contract`, and the final workspace validation prove sync no-timeout behavior, async rejection, and the aligned manifest/audit/runtime posture.
- What follow-on work is blocked or unblocked by this sprint?
  Sprint 4 and later release cleanup now inherit one explicit `long_running` contract instead of a split host/audit/SDK interpretation. Richer SDK ergonomics remain deferred and do not block the remaining sprints.

### Sprint 4: Runtime Layout And Setup Proof (In Review)

Status:
- in review

Focus:
- prove the expected `.sc-hooks/` runtime layout from a clean contributor starting point

Write scope:
- runtime layout docs
- setup guidance
- any checked example `.sc-hooks/` tree or install-proof fixture

Early retire or replace:
- ambiguous setup instructions that require reading source to succeed
- duplicate or partial installation guidance across README and docs

Deliverables:
- add either a checked-in example `.sc-hooks/` tree or a setup guide that proves the same layout
- document how `.sc-hooks/config.toml`, `.sc-hooks/plugins/`, and install output fit together
- verify that contributor docs match the actual CLI behavior

Verification:
- a clean setup path succeeds by following the documented steps only
- the example tree or setup guide matches actual CLI/runtime expectations

Acceptance criteria:
- `GAP-004` is closed
- `CFG-001`, `RES-002`, and `CLI-004` no longer depend on a gap note for practical setup clarity
- a contributor can follow the checked-in example or guide without reading source code

Definition of done:
- one canonical setup path exists
- stale or contradictory setup instructions are removed
- docs, examples, and runtime layout all agree

QA checklist answers:
- Which requirement IDs or gap IDs changed status?
  Sprint 4 closes `GAP-004` and removes the practical setup-gap dependency from `RES-002` and `CLI-004`.
- What code was removed early rather than left in parallel?
  No runtime code path was duplicated for setup proof; Sprint 4 replaced inference-only setup guidance with one checked example tree and one host-level validation path.
- Which files/crates were the owned write scope for the sprint?
  `examples/runtime-layout/.sc-hooks/`, `examples/runtime-layout/README.md`, `sc-hooks-cli/tests/runtime_layout_example.rs`, `sc-hooks-cli/tests/`, and the runtime-layout docs/traceability/gap-plan files including `docs/traceability.md`.
- What validation commands and direct tests proved the new contract?
  `cargo test -p sc-hooks-cli --test runtime_layout_example` proves that the checked example audits and runs successfully using the real CLI from the example directory; the final workspace validation keeps that example in the normal release gate.
- What follow-on work is blocked or unblocked by this sprint?
  Sprint 5 is unblocked with one canonical runtime layout frozen in-repo, so plugin packaging and maturity claims can now be evaluated against a concrete install/runtime path instead of inferred setup.

### Sprint 5: Plugin Packaging And Release Honesty

Status:
- in review

Focus:
- keep plugin claims honest unless runtime installation, behavior, and tests exist

Write scope:
- `plugins/`
- release/docs inventory for plugin maturity
- packaging or install-proof checks tied to promoted plugins

Early retire or replace:
- release-facing language that implies a plugin ships when it is still scaffold/reference code
- plugin inventory claims that are not backed by install/runtime proof

Deliverables:
- choose the release posture for each source crate under `plugins/`
- if a plugin is promoted as shipped behavior, add runtime installation guidance and direct behavior tests
- otherwise keep the crate clearly documented as scaffold/reference code

Verification:
- each plugin named as shipped behavior has install/runtime proof
- non-shipping plugins are explicitly scoped as scaffold/reference code in docs

Acceptance criteria:
- `GAP-003` is closed
- `BND-002` is either satisfied for promoted plugins or avoided by keeping release claims scoped to scaffold/reference status only
- README and docs agree on the exact plugin inventory and maturity level

Definition of done:
- plugin release posture is binary for every crate: shipped or scaffold/reference
- no ambiguous maturity claims remain in docs
- packaging and runtime behavior are verified for anything promoted

QA checklist answers:
- Which requirement IDs or gap IDs changed status?
  Sprint 5 closes `GAP-003` and moves `BND-002` to implemented by freezing every current `plugins/` crate as scaffold/reference only.
- What code was removed early rather than left in parallel?
  No runtime plugin behavior was promoted without proof; the sprint removed the remaining ambiguous shipped-plugin posture instead of leaving mixed release claims in parallel.
- Which files/crates were the owned write scope for the sprint?
  `plugins/*/Cargo.toml`, `README.md`, `docs/architecture.md`, `docs/requirements.md`, `docs/implementation-gaps.md`, `docs/traceability.md`, and the Sprint 5 planning section.
- What validation commands and direct tests proved the new contract?
  Sprint 5 closes a release-honesty gap rather than adding shipped plugin behavior. Validation relies on source inspection plus the existing runtime-layout and workspace test gates to confirm the runtime still resolves only `.sc-hooks/plugins/`.
- What follow-on work is blocked or unblocked by this sprint?
  Sprint 6 is unblocked because plugin maturity claims are now binary and consistent across docs and metadata; any future plugin promotion will require a new scoped sprint with install guidance and direct behavior tests.

### Sprint 6: Merge Closeout And Release Gate

Status:
- in review

Focus:
- freeze the release-doc set and record the final reviewer/QA handoff against the chosen scope

Write scope:
- release docs
- PR/review notes
- final branch cleanup only

Early retire or replace:
- stale review-only notes that no longer describe open work
- merge-time TODOs carried forward after the underlying requirement is already resolved

Deliverables:
- confirm no remaining merge-time review note maps to real open work
- remove stale review placeholders that no longer correspond to current requirement or gap IDs
- final doc/code consistency check
- final reviewer and QA handoff

Verification:
- every remaining review note maps to an actually open item or is removed
- final QA targets one frozen branch head with recorded validation commands

Acceptance criteria:
- merge-time review items are closed
- no open blocker gaps remain for the chosen release scope
- branch handoff records exact validation commands and reviewer status

Definition of done:
- branch head is frozen for QA/review
- there are no stale review placeholders left in the plan or release docs
- release docs, code, and traceability describe the same final scope

QA checklist answers:
- Which requirement IDs or gap IDs changed status?
  Sprint 6 does not change release-facing behavior IDs. It retires stale merge-residue framing in the plan and freezes the final release handoff on a scope with no remaining blocker or important gaps.
- What code was removed early rather than left in parallel?
  No code path changed in Sprint 6. The retirement here is review/process residue: the specific stale text `Current open release-relevant drivers are: merge-time review residue tracked under task #370` and the Sprint 6 driver `task #370, final QA/PR review` were removed instead of being carried into release QA.
- Which files/crates were the owned write scope for the sprint?
  `docs/project-plan.md` and any release-facing handoff notes required to record the frozen validation state.
- What validation commands and direct tests proved the new contract?
  Frozen branch head `cdce7b1` recorded `cargo test --workspace`; this fix pass keeps that exact frozen validation record explicit in the release handoff and preflight sections.
- What follow-on work is blocked or unblocked by this sprint?
  Final QA/reviewer handoff is unblocked because the release docs now describe one closed scope with no remaining blocker or important gaps outside deferred items.

Sprint 6 signoff record:
- frozen branch head under review: `cdce7b1`
- reviewer handoff owner: `arch-hook`
- validation commands recorded on frozen head: `cargo test --workspace`
- QA result on frozen head: `SC-QA-S6-1` found blockers/important findings, opened `SC-DEV-S6-FIX-1`, and did not clear the branch for final handoff until those doc fixes landed
- task `#370` disposition: retired as merge-review residue only; it is not a standing requirement or gap ID after the `cdce7b1` freeze

### Sprint 8: Rust Best-Practices Closeout

Status:
- in review

Focus:
- close the remaining post-release Rust best-practices findings without reopening the release contract

Write scope:
- `sc-hooks-sdk/src/conditions.rs`
- `sc-hooks-sdk/src/manifest.rs`
- `sc-hooks-cli/src/audit.rs`
- `docs/requirements.md`
- `docs/traceability.md`
- `docs/observability-contract.md`
- `docs/project-plan.md`
- `docs/implementation-gaps.md`

Deliverables:
- delete the dead `unreachable!()` branch in `sc-hooks-sdk/src/conditions.rs`
- document `AUD-005` and `AUD-009` as implemented audit requirements with matching traceability rows
- document the dispatch stderr fallback when observability emission fails
- record Sprint 8 closeout in the release plan and implementation-gap notes
- add direct tests for the `long_running` audit rejection paths that Sprint 8 promotes into the release docs

Verification:
- `cargo fmt --check --all`
- `cargo test --workspace`
- `cargo clippy --all-targets --all-features -- -D warnings`

Acceptance criteria:
- `SCHOOK-QA-001` stays closed with the dead condition-validation branch removed
- `AUD-005` and `AUD-009` are present in both `docs/requirements.md` and `docs/traceability.md`
- `docs/observability-contract.md` documents the current stderr fallback behavior instead of leaving it implicit in code
- the Sprint 8 plan section reflects the actual write scope, verification, and closure records

Definition of done:
- the remaining Rust best-practices follow-up is represented as a closed, documented sprint rather than an orphaned table row
- code and docs agree on the `long_running` audit failures and observability fallback behavior
- the Sprint 8 branch is ready for the final QA pass on documented closure state

QA checklist answers:
- Which requirement IDs or gap IDs changed status?
  Sprint 8 promotes `AUD-005` and `AUD-009` into the implemented requirement and traceability set; no deferred gap is newly opened.
- What code was removed early rather than left in parallel?
  The dead `if let ConditionOperator::OneOf = condition.op { unreachable!(); }` branch in `sc-hooks-sdk/src/conditions.rs` was removed instead of being left as unreachable residue.
- Which files/crates were the owned write scope for the sprint?
  `sc-hooks-sdk/src/conditions.rs`, `sc-hooks-sdk/src/manifest.rs`, `sc-hooks-cli/src/audit.rs`, and the Sprint 8 release-doc set listed above.
- What validation commands and direct tests proved the new contract?
  `cargo fmt --check --all`, `cargo test --workspace`, and `cargo clippy --all-targets --all-features -- -D warnings` passed; direct tests include `audit_rejects_async_long_running_manifest`, `audit_rejects_long_running_without_description`, `rejects_async_long_running_manifest`, and `rejects_long_running_manifest_without_description`.
- What follow-on work is blocked or unblocked by this sprint?
  Sprint 8 does not introduce new follow-on implementation work; it closes remaining best-practices review residue so the branch can clear final QA.

## 11. Sprint QA Checklist

Each sprint closeout must answer these questions explicitly:

- Which requirement IDs or gap IDs changed status?
- What code was removed early rather than left in parallel?
- Which files/crates were the owned write scope for the sprint?
- What validation commands and direct tests proved the new contract?
- What follow-on work is blocked or unblocked by this sprint?

## 12. Release Preflight

Before release cut or final QA handoff, complete these checks explicitly:

- claim audit: every strong release-facing statement in requirements and contract docs points to code, tests, or a gap/deferred ID
- removal audit: no stale duplicate implementation path remains for any release-facing behavior
- advisory audit: every non-blocking QA advisory is either verified with a cited code path or converted into an explicit gap note
- misalignment audit: no high-risk doc/code/API misalignment class remains outside this plan or `docs/implementation-gaps.md`
- release-doc audit: requirements, architecture, traceability, project plan, and contract docs describe the same final scope
- branch freeze: branch head is frozen before QA/reviewer handoff
- validation record: exact validation commands are recorded on the frozen branch state

Release preflight evidence:

| Check | Status | Evidence |
| --- | --- | --- |
| claim audit | complete | `docs/traceability.md` now includes the previously missing implemented rows `RES-003` and `OBS-005`, so the release-facing claims in `docs/requirements.md` no longer out-run the code/test map. |
| removal audit | complete | The surviving single-path decisions remain recorded in this plan and `docs/implementation-gaps.md`: shared compliance engine (`GAP-001`), sync-only `long_running` posture (`GAP-002`), scaffold-only plugin posture (`GAP-003`), and removed ad hoc logging/builtin handler paths under Sprint 0. |
| advisory audit | complete | Sprint 6 QA findings are explicitly resolved in this fix pass: missing `RES-003`/`OBS-005` traceability rows, missing signoff artifact, missing preflight evidence, and missing task `#370` retirement disposition. |
| misalignment audit | complete | Section 9 still covers every known high-risk misalignment class, and Section 2 continues to report no open release-relevant drivers for the chosen scope outside deferred items. |
| release-doc audit | complete | `docs/requirements.md`, `docs/architecture.md`, `docs/traceability.md`, this plan, `docs/protocol-contract.md`, `docs/observability-contract.md`, and `docs/logging-contract.md` all describe the same plugin-only runtime, `sc-observability` boundary, and scaffold-only `plugins/` posture. |
| branch freeze | complete | Sprint 6 froze branch head `cdce7b1` for reviewer/QA handoff before `SC-QA-S6-1`; this record keeps that frozen-head reference durable instead of implicit in ATM only. |
| validation record | complete | The frozen-head validation command is recorded as `cargo test --workspace` in both the Sprint 6 QA checklist answers and the Sprint 6 signoff record above. |

## 13. Risk Containment

- Dispatch/protocol changes must land with direct regression tests in the same sprint; no speculative parser or contract changes without tests.
- Any replacement of runtime or observability behavior must delete the superseded path in the same sprint unless a compatibility exception is recorded.
- If a sprint cannot complete removal safely, the retained path must be named in the sprint handoff with an explicit follow-up owner.

## 14. Decision Register

Closed decisions that should not be reopened during implementation:

- observability is a current requirement and is implemented through `sc-observability` at the `sc-hooks-cli` boundary
- the old ad hoc logging path and builtin `log` handler are removed, not retained for compatibility
- the runtime is plugin-only; builtin handler resolution is not part of the current release baseline
- stdout protocol handling is strict for invalid trailing output and warning-only only for additional valid JSON objects after the first result

## 15. Out Of Scope For This Plan

These items stay deferred unless product direction changes:
- richer `fire` output beyond the current summary string
- finer-grained resolution-time exit codes
- SDK ergonomics beyond the current host-enforced contract
- production-ready bundled plugin behavior beyond the scaffold/reference posture frozen in Sprint 5

## 16. Release Gate

The release plan is complete only when:
- all non-deferred blocker or important gaps are either closed or explicitly removed from release scope
- every required-before-release item is either implemented or intentionally cut from the release contract
- traceability rows for release claims point to real code and tests, not inference alone
- no uncovered high-risk misalignment class remains outside this plan or `docs/implementation-gaps.md`
- merge-time review items are closed
- branch head is frozen before QA/reviewer handoff
- exact validation commands are recorded on that frozen branch state
- reviewer and QA signoff are recorded on the final branch state

## 17. Post-Release Hook Extension Track

This track begins only after the current release plan is accepted.

Purpose:
- extend `sc-hooks` toward the Claude ATM hook set without guessing hook schemas
- keep provider-specific evidence and ATM-specific behavior separate
- make the first implementation pass small, exact, and test-driven

### Hook Phase 0: Review Baseline

Status:
- in review

Focus:
- freeze the hook planning baseline in docs before any hook runtime code is written

Deliverables:
- freeze `docs/archive/plugin-plan-s9.md` as the umbrella Sprint 9 execution plan
- `docs/hook-api/claude-hook-api.md`
- `docs/hook-api/atm-hook-extension.md`
- `docs/hook-api/codex-hook-api.md`
- `docs/hook-api/cursor-agent-hook-api.md`
- core-doc additions in `docs/requirements.md` and `docs/architecture.md`

Acceptance criteria:
- QA can review the Sprint 9 sequence from one umbrella document instead of
  reconstructing it from multiple planning fragments
- the Claude implementation baseline is explicit
- ATM-specific behavior is isolated in its own document
- Cursor remains documented but deferred from the first implementation pass
- no implementation-facing field is promoted without a verified source
- Hook Phase 0 closes only after Sprint 6 is formally accepted and the post-release hook track is allowed to begin

### Hook Phase 1: Claude Schema Harness

Focus:
- build the first hook harness for Claude only and freeze the captured
  provider baseline before writing runtime hook code

Write scope:

- `test-harness/hooks/README.md`
- `test-harness/hooks/scripts/run-capture.sh`
- `test-harness/hooks/claude/{prompts,hooks,models,fixtures,captures,reports,scripts,tests}/`
- fixture manifests and harness runner helpers

Deliverables:
- `test-harness/hooks/README.md` harness contract file
- `test-harness/hooks/` scaffold
- Claude provider adapter
- Claude fixture capture scripts
- Claude validation models
- CI drift check for breaking Claude payload changes
- approved fixture snapshots and a first live Claude Haiku report

Required tests:

- `pytest test-harness/hooks/`
- harness structure and fixture validation tests under
  `test-harness/hooks/claude/tests/`

Acceptance criteria:
- Claude hook payloads for the planned hook set are captured and validated
- raw captured fixtures are stored as review evidence
- CI fails on required-field removal or type drift
- the harness can be rerun from repo docs without reconstructing ad hoc setup

Definition of done:
- the team can point to captured Claude payloads instead of inferred shapes

### Hook Phase 2: Plan Revision From Captured Claude Schema

Focus:
- revise the hook plan from captured evidence before implementation starts

Write scope:

- `docs/archive/plugin-plan-s9.md`
- `docs/hook-api/claude-hook-api.md`
- `docs/hook-api/atm-hook-extension.md`
- `docs/project-plan.md`
- `docs/requirements.md`
- `docs/architecture.md`

Deliverables:
- updated `docs/archive/plugin-plan-s9.md`
- updated `docs/hook-api/claude-hook-api.md`
- any additional traceability/gap notes needed for implementation readiness
- frozen normalized `agent_state` model
- frozen canonical session-state schema
- frozen hook trait/result/context contract

Required tests:

- `pytest test-harness/hooks/`
- `cargo test --workspace`

Acceptance criteria:
- every planned Claude implementation field is backed by captured fixtures or
  existing source-of-truth code/docs/tests
- unknown fields remain explicitly deferred
- implementation tasks can start without schema guessing
- the remaining hook phases define exact code to write, tests required, and
  success criteria

### Hook Phase 3: Claude Session And Lifecycle Implementation

Focus:
- freeze the hook trait and implement the generic lifecycle/state foundation first

Write scope:

- `sc-hooks-core/`
- `sc-hooks-sdk/`
- `plugins/agent-session-foundation/`
- same-PR updates to `docs/architecture.md`, `docs/requirements.md`, and
  `docs/project-plan.md`

Deliverables:
- final hook trait/context/result contract in `sc-hooks-core` / `sc-hooks-sdk`
- `plugins/agent-session-foundation`
- tests proving `SessionStart`, `SessionEnd`, and `PreCompact` against the
  captured contract
- session-state file implementation with normalized `agent_state` transitions
- same-agent correlation across directory changes

Required tests:

- unit tests for normalized `agent_state` transitions
- integration tests for session-state persistence keyed by `session_id`
- integration tests proving `SessionStart` in directory A and later lifecycle
  events in directory B still resolve the same session record
- `cargo test --workspace`
- `cargo clippy --all-targets --all-features -- -D warnings`

Acceptance criteria:
- lifecycle hooks use only verified inputs
- ATM-specific routing stays out of the generic lifecycle crate
- the session-state schema matches the documented canonical record
- the trait boundary no longer relies on raw `serde_json::Value` alone as the
  only plugin-facing abstraction

### Hook Phase 4: Claude Command And Spawn Gates

Focus:
- implement the generic spawn and tool-gate utilities

Write scope:

- `plugins/agent-spawn-gates/`
- `plugins/tool-output-gates/`
- any same-PR doc updates required if the captured schema or blocking contract
  needs clarifying

Deliverables:
- `plugins/agent-spawn-gates`
- `plugins/tool-output-gates`
- direct behavior tests for named-agent vs background-agent policy
- direct behavior tests for fenced-JSON/schema-governed spawn blocking
- schema lookup from inline prompt definitions or same-name sibling schema files
- exact retryable block responses for invalid fenced JSON

Required tests:

- direct tests for `tool_name = "Agent"` spawn-gate routing
- tests for named-agent versus background-agent policy outcomes
- tests for subagent linkage fields written into the canonical session-state file
- tests for fenced `json` extraction and schema validation success/failure
- tests proving invalid input returns exact retryable failure reasons
- `cargo test --workspace`
- `cargo clippy --all-targets --all-features -- -D warnings`

Acceptance criteria:
- no field is relied on unless it was verified in Phase 1 or added in a later
  approved schema capture
- spawn and tool-blocking behavior is tested directly
- block responses explain exactly how the caller can retry successfully
- generic blocking/fenced-JSON policy remains separate from ATM-specific relay
  behavior

### Hook Phase 5: Claude Relay Hooks

Focus:
- implement ATM-specific extension behavior after the generic layer is stable

Write scope:

- `plugins/atm-extension/`
- ATM-only docs where relay semantics or teammate-idle mapping must be frozen

Deliverables:
- `plugins/atm-extension`
- direct tests for ATM Bash identity-file behavior
- direct tests for `PermissionRequest` and `Stop`
- direct tests for teammate-idle mapping onto normalized `idle`
- ATM enrichment on the canonical session-state file through extension fields
- `Notification` stays wired and documented, but remains deferred until a live
  payload is captured

Required tests:

- tests for ATM identity-file create/delete behavior around `atm` Bash commands
- tests for ATM extension fields on the canonical session-state record
- tests for relay mapping on `PermissionRequest`, `Stop`, and teammate-idle
- `cargo test --workspace`
- `cargo clippy --all-targets --all-features -- -D warnings`

Acceptance criteria:
- ATM behavior is layered on top of the generic hook utilities rather than
  defining them
- failure posture is documented and tested
- `Notification` stays wired but does not block completion of this phase until
  a live payload is captured and promoted

### Hook Phase 6: Cross-Provider Follow-On

Focus:
- only after the Claude baseline is stable, decide whether to expand to other
  providers

Write scope:

- provider follow-on planning docs only
- no runtime crate work without separate approval and provider-specific capture

Current deferred items:
- Codex harness and implementation work
- Gemini harness and implementation work
- Cursor harness capture
- Cursor runtime implementation

Required tests:

- docs-only validation plus any provider harness tests explicitly approved for
  that provider follow-on

Acceptance criteria:

- follow-on provider work is represented as schema-backed planning, not guessed
  implementation
- Claude remains the only active runtime baseline until another provider is
  explicitly captured and approved

Entry rule:
- this phase requires separate approval after the Claude ATM baseline is
  captured, revised, and implemented

### S10-VERSION-BUMP-1: Claude Version-Bump Detection

Status:
- completed

Focus:
- detect Claude CLI version bumps before maintainers accept Claude hook
  contract changes without rerunning schema validation

Write scope:

- `scripts/verify-claude-hook-api.py`
- `test-harness/hooks/claude/fixtures/approved/manifest.json`
- `test-harness/hooks/claude/tests/test_version_bump_detector.py`
- `docs/requirements.md`
- `docs/architecture.md`
- `docs/project-plan.md`
- `docs/traceability.md`

Deliverables:
- a Claude-only detector comparing `claude --version` to the approved manifest
- approved manifest stores the current validated `claude_version`
- pytest coverage for match, mismatch, and missing-manifest-version failure modes
- release-doc updates recording the detector as the implementation path for
  `TST-008`

Required tests:

- `python3 scripts/verify-claude-hook-api.py`
- `python3 -m pytest test-harness/hooks/claude/tests/test_version_bump_detector.py`
- `cargo test --workspace`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo fmt --check --all`

Acceptance criteria:
- the approved Claude manifest stores a validated `claude_version`
- the detector exits `0` when the installed Claude version matches the approved
  manifest
- version mismatches exit non-zero with rerun guidance
- missing or invalid manifest version data fails clearly instead of producing a
  traceback

### S11-DOC.1: README And Usage Guide Release-Doc Alignment

Status:
- in review

Focus:
- align the operator-facing top-level docs with the current release baseline
- remove stale CLI/example wording that drifted from the control docs

Write scope:
- `README.md`
- `USAGE.md`
- `docs/project-plan.md`

Deliverables:
- README and usage examples with no `--sync` flag on `fire` invocations
- explicit Unix-like-shell qualifier on install snippets
- README plugin inventory aligned with the architecture baseline that treats
  all current `plugins/` source crates as scaffold/reference only in release docs
- a clear naming note that `docs/requirements.md` uses `sc-hooks` as the
  product command label while the current Cargo binary artifact in this repo is
  `sc-hooks-cli`

Required tests:

- `cargo test --workspace`
- `cargo clippy --all-targets --all-features -- -D warnings`

Acceptance criteria:
- no `fire` example in `README.md` or `USAGE.md` includes `--sync`
- install snippets are explicitly scoped to Unix-like shells
- README plugin table contains scaffold/reference-only language and no
  `runtime-implementation` claims
- `S11-DOC.1` appears in the sprint table and this detail section

### S12-PUB.1: Workspace Publish Prep And Release Infrastructure

Status:
- in review

Focus:
- move publishable workspace crates under `crates/`
- establish manifest-driven release infrastructure
- document the honest first-release scope

Write scope:

- `Cargo.toml`
- `crates/`
- `release/publish-artifacts.toml`
- `.github/workflows/release-preflight.yml`
- `.github/workflows/release.yml`
- `PUBLISHING.md`
- `scripts/release_gate.sh`
- release-facing docs

Deliverables:
- workspace crates moved to `crates/`
- manifest-driven publish inventory for:
  - `sc-hooks-core`
  - `sc-hooks-sdk`
  - `sc-hooks-test` (tracked, not published)
  - `sc-hooks-cli` (tracked for binary releases; crates.io publish remains outside the current manifest wave)
- release workflows for preflight, tagged release, GitHub archives, Homebrew, and WinGet
- release gate script for branch/clean-tree/version checks
- docs that state the current crates.io publish wave covers only the currently publishable working crates, not scaffold/reference plugin crates or the CLI crate still excluded by the current release manifest

Required tests:

- `cargo test --workspace`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `python3 scripts/release_artifacts.py list-publish-plan --manifest release/publish-artifacts.toml`
- `python3 scripts/release_artifacts.py list-release-binaries --manifest release/publish-artifacts.toml`

Acceptance criteria:
- the workspace uses `crates/<name>` paths for the four host crates
- release infrastructure is manifest-driven rather than hardcoded in workflow YAML
- `PUBLISHING.md` documents crates.io, GitHub Releases, Homebrew, and WinGet
- the documented current crates.io release scope excludes scaffold/reference plugin crates and the CLI crate still excluded by the current release manifest
