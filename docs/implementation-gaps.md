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
  - `Phase O` note: this ruling remains closed under the accepted
    unsealed-trait decision and is not reopened by `O.1`
  - deferral note: sealed-trait migration is deferred until the public API
    stabilization gate for the next release-track boundary; any new trait
    methods must carry default implementations until that stabilization sprint is
    explicitly scheduled

### RULING-NEEDED-NT-CLI-002: Raw Hook And Plugin Identifiers At Dispatch Boundaries

- Status: `closed in O.6`
- Owner area:
  - `sc-hooks-cli`, docs
- Closure note:
  - the planned closeout remained assigned to `O.6`, and `Phase P` does not
    reopen it as an active ruling item
  - any future CLI typing expansion beyond the `O.6` closure requires a new
    discrete ruling or implementation-gap entry rather than silently carrying
    `RULING-NEEDED-NT-CLI-002` forward

## Deferred Items

### RULING-NEEDED-ECR-001: `HookError` Surface Split

- Status: `deferred past Phase P`
- Owner area:
  - `sc-hooks-core`, `sc-hooks-sdk`, docs
- Recorded owner:
  - `randlee`
- Deferral note:
  - `HookError` remains a single cross-crate error enum spanning payload,
    validation, state-I/O, divergence, and internal failures
  - `Phase O` and `Phase P` seam additions stay inside
    `HookError::Normalization { message, source }`; splitting that surface now
    would be a public API break across the core/sdk boundary
  - the next release-track decision must explicitly choose between a stable
    multi-type taxonomy and a deliberate freeze of the current monolithic enum

### RULING-NEEDED-ECR-002: Backtrace Capture Policy

- Status: `deferred past Phase P`
- Owner area:
  - `sc-hooks-core`, `sc-hooks-sdk`, docs
- Recorded owner:
  - `randlee`
- Deferral note:
  - adding `Backtrace` capture to public error types changes error layout,
    serialization assumptions, and support expectations across the core/sdk
    boundary
  - the current release track keeps source chaining intact without introducing
    a partially scoped backtrace policy
  - any future backtrace policy should land together with the next explicit
    error-surface decision rather than as an isolated mid-track expansion

## Closed Items

### RULING-NEEDED-HRN-005: Library-Owned `worktree_hooks` Test Module

- Status: `closed in P.7`
- Owner area:
  - `sc-hooks-test`, docs
- Closure note:
  - `worktree_hooks.rs` remains in `src/` under `#[cfg(unix)]` so the shared
    shell fixture helpers stay reusable from one crate-local test surface
  - this is accepted as a Unix-gated test-only helper rather than a runtime
    portability defect; moving it to `tests/` would force extra public helper
    exposure or duplicate fixture wiring without changing the runtime contract
    being proved

### RULING-NEEDED-COW-003: Allocation-Backed Handler Chain Snapshot

- Status: `closed in P.7`
- Owner area:
  - `sc-hooks-cli`, docs
- Closure note:
  - `execute_chain()` keeps the owned `Vec<String>` handler snapshot because
    dispatch-complete and full-audit emission need a chain value independent of
    handler iterator lifetimes and result construction
  - this allocation remains the accepted current posture; revisit only if
    profiling later proves it is a real hot-path cost worth redesigning

### PRR-009: Missing `hooks` CLI Alias

- Status: `closed in P.8`
- Owner area:
  - packaging, install docs, release docs
- Closure note:
  - the local install/cutover path now writes a real `hooks` wrapper beside the
    canonical `sc-hooks` binary under `~/.local/bin/`
  - operator docs now distinguish between the canonical binary and the alias
    install path instead of implying the alias appears everywhere automatically

### LOGR-QA-004: Exhausted Retry Path Coverage For Shared Spawn Helper

- Status: `closed in P.8`
- Owner area:
  - `sc-hooks-core`, `sc-hooks-test`, docs
- Closure note:
  - `retry_executable_file_busy()` retains the shared bounded retry behavior
    used by the host and test harness
  - the helper now has direct proof for:
    - retry-before-success
    - immediate non-retryable failure
    - fully exhausted `ExecutableFileBusy` retry budget returning the final
      retryable error

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
