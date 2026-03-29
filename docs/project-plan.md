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
- none; release-facing blocker and important gaps are closed for the chosen scope

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
- console-sink coverage is the first planned observability expansion because it
  is the most useful next surface for debugging live multi-agent interactions
  and background-agent monitoring
- custom sink coverage and richer multi-hook monitoring remain follow-on work
  after console-sink behavior is frozen

Important planning rule:
- `sc-observability` remains a requirement, but it is already implemented
- therefore observability appears below as a completed baseline sprint, not as pending build work

## 4. Sprint Sequence

| Sprint | Identifier | Status | Focus | Primary drivers | Gate / Depends on | Primary write scope |
| --- | --- | --- | --- | --- | --- | --- |
| Sprint 0 | — | Completed | architecture and observability alignment | `OBS-001`, `OBS-002`, `OBS-006`, `OBS-007`, `OBS-008`, `GAP-005`, `GAP-007` | none | `sc-hooks-cli`, observability docs, release docs |
| Sprint 1 | — | In review | baseline alignment and code retirement | `GAP-001`, `GAP-002`, `GAP-003` | Sprint 0 | `sc-hooks-cli/src/testing.rs`, `sc-hooks-test`, `sc-hooks-sdk`, release docs |
| Sprint 2 | — | In review - fix-r1 pushed | compliance harness hardening | `GAP-001`, `CLI-007`, `TST-007` | Sprint 1 | `sc-hooks-test`, `sc-hooks-cli/src/testing.rs`, dispatch/runtime contract tests |
| Sprint 3 | — | In review | `long_running` contract alignment | `GAP-002`, `TMO-004` | Sprint 1 | `sc-hooks-sdk`, timeout/dispatch flow, requirements/architecture/traceability |
| Sprint 4 | — | In review | runtime layout and setup proof | `GAP-004`, `CFG-001`, `RES-002`, `CLI-004` | Sprint 2 | install/runtime layout docs, example `.sc-hooks/` tree, contributor path |
| Sprint 5 | — | In review | plugin packaging and release honesty | `GAP-003`, `BND-002` | Sprint 4 | `plugins/`, install/release docs, runtime packaging checks |
| Sprint 6 | — | In review | release freeze and final QA handoff | final reviewer/QA handoff | Sprints 2-5 | release docs, PR/review records, final cleanup |
| Sprint 8 | — | In review | Rust best-practices closeout | `AUD-005`, `AUD-009`, `OBS-005`, `SCHOOK-QA-001` | Sprint 6 | `sc-hooks-sdk`, `sc-hooks-cli`, release docs |
| Sprint 9 | S9-P0 | Completed | Phase 0: Review Baseline | `HKR-001`, `HKR-002`, `HKR-003`, `HKR-006`, `HKR-007` | Sprint 6 formally accepted | hook API docs, `docs/plugin-plan-s9.md`, `docs/requirements.md`, `docs/architecture.md` |
| Sprint 9 | S9-P1 | Completed | Phase 1: Build Harness | `HKR-002`, `HKR-005` | S9-P0 | `test-harness/hooks/README.md`, `test-harness/hooks/claude/`, harness models, fixtures, reports |
| Sprint 9 | S9-P2 | Completed | Phase 2: Capture Claude Payloads | `HKR-002`, `HKR-003` | S9-P1 | `test-harness/hooks/claude/captures/raw/`, approved fixtures, capture reports |
| Sprint 9 | S9-P2A | Completed | Phase 2A: Global HTML Reporting Stack | `HKR-005`, `HKR-012` | S9-P2 | `$HOME/.claude/skills/html-report/SKILL.md`, `~/.claude/agents/html-report-generator.md`, validation example |
| Sprint 9 | S9-P3 | Completed | Phase 3: Pydantic Models + Schema | `HKR-003`, `HKR-005` | S9-P2A | Pydantic models, generated schema artifacts, drift validation tests |
| Sprint 9 | S9-P4 | Completed | Phase 4: Revise Plan from Schema | `HKR-003` | S9-P3 | `docs/plugin-plan-s9.md`, `docs/hook-api/claude-hook-api.md`, traceability notes |
| Sprint 9 | S9-P5 | Completed | Phase 5: Re-Evaluate + Sequence | `HKR-003`, `HKR-004` | S9-P4 | implementation disposition notes, runtime sequence updates |
| Sprint 9 | S9-HP3 | Implementation Complete | Hook Phase 3: Session Foundation | `HKR-004` | S9-P5 | `sc-hooks-session-foundation`, same-PR architecture inventory update |
| Sprint 9 | S9-HP4 | Implementation Complete | Hook Phase 4: Bash Identity + Spawn Gates | `HKR-004` | S9-HP3 | `agent-spawn-gates`, `tool-output-gates`, direct behavior tests |
| Sprint 9 | S9-HP5 | Implementation Complete | Hook Phase 5: Relay Hooks | `HKR-004` | S9-P5 | `atm-extension`, relay tests |
| Sprint 9 | S9-PBC | In QA | Plan-BC: BC Design Consolidation | `HKR-003`, `HKR-004` | independent | `docs/phase-bc-hook-runtime-design.md`, core plan docs, hook API alignment |
| Sprint 9 | S9-BONUS | Completed | Console-sink observability coverage (`DEF-008` partial) | `DEF-008` | Sprint 0 | `sc-hooks-cli/tests/`, `docs/observability-contract.md`, `docs/logging-contract.md`, `docs/architecture.md`, `docs/implementation-gaps.md`, `docs/requirements.md`, `docs/traceability.md` |
| Hook Phase 6 | — | Planned | post-Claude follow-on planning only | `HKR-006`, `HKR-007` | S9-HP5 plus separate approval | provider follow-on planning docs only |

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

### Deferred Follow-Up: DEF-008 Console-Sink Coverage

Status:
- partially complete — console-sink delivered; custom sinks + multi-hook correlation deferred

Focus:
- prove console-sink output under real `sc-hooks-cli` dispatch
- keep file-sink contract coverage intact while adding an operator-facing
  debugging surface

Write scope:
- `sc-hooks-cli/tests/`
- `docs/requirements.md`
- `docs/architecture.md`
- `docs/observability-contract.md`
- `docs/logging-contract.md`
- `docs/implementation-gaps.md`
- `docs/traceability.md`

Deliverables:
- real-dispatch tests for console-sink emission on:
  - success
  - block
  - error
  - timeout
- docs that state the intended relationship between:
  - file sink as baseline durable contract
  - console sink as operator/debugging surface
- explicit note on whether console and file sinks are expected to emit the same
  dispatch semantics or intentionally differ in rendered fields

Test plan:
- run the real `sc-hooks-cli` binary in isolated temp dirs
- enable console sink in a controlled test harness
- capture console output and assert:
  - one emitted line per qualifying dispatch
  - correct action/outcome/level rendering for success, block, error, timeout
- keep the existing file-sink contract tests running in the same suite so the
  console work cannot regress the JSONL baseline

Acceptance criteria:
- at least one integration test proves console-sink emission for a successful
  dispatch
- at least one integration test proves console-sink emission for a blocked or
  errored dispatch
- docs name console-sink coverage as the first post-file-sink observability
  expansion
- `DEF-008` remains open only for work beyond console-sink coverage, such as
  custom sinks and multi-hook monitoring correlation

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
  Sprint 5 closes `GAP-003` and moves `BND-002` to implemented by freezing the legacy `plugins/` crates as scaffold/reference only and requiring explicit maturity status for any later implementation crate.
- What code was removed early rather than left in parallel?
  No runtime plugin behavior was promoted without proof; the sprint removed the remaining ambiguous shipped-plugin posture instead of leaving mixed release claims in parallel.
- Which files/crates were the owned write scope for the sprint?
  `plugins/*/Cargo.toml`, `README.md`, `docs/architecture.md`, `docs/requirements.md`, `docs/implementation-gaps.md`, `docs/traceability.md`, and the Sprint 5 planning section.
- What validation commands and direct tests proved the new contract?
  Sprint 5 closes a release-honesty gap rather than adding shipped plugin behavior. Validation relies on source inspection plus the existing runtime-layout and workspace test gates to confirm the runtime still resolves only `.sc-hooks/plugins/`, even when a source implementation crate such as `plugins/agent-session-foundation` exists in the workspace.
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
- extend `schook` toward the Claude ATM hook set without guessing hook schemas
- keep provider-specific evidence and ATM-specific behavior separate
- make the first implementation pass small, exact, and test-driven

### S9-P0: Phase 0: Review Baseline

Status:
- completed

Focus:
- freeze the hook planning baseline in docs before any hook runtime code is written

Deliverables:
- freeze `docs/plugin-plan-s9.md` as the umbrella Sprint 9 execution plan
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
- S9-P0 closes only after Sprint 6 is formally accepted and the post-release hook track is allowed to begin

### S9-P1: Phase 1: Build Harness

Status:
- completed

Focus:
- build the first hook harness for Claude only

Deliverables:
- `test-harness/hooks/README.md` harness contract file
- `test-harness/hooks/` scaffold
- Claude provider adapter
- Claude fixture capture scripts
- Claude validation models
- approved fixture snapshots and harness reports
- rerunnable harness entry points documented in repo docs
- `Notification(idle_prompt)` kept wired in the harness as a bounded probe, not
  promoted as a verified payload shape

Acceptance criteria:
- Claude hook payloads for the planned hook set are captured and validated
- raw captured fixtures are stored as review evidence
- the harness can be rerun from repo docs without reconstructing ad hoc setup
- the harness baseline distinguishes verified captures from wired-but-unresolved
  probes

Definition of done:
- the team can point to captured Claude payloads instead of inferred shapes

### S9-P2: Phase 2: Capture Claude Payloads

Status:
- completed

Focus:
- collect the actual Claude hook payloads that later schema/model and implementation work must obey

Deliverables:
- captured Claude payloads stored under `test-harness/hooks/claude/captures/raw/`
- approved fixtures and follow-up capture notes in the harness tree
- evidence for `startup`, `resume`, `clear`, `compact`, `PreCompact`, `PreToolUse(Bash)`, `PreToolUse(Agent)`, `PostToolUse(Bash)`, `PermissionRequest`, `Stop`, and `SessionEnd`
- explicit note that `Notification(idle_prompt)` remains wired-but-unresolved in
  this environment after bounded probing

Acceptance criteria:
- the required first-pass Claude capture set is backed by local fixtures
- capture artifacts are committed in the harness tree
- implementation-facing docs cite captured evidence instead of inferred payload shapes

Note:
- S9-P2 completed as part of the executed S9-P1 harness follow-on; captured artifacts now live under `test-harness/hooks/claude/captures/raw/`

### S9-P2A: Phase 2A: Global HTML Reporting Stack

Status:
- not started

Focus:
- build the reusable global HTML reporting stack before any schema-drift sprint
  depends on report generation

Deliverables:
- `$HOME/.claude/skills/html-report/SKILL.md`
- `~/.claude/agents/html-report-generator.md`
- both files reviewed against:
  `/Users/randlee/Documents/github/synaptic-canvas/docs/claude-code-skills-agents-guidelines-0.4.md`
- one tested invocation producing a valid self-contained HTML file

Acceptance criteria:
- the discovery and execution layers both exist at the required paths
- both pass the guidelines review gate
- the execution layer can be invoked as a background agent from a caller skill
- a test invocation proves self-contained HTML output

### S9-P3: Phase 3: Pydantic Models + Schema

Status:
- not started

Focus:
- create provider-specific Pydantic models and generated schema artifacts from the captured Claude fixtures

Deliverables:
- provider-specific Pydantic models for the captured Claude payloads
- `pyproject.toml` with `pydantic>=2.0` and `pytest`
- generated schema artifacts derived from those models
- drift validation tests for the captured fixtures
- `test-harness/hooks/run-schema-drift.py`
- per-provider adapters under `test-harness/hooks/<provider>/`
- `.claude/skills/hook-schema-drift/`
- report-generation flow that uses the completed global HTML reporting stack

Acceptance criteria:
- captured Claude fixtures validate against the Pydantic models
- schema artifacts exist for the validated models
- drift classification rules execute in tests
- schema drift remains a manual tool path, not CI
- any report-generating workflow depends on completed `S9-P2A`

### S9-P4: Phase 4: Revise Plan from Schema

Status:
- not started

Focus:
- revise the hook plan from validated schema before implementation starts

Deliverables:
- updated `docs/plugin-plan-s9.md`
- updated `docs/hook-api/claude-hook-api.md`
- traceability and implementation-readiness notes derived from validated schema
- explicit classification of each implementation dependency as:
  - source-backed
  - fixture-backed
  - deferred
- cross-reference back to `docs/phase-bc-hook-runtime-design.md` for the frozen
  runtime design boundary

Acceptance criteria:
- every planned Claude implementation field is backed by validated schema or retained as an explicit deferral
- implementation-facing docs no longer depend on pre-schema guesses
- implementation-facing docs and the BC design doc agree on what code lands next

### S9-P5: Phase 5: Re-Evaluate + Sequence

Status:
- not started

Focus:
- decide the implementation queue and prototype disposition only after the schema phases finish

Deliverables:
- explicit disposition for each prototype crate
- runtime implementation sequence derived from validated schema

Acceptance criteria:
- implementation tasks are ready to start without schema guessing
- prototype branches remain reference-only until this sequencing pass is complete

### S9-PBC: Plan-BC: BC Design Consolidation

Status:
- in QA

Focus:
- consolidate the clean post-capture runtime design in this repo before implementation starts

Deliverables:
- `docs/phase-bc-hook-runtime-design.md`
- core-doc alignment for session-state ownership, `agent_state`, logging, trait boundaries, and crate split

Acceptance criteria:
- the clean runtime design authority lives in `schook`
- implementation phases below can reference one runtime design source of truth

### S9-HP3: Hook Phase 3: Session Foundation

Status:
- implemented out of sequence
- planned HP3 test inventory is complete on the current branch

Focus:
- implement the Claude lifecycle pair first

Deliverables:
- `plugins/agent-session-foundation`
- tests proving `SessionStart`, `SessionEnd`, and `Stop` state-tracking
  behavior against the captured contract
- canonical session-state file keyed by:
  - `session_id`
  - `active_pid`
  - `ai_root_dir`
- `ai_root_dir` chaining from `CLAUDE_PROJECT_DIR`
- `ai_current_dir` chaining from payload `cwd`
- `PreCompact` and `Stop` handling on the normalized `agent_state` path
- `PreToolUse`, `PostToolUse`, and `PermissionRequest` explicitly deferred to
  `S9-HP4`
- atomic write semantics for `session.json`
- no `session.json` rewrite when the canonical record is unchanged
- mandatory hook logging for every lifecycle invocation

Acceptance criteria:
- lifecycle hooks use only verified inputs
- ATM-specific routing/persistence stays bounded by the ATM extension doc
- the canonical session-state schema matches the documented BC design
- state identity and persistence are stable across directory changes and PID-bound
  hook calls

### S9-HP4: Hook Phase 4: Bash Identity + Spawn Gates

Status:
- implemented out of sequence
- planned HP4 test inventory is complete on the current branch

Focus:
- implement the spawn gate and structured tool-output gate from the clean BC design

Deliverables:
- `agent-spawn-gates`
- `tool-output-gates`
- direct behavior tests for command-sensitive and team-policy behavior
- schema lookup from inline prompt definitions or same-name sibling schema files
- named-agent versus background-agent policy table
- exact retryable block responses for invalid fenced JSON
- fenced `json` extraction and validation tests

Acceptance criteria:
- no field is relied on unless it was verified in Phase 1 or added in a later
  approved schema capture
- command-sensitive behavior is tested directly
- invalid fenced JSON returns exact retryable failure reasons
- spawn policy outcomes are tested for named-agent and background-agent paths

### S9-HP5: Hook Phase 5: Relay Hooks

Status:
- implemented out of sequence
- planned HP5 test inventory is complete on the current branch

Focus:
- implement the notification/permission/stop relays

Deliverables:
- `atm-extension`
- direct tests for `PermissionRequest` and `Stop`
- direct tests for teammate-idle mapping onto normalized `idle`
- ATM enrichment layered onto the canonical session-state record via extension
  fields
- ATM environment inheritance rules for `ATM_TEAM` and `ATM_IDENTITY`
- `Notification(idle_prompt)` kept wired and documented, but still deferred
  unless a live payload is captured

Acceptance criteria:
- relay behavior is bounded to the verified Claude ATM baseline
- failure posture is documented and tested
- ATM extends the canonical record; it does not redefine the generic state model
- parent/child ATM identity behavior is documented and tested where the child
  environment overrides `ATM_IDENTITY`

### Hook Phase 6: Cross-Provider Follow-On

Focus:
- only after the Claude baseline is stable, decide whether to expand to other
  providers

Current deferred items:
- Codex harness and implementation work
- Gemini harness and implementation work
- Cursor harness capture
- Cursor runtime implementation

Entry rule:
- this phase requires separate approval after the Claude ATM baseline is
  captured, revised, and implemented
