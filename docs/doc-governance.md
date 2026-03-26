# sc-hooks Documentation Governance

## 1. Core Rules

1. No public behavior change without updating `docs/requirements.md`.
2. No architecture section may describe an API or boundary as current unless code already implements it.
3. No new requirement without a concrete acceptance scenario.
4. No requirement may stay in the docs if it is neither release-required nor deferred.
5. No crate-boundary claim without a real code owner in the repository layout.
6. No plugin capability may be described as shipped unless runtime installation, runtime behavior, and tests exist.
7. Contract changes must update the relevant contract doc and `docs/traceability.md`.
8. Implementation gaps must be explicit in `docs/implementation-gaps.md`, not hidden inside normative prose.

## 2. Source-Of-Truth Split

- `docs/requirements.md`: release-facing behavior
- `docs/architecture.md`: current boundaries and execution model
- `docs/protocol-contract.md`: host/plugin JSON contract
- `docs/observability-contract.md`: current `sc-observability` event path and JSONL contract
- `docs/implementation-gaps.md`: missing, overstated, or not-yet-release-ready behavior
- `docs/traceability.md`: requirement-to-code/test/gap mapping
- `README.md`: contributor orientation only; it must not be treated as the primary owner of release-contract details

## 3. Review Checklist

Before merging a docs or behavior change, verify:

- the change updates the right source-of-truth document
- any new or changed requirement has an acceptance scenario
- any code change affecting public behavior updates traceability
- any mismatch between docs and code is either fixed or logged as a gap
- any deferred item is clearly labeled as deferred in the requirements doc

## 4. Closeout Rules

An architecture or design-doc repair pass is not complete until:

- the affected source-of-truth documents have been updated together
- `docs/implementation-gaps.md` records every remaining release-relevant mismatch
- each open gap names an owner area, a verification method, and any early retire-or-replace candidates
- the handoff records the branch, commit, and validation commands used for the pass
- reviewer signoff and QA follow-up are recorded before the pass is treated as closed

If code review finds duplicated or stale implementation surfaces, flag them for early removal in the gap tracker before adding more behavior on top of them.

## 5. Anti-Drift Rules

- Do not mix future intent and current behavior in the same paragraph.
- Do not describe scaffold code as production functionality.
- Do not claim compliance-harness coverage that the harness does not assert directly.
- Do not add low-level JSON shape details to architecture when a contract doc should own them.
