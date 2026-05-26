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

### RULING-NEEDED-ECR-001: `HookError` Surface Split

- Status: `deferred past Phase O`
- Owner area:
  - `sc-hooks-core`, `sc-hooks-sdk`, docs
- Recorded owner:
  - `randlee`
- Current note:
  - `HookError` is still a single cross-crate error enum spanning payload,
    validation, state-I/O, divergence, and internal failures
  - splitting it now would be a public API break across the core/sdk surface and
    should not be done implicitly inside the observability closeout
  - disposition for `Phase O`: explicitly deferred past `Phase O` so `O.3`
    may attach provider-normalization failures at
    `HookError::Normalization { message, source }` without reopening the public
    error-surface split mid-sprint; the named `NormalizationError` inventory
    remains the private source behind that envelope
  - `Phase P` consistency note:
    - `P.4` seam additions (`CodexHook::Notify` -> `CanonicalPayload::StopLifecycle`
      and `GeminiHook::AfterAgent` -> `CanonicalPayload::StopLifecycle`) remain
      consistent with the existing `HookError::Normalization { message, source }`
      envelope
    - the `P.5` seam additions for Codex `notify` via the retained
      stop-family normalization path remain inside the existing
      `HookError::Normalization` envelope and do not reopen the error-surface
      split
    - `P.6` plugin-side runtime seam additions that carry Gemini `AfterAgent`
      through the shared `Stop` path inside `agent-session-foundation` and
      `atm-extension` remain inside that same
      `HookError::Normalization { message, source }` envelope and do not
      reopen the error-surface split
    - `P.7` may not retroactively remove those landed seam additions without a
      new breaking-change sprint
  - recommendation: take an explicit architecture ruling after `Phase O` on
    whether the next release track wants a stable multi-type error taxonomy or
    to freeze the current monolithic enum deliberately

### RULING-NEEDED-ECR-002: Backtrace Capture Policy

- Status: `deferred past Phase P`
- Recorded owner:
  - `randlee`
- Owner area:
  - `sc-hooks-core`, `sc-hooks-sdk`, docs
- Current note:
  - adding `Backtrace` capture to public error types changes error layout,
    serialization assumptions, and support expectations across the core/sdk
    boundary
  - the current release keeps source chaining intact without introducing a
    partially scoped backtrace policy
  - no implementation work landed on the `ECR-002` surface in `Phase P`; the
    accepted `ECR-001` seam additions remain isolated from this broader
    backtrace-boundary question
  - deferred to `Phase Q` or later, with the recorded owner responsible for
    re-evaluating the product-wide backtrace policy together with any future
    public error-surface split

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

- Status: `closed in O.6`
- Owner area:
  - `sc-hooks-cli`, docs
- Closure note:
  - the planned closeout remained assigned to `O.6`, and `Phase P` does not
    reopen it as an active ruling item
  - any future CLI typing expansion beyond the `O.6` closure requires a new
    discrete ruling or implementation-gap entry rather than silently carrying
    `RULING-NEEDED-NT-CLI-002` forward

### RULING-NEEDED-HRN-005: Library-Owned `worktree_hooks` Test Module

- Status: `closed in P.7`
- Owner area:
  - `sc-hooks-test`, docs
- Closure note:
  - `worktree_hooks.rs` remains in `src/` under `#[cfg(unix)]` so the shared
    shell fixture helpers stay reusable from one crate-local test surface
  - moving it to `tests/` would force extra public helper exposure or duplicate
    fixture wiring without changing the runtime contract being proved
  - accepted posture: keep the unix-gated library test module in place until a
    larger `sc-hooks-test` surface split is approved
  - the non-Unix path already uses `std::process::Command`, which is the
    platform-native process surface and does not require a second test-module
    ownership pattern

### RULING-NEEDED-COW-003: Allocation-Backed Handler Chain Snapshot

- Status: `closed in P.7`
- Owner area:
  - `sc-hooks-cli`, docs
- Closure note:
  - `execute_chain()` still clones handler names into a `Vec<String>` because
    dispatch-complete and full-audit emission need an owned chain snapshot that
    survives independent result construction and error returns
  - removing that allocation cleanly would require a broader change to the
    observability/audit argument surface rather than a small mechanical patch
  - accepted posture: keep the owned `Vec<String>` snapshot for the current
    load profile
  - any future `Cow` refactor remains deferred pending profiling evidence that
    the allocation is a real hot-path cost

### PRR-009: Missing `hooks` CLI Alias

- Status: `closed in P.8`
- Owner area:
  - packaging, install docs, release docs
- Current note:
  - `P.8` closed this by adding the install-time alias wrapper through
    `ensure_cli_alias()` in `crates/sc-hooks-cli/src/install.rs`
  - `USAGE.md` now documents the supported `hooks` alias path alongside the
    canonical `sc-hooks` install/cutover flow
  - `docs/project-plan.md` records `PRR-009` as closed by `P.8`

### LOGR-QA-004: Exhausted Retry Path Coverage For Shared Spawn Helper

- Status: `closed in P.8`
- Owner area:
  - `sc-hooks-core`, `sc-hooks-test`, docs
- Current note:
  - `P.8` closed this by adding direct exhausted-retry-path coverage in
    `crates/sc-hooks-core/src/process.rs` via
    `returns_final_executable_file_busy_after_retry_budget_exhausted()`
  - the shared retry helper remains single-owned and now has explicit proof for
    successful retry, immediate non-retryable failure, and retry-budget
    exhaustion
  - `docs/project-plan.md` records `LOGR-QA-004` as closed by `P.8`, and the
    merged integration baseline passes `cargo test --workspace`

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
