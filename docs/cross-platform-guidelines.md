# Cross-Platform Guidelines

This document records the minimum cross-platform constraints for `schook`
runtime and harness code.

## Path Rules

- build paths with path-join APIs, not string concatenation
- do not hardcode absolute machine-local paths
- do not hardcode Unix-only separators
- keep user-home resolution behind the canonical home-directory resolver

## State And Tempfile Rules

- canonical hook session-state storage must resolve from `SC_HOOKS_STATE_DIR`
  first, then the platform home-directory resolver
- `/tmp` must not be used as canonical state storage
- atomic writes should use same-directory temp-file creation plus rename
- Windows-safe atomic write behavior should use the `tempfile` crate rather
  than assuming raw `std::fs::rename` is sufficient in all cases

## Harness Rules

- harness fixtures and reports must use repo-relative paths
- documentation should prefer `$HOME` notation over author-machine absolute
  paths when an example needs a home-relative location
