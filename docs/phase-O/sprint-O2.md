---
id: O.2
title: `sc-lint` Setup And Boundary Enforcement
status: planned
branch: feature/pO-s2-sc-lint-setup
worktree: ../schook-worktrees/feature/pO-s2-sc-lint-setup
target: integrate/phase-O
---

# Sprint O.2 — `sc-lint` Setup And Boundary Enforcement

## Goal

- adopt `sc-lint-boundary` in this repo through `just` wrappers before
  runtime-normalization code begins
- enforce one hard lint-detected architectural boundary for normalization work

## Hard Dependencies

- `O.1` complete
- local or Homebrew-installed `sc-lint-boundary` release `0.1.x`, or the exact
  repo-local fallback from `../sc-lint`

## Exact Targets

- `justfile`
- `.just/`
- `Cargo.toml`
- `crates/sc-hooks-core/Cargo.toml`
- `crates/sc-hooks-cli/Cargo.toml`
- `crates/sc-hooks-sdk/Cargo.toml`
- `boundaries/`
- `docs/sc-lint-boundary.md`

## Deliverables

- repo-local `just` wrapper surface following the top-level `../atm-core`
  pattern
- stable `just test hooks claude`, `just test hooks codex`, and
  `just test hooks gemini` entrypoints owned by this repo-level command surface
- boundary definitions for the normalization seam
- lint commands that fail when the normalization boundary is bypassed

## Required Signatures

Public `just` surface:

```just
default: help

help:
    {{python_cmd}} .just/print_help.py

lint target='all':
    {{python_cmd}} .just/run_lint.py {{target}}

[private]
_lint-sc-boundary:
    {{python_cmd}} .just/lint_sc_boundary.py

test hooks provider:
    {{python_cmd}} .just/run_hook_tests.py {{provider}}
```

## Acceptance Criteria

- the repo exposes `just` entrypoints in the same top-level pattern used by
  `../atm-core` for help, lint, and CI-oriented invocation
- the `just lint sc-boundary` path is backed by `sc-lint-boundary`, using the
  Homebrew-installed binary when available or the explicit repo-local fallback
  from `../sc-lint`
- the same `just` surface owns `just test hooks claude`, `just test hooks codex`,
  and `just test hooks gemini`
- `boundaries/` records exist for the normalization seam the later runtime
  sprints will rely on
- boundary lint runs in this repo and can detect `internal_only` and
  `forbid_external_impls` violations on the normalization boundary
- the sprint documents whether the machine is using the Homebrew-installed
  `sc-lint-boundary` binary or the explicit repo-local fallback path

## Out Of Scope

- provider runtime parity
- local cutover

## Required Validation

- `cargo check --workspace`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `just help`
- `just lint sc-boundary`
- `just test hooks claude`
- `git diff --check`
