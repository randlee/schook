# Phase N Remediation Plan

## Goal

Define the post-`Phase N` remediation sprint sequence required to clear the
phase-ending QA findings on `integrate/phase-N` before the final
`integrate/phase-N -> develop` merge path resumes.

This plan covers:

- 5 blockers from the phase-ending review
- 10 important findings from the same review
- 1 additional net-new important finding from the follow-up production-readiness
  review on the newer `integrate/phase-N` head

One production-readiness finding overlaps an existing QA item and is therefore
merged into the same remediation sprint instead of being tracked twice.

## Baseline

- planning branch: `feature/phase-N-fix-planning`
- target integration branch for remediation execution: `integrate/phase-N`
- current reviewed integration baseline: `df2794f`
  - `docs: finalize Phase N readiness verdict (PARTIAL_GO)`

Authoritative finding sources:

- `quality-mgr` phase-ending QA findings on `integrate/phase-N`
- `team-lead` task `SC-PN-FIX-PLAN-1`
- direct production-readiness review against `integrate/phase-N`

## Remediation Inventory

### Blockers

| ID | Summary | Planned sprint |
| --- | --- | --- |
| `BP-HARNESS-NEW-001` | add fixture-validation test modules in `codex/tests` and `gemini/tests` | `NF2` |
| `BP-HARNESS-NEW-002` | add `test-harness/hooks/tests/__init__.py` | `NF1` |
| `BP-HARNESS-NEW-003` | replace `$REPO_ROOT` with `/synthetic/test/codex-harness/` in approved Codex fixtures | `NF1` |
| `BP-HARNESS-NEW-004` | replace real GitHub owner in Claude approved fixture | `NF1` |
| `GEMINI-MODULE-IMPORT` | fix `ModuleNotFoundError` in Gemini `session_start.py` | `NF2` |

### Important

| ID | Summary | Planned sprint |
| --- | --- | --- |
| `SCHOOK-QA-PN-001` | `sprint-N1.md` and `sprint-N2.md` still say `status: planned` | `NF1` |
| `SCHOOK-QA-PN-002` | `project-plan.md` Phase N status/closure text inconsistent | `NF3` |
| `BP-CORE-005` | `UnsupportedOperator` should replace `InvalidValue` catch-all path | `NF4` |
| `BP-CLI-001` | `CliError::Internal` needs recovery guidance | `NF5` |
| `BP-CLI-002` | `OnceLock` statics need concurrency-invariant comments | `NF1` |
| `BP-CLI-003` | timeout invariant relies on bare `expect(...)` | `NF5` |
| `BP-HARNESS-NEW-005` | `PendingRecord.from_json` needs structured `KeyError` handling | `NF6` |
| `BP-HARNESS-NEW-006` | Codex/Gemini hook entry points need idempotency documentation | `NF1` |
| `BP-HARNESS-NEW-007` | Gemini approved fixture contains real PGID | `NF1` |
| `BP-HARNESS-NEW-008` | Claude approved-fixture promotion path needs cwd/PWD sanitization rule or enforcement | `NF2` |

### Additional Production-Readiness Finding

| ID | Summary | Planned sprint |
| --- | --- | --- |
| `PN-PRR-001` | `docs/phase-N/release-checklist.md` still says the promotion verdict is `PENDING` after `df2794f` finalized readiness | `NF3` |

Merged overlap:

- `PN-PRR-002` from the production-readiness review is merged into
  `SCHOOK-QA-PN-002`
  - both describe the same cross-document `Phase N` status/closure mismatch
    between `readiness.md`, `release-checklist.md`, and `project-plan.md`

### Minor Carry-Forward Batch

These are intentionally batched into `NF1` unless a later implementation pass
proves one is structural:

- `BP-CORE-006`
- `BP-CLI-004`
- `BP-CLI-005`
- `BP-TEST-001`
- `SCHOOK-QA-PN-003`
- `BP-HARNESS-NEW-010`
- `BP-HARNESS-NEW-011`

## Sequencing Rules

1. Land `NF1` first.
2. Land `NF2` second.
3. `NF3` may begin only after `NF1` is merged, because the closure/status
   cleanup establishes the doc baseline for the final verdict reconciliation.
4. `NF4` and `NF5` may run in parallel after `NF1` if resourcing requires it,
   but both still merge back through `integrate/phase-N`.
5. `NF6` waits for `NF1` because it touches the same Codex harness surface and
   should not race with the harness/doc hygiene batch.

Default merge-forward order:

- `NF1`
- `NF2`
- `NF3`
- `NF4`
- `NF5`
- `NF6`

## Sprint Sequence

### `NF1` Simple Harness And Doc Hygiene Batch

Purpose:

- clear the simple blocker set
- clear the simple fixture sanitization and hook-entry documentation items
- close the minor wording/cosmetic items in the same pass

Planned branch:

- `feature/pN-fix-s1-harness-hygiene-batch`

### `NF2` Harness Validation And Gemini Import Gate

Purpose:

- restore real Codex/Gemini fixture-validation proof
- fix the Gemini module import blocker
- close the Claude capture-path sanitization requirement while the harness
  proof path is open

Planned branch:

- `feature/pN-fix-s2-harness-validation-gate`

### `NF3` Phase N Closure Record Reconciliation

Purpose:

- reconcile `readiness.md`, `release-checklist.md`, `project-plan.md`, and any
  linked summary docs to the now-finalized `PARTIAL_GO` verdict
- remove stale merge-time `PENDING` wording that survived the final readiness
  commit

Planned branch:

- `feature/pN-fix-s3-phase-n-closure-reconciliation`

### `NF4` Core Condition Error Semantics

Purpose:

- correct the `UnsupportedOperator` error mapping in `sc-hooks-core`
- close the linked small core follow-up in the same file while the module is
  under review

Planned branch:

- `feature/pN-fix-s4-core-condition-semantics`

### `NF5` CLI Error Surface And Timeout Invariant Hardening

Purpose:

- harden `CliError::Internal` recovery guidance
- replace or explicitly guard the timeout invariant path
- close the linked CLI construction/casing cleanups while the error surface is
  open

Planned branch:

- `feature/pN-fix-s5-cli-error-hardening`

### `NF6` Codex Pending Record Parse Hardening

Purpose:

- add structured error handling for `PendingRecord.from_json`
- keep the change isolated because it affects persisted debounce-state parsing

Planned branch:

- `feature/pN-fix-s6-codex-pending-record-hardening`

## Scope Estimates

| Sprint | Estimated scope | Why |
| --- | --- | --- |
| `NF1` | medium | many files, but low-risk cleanup with no intended semantic contract changes |
| `NF2` | medium | adds real harness proof coverage and touches live Gemini/Codex test execution paths |
| `NF3` | medium | doc-only, but affects the authoritative release/closure record and must reconcile multiple control docs |
| `NF4` | small | isolated core error-variant correction in one module |
| `NF5` | medium | CLI error/reporting and invariant enforcement touch user-facing behavior and panic resistance |
| `NF6` | small | one isolated structured parse-hardening change |

## Non-Goals

- new Codex or Gemini runtime adapter implementation
- changing the `PARTIAL_GO` substance of the finalized `Phase N` verdict
- expanding canonical `N.3` mappings
- re-opening deferred Codex `notify` / `Stop` / `resume` / `fork` decisions
- re-opening deferred Gemini `AfterAgent` promotion
