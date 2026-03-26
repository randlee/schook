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
- `docs/logging-contract.md`: JSONL record shapes and logging guarantees
- `docs/implementation-gaps.md`: missing, overstated, or not-yet-release-ready behavior
- `docs/traceability.md`: requirement-to-code/test/gap mapping

## 3. Review Checklist

Before merging a docs or behavior change, verify:

- the change updates the right source-of-truth document
- any new or changed requirement has an acceptance scenario
- any code change affecting public behavior updates traceability
- any mismatch between docs and code is either fixed or logged as a gap
- any deferred item is clearly labeled as deferred in the requirements doc

## 4. Anti-Drift Rules

- Do not mix future intent and current behavior in the same paragraph.
- Do not describe scaffold code as production functionality.
- Do not claim compliance-harness coverage that the harness does not assert directly.
- Do not add low-level JSON shape details to architecture when a contract doc should own them.
