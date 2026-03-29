# Cross-Platform Guidelines

This document records the minimum platform-portability rules for `schook`
runtime, harness, and documentation work. These rules apply to code, tests,
fixtures, and examples.

## Path Rules

- build paths with `Path` / `PathBuf` joins, never string concatenation
- do not hardcode machine-local absolute paths
- do not assume Unix-only separators or `/tmp`
- prefer `$HOME` notation in docs when a home-relative example is needed
- resolve home directories through the platform resolver, not custom shell
  expansion

## State, Logs, And Tempfiles

- canonical hook session-state storage must resolve from
  `SC_HOOKS_STATE_DIR` first, then the platform home-directory resolver
- canonical observability/log roots must resolve through the shared storage
  helper, not per-crate cwd joins
- same-directory temp-plus-rename is required for atomic state writes
- use `tempfile` for temporary files and Windows-safe rename behavior instead
  of hand-rolled temp naming
- when securing a tempfile on Unix, prefer file-descriptor-based permission
  changes over path-based chmod after creation

## Test And Fixture Rules

- test payloads must use cross-platform placeholders, not hardcoded `/tmp`
  paths
- harness fixtures, reports, and generated artifacts must use repo-relative
  locations
- if a test needs a temp directory, derive it from `tempfile` / platform temp
  APIs rather than embedding literals

## Documentation Rules

- docs must distinguish current source-crate names from future packaging names
  when those names intentionally differ
- cross-repo references should use relative repo docs when possible, or clearly
  marked external references when not
- examples expected to work cross-platform must avoid shell-specific assumptions
  unless called out explicitly
