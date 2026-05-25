# Cross-Platform Guidelines

This document records the minimum portability rules for `sc-hooks` runtime,
harness, and documentation work.

## Phase P Gate

- `just lint portability` is a required pre-merge gate for `Phase P`
  runtime-facing work
- new Codex/Gemini runtime parity changes are not allowed to rely on
  Mac-only/Unix-only implementation paths without an explicit cross-platform
  fallback in the same change
- `cfg(unix)` is allowed only when the non-Unix compile path is also defined
  and the public behavior remains explicit

## Path Rules

- build filesystem paths with `Path` / `PathBuf` joins, never string
  concatenation
- avoid machine-local absolute paths in code, fixtures, tests, and docs
- do not assume Unix-only separators or `/tmp`
- prefer `$HOME` notation in docs when a home-relative example is needed

## State, Logs, And Tempfiles

- canonical session-state storage resolves from `SC_HOOKS_STATE_DIR` first,
  then the platform home-directory resolver
- canonical observability/log roots resolve through the shared storage helper,
  not per-crate cwd joins
- same-directory temp-plus-rename is required for atomic state writes
- use `tempfile` for temporary files and cross-platform rename behavior
- on Unix, secure tempfiles via file-descriptor-based permission changes rather
  than path-based chmod after creation

## Permissions

- secrets, state files, and metadata files that must be private should use
  Unix owner-only permissions (`0o600`) behind `cfg(unix)` guards
- Unix-only permission logic should use `std::os::unix::fs::PermissionsExt`
  from an open file descriptor or already-created file handle rather than
  shelling out or assuming chmod semantics everywhere
- Windows and other non-Unix targets must compile cleanly without
  `PermissionsExt`; keep Unix-specific permission code isolated behind
  `cfg(unix)` and provide a no-op or platform-appropriate fallback elsewhere

## Process Handling

- signal-driven process management must be guarded with `cfg(unix)` and must
  not assume `SIGTERM`/`SIGKILL` are available cross-platform
- Windows process termination should use the Windows-native process APIs rather
  than Unix signal constants
- cross-platform process wrappers should keep the public contract at the
  `std::process` boundary when possible, with platform-specific termination
  behavior hidden behind small helper functions

## Harness And Tests

- test payloads must use cross-platform placeholders, not hardcoded `/tmp`
  paths
- repo artifacts and generated reports must use repo-relative paths
- if a test needs a temp directory, derive it from `tempfile` or platform temp
  APIs rather than embedding literals

## Environment And Docs

- provider-specific environment variables must have documented fallback rules
  and must not assume a shell-specific setup path
- cross-repo references should use repo-relative docs when possible, or clearly
  marked external references when not
- examples expected to work cross-platform must call out any shell-specific
  assumptions explicitly

## Phase P Runtime Parity Notes

- `P.5` and `P.6` did not add a new Unix-only runtime branch for Codex
  `notify` or Gemini `AfterAgent`; both retained lifecycle surfaces close
  through the shared normalized Rust runtime path
- the remaining Unix-gated behavior relevant to that parity claim is explicit:
  - host timeout escalation uses Unix `SIGTERM` first and a platform-native
    termination fallback elsewhere
  - `crates/sc-hooks-test/src/worktree_hooks.rs` is a Unix-only test helper,
    not part of the generic provider runtime parity contract
- docs must not describe Unix-first behavior as if it were the cross-platform
  runtime contract
