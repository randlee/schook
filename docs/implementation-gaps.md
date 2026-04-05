# Implementation Gaps

This gap ledger remains the current implementation-gap reference for release
honesty, removals, and deferred work. Current control-doc ownership lives in:

- [docs/requirements.md](requirements.md)
- [docs/architecture.md](architecture.md)
- [docs/project-plan.md](project-plan.md)
- [docs/traceability.md](traceability.md)

## Deferred Items

### DEF-009: Observability Failure Fallback Integration Test

- Severity: `deferred`
- Owner area:
  - `sc-hooks-cli`, docs
- Current behavior:
  - runtime code writes `sc-hooks: failed emitting observability event: {err}`
    to `stderr` when structured observability emission fails during
    `dispatch.complete` or `session.root_divergence`
  - current integration coverage does not yet force an observability emit
    failure and assert that fallback stderr path end to end
- Exit condition:
  - an integration test drives a real `sc-hooks-cli` dispatch through a forced
    observability-emission failure and asserts the fallback stderr output
