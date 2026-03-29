# Cross-Platform Guidelines

This document records the minimum portability rules for `schook` runtime,
harness, and documentation work.

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
