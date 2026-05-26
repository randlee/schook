---
id: O.7
title: Local Deployment And Cutover
status: complete
branch: feature/pO-s7-local-cutover
worktree: ../schook-worktrees/feature/pO-s7-local-cutover
target: integrate/phase-O
---

# Sprint O.7 — Local Deployment And Cutover

## Goal

- deploy the normalized `sc-hooks` runtime for the supported agents on this
  computer
- finish with a rollback-safe local cutover path

## Hard Dependencies

- `O.6` complete
- current `origin/integrate/phase-O` branch head
- local access to the provider runtime config paths on this machine

## Exact Targets

- `crates/sc-hooks-cli/src/install.rs`
- `justfile`
- `README.md`
- `USAGE.md`
- local provider runtime config paths documented by the sprint

## Deliverables

- `crates/sc-hooks-cli/src/install.rs` helper behavior that writes the local
  provider install/cutover plan used by this sprint
- one local install/cutover path for Claude, Codex, and Gemini on this machine
- rollback instructions
- machine-local smoke-test record for the supported providers

## Required Signatures

```rust
pub(crate) fn write_local_provider_cutover(
    provider: TargetProvider,
) -> Result<InstallPlan, InstallError>;

pub(crate) enum InstallError {
    UnsupportedProvider {
        provider: TargetProvider,
        supported: &'static [&'static str],
    },
    MissingProviderConfig { provider: TargetProvider, path: PathBuf },
    WriteFailed { path: PathBuf, reason: String },
    RollbackPlanFailed { provider: TargetProvider, path: Option<PathBuf>, reason: String },
}
```

## Acceptance Criteria

- `crates/sc-hooks-cli/src/install.rs` implements the local provider cutover
  helper signature documented by this sprint, and that behavior is exercised
  once during the machine-local cutover validation
- the documented local cutover path installs the normalized runtime for Claude,
  Codex, and Gemini on this machine
- rollback steps are documented and tested once
- `write_local_provider_cutover` and `InstallError` use intentionally aligned
  `pub(crate)` visibility because the cutover helper remains an internal CLI
  surface rather than a new public SDK/API contract
- `InstallError::UnsupportedProvider` reports the rejected provider together
  with the supported provider list so the recovery path is operator-visible
- `InstallError::RollbackPlanFailed` includes `path: Option<PathBuf>` so the
  failed restore target is reported when one is known
- machine-local smoke tests prove the supported providers are using the
  normalized runtime path

## Out Of Scope

- Cursor runtime work
- deferred `Phase N` surfaces
- cross-machine fleet rollout beyond this computer

## Required Validation

- `cargo test --workspace`
- `cargo run -p sc-hooks-cli -- install`
- `just test hooks claude`
- `just test hooks codex`
- `just test hooks gemini`
- `git diff --check`
