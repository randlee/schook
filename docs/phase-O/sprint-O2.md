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
- local or Homebrew-installed `sc-lint-boundary` release `0.1.0`, or the exact
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
- `docs/sc-lint-boundary.md` documentation for the public
  `just lint sc-boundary` wrapper over the private `_lint-sc-boundary` recipe
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

# `just lint sc-boundary` dispatches through this private helper.

[private]
_lint-sc-boundary:
    {{python_cmd}} .just/lint_sc_boundary.py

test hooks provider:
    {{python_cmd}} .just/run_hook_tests.py {{provider}}

test hooks claude:
    {{python_cmd}} .just/run_hook_tests.py claude

test hooks codex:
    {{python_cmd}} .just/run_hook_tests.py codex

test hooks gemini:
    {{python_cmd}} .just/run_hook_tests.py gemini
```

Boundary record:

```toml
# boundaries/provider-normalization.toml
name = "provider-normalization"
crate = "crates/sc-hooks-core"

[attributes]
internal_only = ["crate::normalization::private"]
forbid_external_impls = ["crate::normalization::ProviderHookNormalizer"]
```

## Acceptance Criteria

- the repo exposes `just` entrypoints in the same top-level pattern used by
  `../atm-core` for help, lint, and CI-oriented invocation
- the `just lint sc-boundary` path is backed by `sc-lint-boundary`, using the
  Homebrew-installed `0.1.0` binary when available or the explicit repo-local
  fallback from `../sc-lint`
- the public `just lint sc-boundary` entrypoint is documented as a wrapper over the private `_lint-sc-boundary` recipe rather than a separate implementation path
- the same `just` surface owns explicit `just test hooks claude`,
  `just test hooks codex`, and `just test hooks gemini` entrypoints rather
  than relying on a parameterized acceptance shortcut
- `boundaries/` records exist for the normalization seam the later runtime
  sprints will rely on
- boundary lint runs in this repo and can detect `internal_only` and
  `forbid_external_impls` violations on the normalization boundary
- the sprint records and validates the exact lint-coverage scope: the
  `internal_only` module seam and the `ProviderHookNormalizer` trait, not the
  full canonical type set
- the sprint documents whether the machine is using the Homebrew-installed
  `sc-lint-boundary` binary or the explicit repo-local fallback path
- if neither the pinned Homebrew binary nor the documented repo-local fallback
  is available, `O.2` fails rather than silently skipping boundary enforcement

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
