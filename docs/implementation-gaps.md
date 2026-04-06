# Implementation Gaps

This gap ledger remains the current implementation-gap reference for release
honesty, removals, and deferred work. Current control-doc ownership lives in:

- [docs/requirements.md](requirements.md)
- [docs/architecture.md](architecture.md)
- [docs/project-plan.md](project-plan.md)
- [docs/traceability.md](traceability.md)

## Active Items

### SEAL-001: SDK Trait-Sealing Decision

- Status: `active`
- Owner area:
  - `sc-hooks-sdk`, docs
- Current note:
  - `ManifestProvider`, `SyncHandler`, and `AsyncHandler` remain intentionally
    unsealed because sibling runtime crates still implement them directly
  - the executable-plugin JSON contract is the current release boundary, but
    the SDK trait surface is still public for source-owned runtime crates
  - any future trait sealing requires an explicit architecture ruling and a
    migration plan for the in-repo runtime crates

### RULING-NEEDED-ECR-001: `HookError` Surface Split

- Status: `active`
- Owner area:
  - `sc-hooks-core`, `sc-hooks-sdk`, docs
- Current note:
  - `HookError` is still a single cross-crate error enum spanning payload,
    validation, state-I/O, divergence, and internal failures
  - splitting it now would be a public API break across the core/sdk surface and
    should not be done implicitly inside the observability closeout
  - recommendation: take an explicit architecture ruling on whether the next
    release track wants a stable multi-type error taxonomy or to freeze the
    current monolithic enum deliberately

### RULING-NEEDED-ECR-002: Backtrace Capture Policy

- Status: `active`
- Owner area:
  - `sc-hooks-core`, `sc-hooks-sdk`, docs
- Current note:
  - adding `Backtrace` capture to public error types changes error layout,
    serialization assumptions, and support expectations across the core/sdk
    boundary
  - the current release keeps source chaining intact without introducing a
    partially scoped backtrace policy
  - recommendation: decide the product-wide backtrace policy together with any
    future error-surface split so the public error contract changes once

### RULING-NEEDED-TS-001: Ended-State Transition Guard

- Status: `active`
- Owner area:
  - `sc-hooks-core`, docs
- Current note:
  - `apply_hook_update()` and `rebuild_with_root_change()` still accept
    `AgentState::Ended` without a type-level guard because HKR-009 requires
    canonical session records to round-trip through file-backed state updates
  - tightening that API to a non-ended newtype would force broader record
    reconstruction changes across the persisted session path late in phase-end
    closeout
  - recommendation: keep the current validated runtime guard for this release
    and decide in a follow-on whether a separate `ActiveAgentState` type is
    worth the migration cost

### RULING-NEEDED-NT-CLI-002: Raw Hook And Plugin Identifiers At Dispatch Boundaries

- Status: `active`
- Owner area:
  - `sc-hooks-cli`, docs
- Current note:
  - hook types are now typed at the main resolution and dispatch boundaries, but
    some CLI-facing plugin and matcher identifiers still remain `String`-backed
    because they are assembled from config and manifest data used directly by
    observability/audit output
  - forcing a full newtype conversion in the phase-end fix pass would widen the
    API churn beyond the targeted blocker set
  - recommendation: schedule a focused cleanup if the team wants typed wrapper
    boundaries for plugin names and matcher IDs, instead of doing it implicitly
    in the observability closeout

### RULING-NEEDED-HRN-005: Library-Owned `worktree_hooks` Test Module

- Status: `active`
- Owner area:
  - `sc-hooks-test`, docs
- Current note:
  - `worktree_hooks.rs` remains in `src/` under `#[cfg(unix)]` so the shared
    shell fixture helpers stay reusable from one crate-local test surface
  - moving it to `tests/` would force extra public helper exposure or duplicate
    fixture wiring without changing the runtime contract being proved
  - recommendation: keep the unix-gated library test module in place until a
    larger `sc-hooks-test` surface split is approved

### RULING-NEEDED-COW-003: Allocation-Backed Handler Chain Snapshot

- Status: `active`
- Owner area:
  - `sc-hooks-cli`, docs
- Current note:
  - `execute_chain()` still clones handler names into a `Vec<String>` because
    dispatch-complete and full-audit emission need an owned chain snapshot that
    survives independent result construction and error returns
  - removing that allocation cleanly would require a broader change to the
    observability/audit argument surface rather than a small mechanical patch
  - recommendation: keep the owned snapshot for now and revisit only if profiling
    shows it is a real hot-path cost

## Closed Items

### DEF-009: Observability Failure Fallback Integration Test

- Status: `closed in SC-LOG-S6`
- Owner area:
  - `sc-hooks-cli`, docs
- Closure note:
  - integration coverage now forces logger-init, emit, append, and prune
    degradation paths through the real `sc-hooks-cli` runtime
  - the closing tests are:
    - `standard_mode_logger_init_failure_is_non_blocking`
    - `standard_mode_emit_failure_is_non_blocking`
    - `full_mode_logger_init_failure_is_non_blocking`
    - `full_mode_append_failure_is_non_blocking`
    - `full_mode_prune_failure_is_non_blocking`
  - those tests prove the degraded stderr fallback remains visible while hook
    exits do not change

### DEF-015: Non-Blocking Observability And Audit Failures

- Status: `closed in SC-LOG-S6`
- Owner area:
  - `sc-hooks-cli`, docs
- Closure note:
  - the same five integration tests above now prove logger-init, emit,
    append, and prune failures are all best-effort and never change hook
    execution outcomes
