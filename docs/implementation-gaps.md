# Implementation Gaps

This gap ledger remains the current implementation-gap reference for release
honesty, removals, and deferred work. Current control-doc ownership lives in:

- [docs/requirements.md](requirements.md)
- [docs/architecture.md](architecture.md)
- [docs/project-plan.md](project-plan.md)
- [docs/traceability.md](traceability.md)

## Active Items

### SEAL-001: SDK Trait-Sealing Decision

- Status: `closed in LOGR-COMP-FIX-1`
- Owner area:
  - `sc-hooks-sdk`, docs
- Closure note:
  - `ManifestProvider`, `SyncHandler`, and `AsyncHandler` remain intentionally
    unsealed because sibling runtime crates still implement them directly
  - the executable-plugin JSON contract is the current release boundary, but
    the SDK trait surface is still public for source-owned runtime crates
  - decision rationale: keep the current trait surface open for in-repo
    production-track plugin crates, and treat any future trait sealing as a
    deliberate architecture change requiring a migration plan rather than a
    silent hardening pass

### LOGR-POSTURE-001: Hook Runtime Control-Doc Reconciliation

- Status: `closed in CDR-B-1`
- Owner area:
  - control docs, README
- Closure note:
  - `docs/requirements.md`, `docs/architecture.md`, `docs/project-plan.md`,
    `docs/traceability.md`, and `README.md` now classify
    `agent-session-foundation`, `agent-spawn-gates`, `tool-output-gates`, and
    `atm-extension` as production-track runtime implementation source crates
    with direct tests
  - Hook Phases 3 through 5 are now recorded as landed baseline work rather than
    future-only planning targets
  - the reconciled docs keep all `plugins/` crates source-owned and avoid any
    bundled/preinstalled claim that the runtime/install docs do not prove

### ECR-001: `HookError` Surface Split

- Status: `closed in CDR-B-FIX-1`
- Owner area:
  - `sc-hooks-core`, `sc-hooks-sdk`, docs
- Closure note:
  - `sc-hooks-core::errors` now distinguishes payload/validation failures
    (`PayloadError`) from runtime/persistence failures (`RuntimeError`)
  - `HookError` remains as a thin compatibility wrapper so existing helper APIs
    and source-owned runtime crates can migrate without a second public-surface
    break
  - the SDK handler boundary now references the split taxonomy through the
    handler-facing alias rather than treating the old monolithic enum as the
    primary design

### ECR-002: Backtrace Capture Policy

- Status: `closed in CDR-B-FIX-1`
- Owner area:
  - `sc-hooks-core`, `sc-hooks-sdk`, docs
- Closure note:
  - `RuntimeError::StateIo` and `RuntimeError::Internal` now capture
    `std::backtrace::Backtrace` at construction time
  - the repository is pinned to stable Rust `1.94.1`, so those variants retain
    `Box<Backtrace>` fields for now because unboxed `Backtrace` triggers
    unstable `error_generic_member_access` paths through `thiserror`
  - cleanup when the toolchain advances: unbox the field and remove the
    `Box::new(Backtrace::capture())` wrapper once stable `thiserror`/compiler
    support makes `Error::request_ref::<Backtrace>()` available
  - backtrace rendering follows standard Rust behavior: capture occurs
    unconditionally, while visible detail still depends on the operator’s
    `RUST_BACKTRACE` environment policy
  - payload/validation errors remain backtrace-free so the public error surface
    does not imply stack traces for ordinary contract failures

### RULING-NEEDED-TS-001: Ended-State Transition Guard

- Status: `closed in SC-LOG-PRR-FIX-R6-TS`
- Owner area:
  - `sc-hooks-core`, docs
- Closure note:
  - `ActiveSessionRecord::apply_hook_update()` and
    `ActiveSessionRecord::rebuild_with_root_change()` now return a validation
    error when asked to transition directly to `AgentState::Ended`
  - `ActiveSessionRecord::transition_to_ended()` is the dedicated terminal
    transition path, and `agent-session-foundation` now uses it for the
    `SessionEnd` flow instead of routing terminal state through
    `apply_hook_update()`
  - decision rationale: keep runtime enforcement for this release so persisted
    canonical records and resume flows remain stable, while explicitly blocking
    implicit terminal transitions until a larger typestate redesign is
    intentionally approved

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

### PRR-009: Missing `hooks` CLI Alias

- Status: `active`
- Owner area:
  - packaging, install docs, release docs
- Current note:
  - the naming direction is frozen on `sc-hooks` as canonical with `hooks` as a
    convenience alias, but this repo does not yet ship an alias wrapper,
    symlink, or install-time alias mechanism
  - release/docs work should not imply that invoking `hooks` is already a
    guaranteed supported path until packaging or install output creates that
    alias explicitly
  - recommendation: implement the alias in release packaging/install flow or
    downgrade any remaining “supported alias” language to planned follow-on text

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
