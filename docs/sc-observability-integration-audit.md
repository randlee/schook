# sc-observability Integration Audit

## Scope

This note records the `SC-OBS-INTEGRATION-1` audit of the current `schook`
integration with the sibling `sc-observability` repository on `develop`.

Checked areas:
- dependency wiring from `sc-hooks-cli`
- CI validation shape for the sibling workspace
- dispatch-complete and root-divergence observability emission
- fallback behavior when observability setup or emission is degraded
- warning/error paths that occur before logger initialization

## Changes In This Pass

- `sc-hooks-cli` now references the sibling `sc-observability` checkout through
  the normal repository-relative path used by the main checkout:
  `../../../sc-observability/...`
- CI now clones `sc-observability` from `develop` into the matching sibling
  path layout instead of cloning `main` two levels above the checkout
- invalid `SC_HOOKS_ENABLE_*` values now write a guaranteed warning to `stderr`
  instead of relying on the uninitialized `log` facade
- dispatch observability emission fallback now writes directly to `stderr` in
  addition to `error!`, matching the documented contract
- metadata stale-file clock-skew warnings now also write directly to `stderr`
- stale metadata cleanup failures now warn on `stderr` instead of being dropped

## Findings

### Fixed

1. Invalid sink-toggle warnings were previously weak.
   - Prior behavior: `env_flag()` used `warn!` before any logger backend
     existed, so the documented stderr warning could be dropped silently.
   - Current behavior: the warning is emitted to both `warn!` and `stderr`, and
     the integration test now covers the stderr path.

2. Observability emission fallback was previously weaker than documented.
   - Prior behavior: `emit_observability_stderr_fallback()` only called
     `error!`, even though the docs stated the fallback was on `stderr`.
   - Current behavior: the fallback writes the rendered error directly to
     `stderr` as documented, and the fallback text no longer incorrectly says
     "dispatch" when the failing event is `session.root_divergence`.
   - Coverage note: this pass changes the runtime code path and the contract
     docs, but there is not yet a dedicated integration test that forces an
     observability emit failure and asserts the fallback stderr text.

3. Stale metadata cleanup previously dropped removal failures.
   - Prior behavior: stale temp-file cleanup ignored `remove_file` failures,
     which meant a useful warning could disappear entirely during metadata
     maintenance.
   - Current behavior: failed stale-file removal emits a warning to `stderr`
     through the same guaranteed warning helper used for other pre-init paths.

### Remaining Caveat

1. Cargo path dependencies cannot simultaneously match both checkout layouts
   without local environment help.
   - Main checkout layout (`.../github/schook`) needs
     `../../../sc-observability/...` from `crates/sc-hooks-cli`.
   - Required worktree layout (`.../github/schook-worktrees/<branch>`) needs a
     parent-level alias so the same Cargo path resolves during local worktree
     validation.
   - For this sprint, validation in the assigned worktree uses a local alias in
     the shared `schook-worktrees/` parent. Release packaging still cannot ship
     with path dependencies and remains gated separately by release workflows.

## Audit Summary

- Dispatch-complete logging is implemented and contract-tested.
- Root-divergence logging is implemented and contract-tested.
- The shared observability-failure stderr fallback is implemented in code, but
  dedicated integration coverage for that degraded path remains deferred.
- File sink remains the canonical structured surface.
- Console sink remains a debugging/operator surface.
- The primary remaining non-release caveat is local worktree path ergonomics
  while unpublished sibling path dependencies are still in use.
