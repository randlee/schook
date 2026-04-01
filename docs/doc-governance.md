# sc-hooks Documentation Governance

## 1. Enforceable Rules

1. No public behavior change without updating `docs/requirements.md` and `docs/traceability.md`.
2. No architecture section may describe an API, boundary, or runtime path as current unless code implements it now; otherwise mark it deferred and link the gap or deferred ID.
3. No new requirement without an acceptance scenario in `docs/requirements.md`.
4. No crate-boundary claim without both a named owner crate and an enforcement note in `docs/architecture.md`.
5. No contract change without updating the owning contract doc and `docs/traceability.md`.
6. Supporting docs must reference the requirement IDs or gap IDs they implement when they describe release-facing behavior. Inline backtick references such as `OBS-006` or `GAP-005` satisfy this rule unless the doc also maintains a stronger local anchor or table entry.
7. No intentional removal, deferral, or unsupported surface may disappear silently; it must be recorded in `docs/implementation-gaps.md` or `docs/requirements.md`.

## 2. Source-Of-Truth Split

- `docs/requirements.md`: normative release-facing behavior and requirement IDs
- `docs/architecture.md`: current architecture, owners, and enforcement notes
- `docs/project-plan.md`: derived execution plan only
- `docs/sc-hooks-cli/`, `docs/sc-hooks-core/`, `docs/sc-hooks-sdk/`: crate-local ownership docs
- `docs/protocol-contract.md`: host/plugin wire contract
- `docs/observability-contract.md`: observability ownership boundary and file layout
- `docs/logging-contract.md`: current JSONL dispatch-log schema for consumers
- `docs/implementation-gaps.md`: explicit mismatches, removals, and deferred work
- `docs/traceability.md`: requirement-to-code/test/gap mapping

## 3. Docs PR Gate

Before any PR touching docs or behavior merges to `develop`, verify:

- the owning source-of-truth doc changed with the code
- `docs/traceability.md` was updated for any public behavior or contract change
- new or changed requirements include acceptance scenarios
- architecture edits only describe implemented behavior unless marked deferred with a linked gap or deferred ID
- any intentional removal or unproven behavior is recorded in `docs/implementation-gaps.md` or deferred in `docs/requirements.md`
- contract docs and consumer docs reference the requirement IDs or gap IDs they implement; inline backtick IDs are sufficient unless a stronger local anchor exists
- the PR records the validation commands used for the change

## 4. Sprint Kickoff Gate

Before implementation starts for a sprint, record:

- exact requirement IDs and gap IDs in scope
- the single owning implementation path for the affected behavior
- code or docs that must be removed before new behavior lands
- direct tests expected to fail before the sprint and pass after it
- every doc that must change in the same PR
- the file or crate write scope so parallel work does not overlap

## 5. Release Preflight Gate

Before release or final QA signoff, verify:

- every strong release-facing claim is backed by code, tests, or an explicit gap/deferred ID
- no duplicate implementation path remains for the same release-facing behavior unless a compatibility exception is documented
- every non-blocking QA advisory is either verified with a cited code path or converted into an explicit gap note
- no high-risk doc/code/API misalignment class remains outside `docs/project-plan.md` or `docs/implementation-gaps.md`
- requirements, architecture, traceability, project plan, and contract docs describe the same final scope
- the final handoff records the frozen branch head and validation commands used for QA/reviewer signoff
