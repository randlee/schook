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

- adopt the new `sc-lint` tooling in this repo before runtime-normalization
  code begins
- enforce one hard lint-detected architectural boundary for normalization work

## Hard Dependencies

- `O.1` complete
- local or Homebrew-installed `sc-lint` release `0.1.x`, or the exact
  repo-local fallback from `../sc-lint`

## Exact Targets

- `justfile`
- `.just/`
- `Cargo.toml`
- `crates/sc-hooks-core/Cargo.toml`
- `crates/sc-hooks-cli/Cargo.toml`
- `crates/sc-hooks-sdk/Cargo.toml`
- `boundaries/`
- docs describing the repo-local lint surface

## Deliverables

- repo-local `sc-lint` command surface following the `../sc-lint` pattern
- boundary definitions for the normalization seam
- lint commands that fail when the normalization boundary is bypassed

## Acceptance Criteria

- the repo exposes `sc-lint` entrypoints in the same top-level pattern used by
  `../sc-lint` for help, lint, and CI-oriented lint invocation
- `boundaries/` records exist for the normalization seam the later runtime
  sprints will rely on
- boundary lint runs in this repo and can detect `internal_only` and
  `forbid_external_impls` violations on the normalization boundary
- the sprint documents whether the machine is using the Homebrew-installed
  `sc-lint` binary or the explicit repo-local fallback path

## Out Of Scope

- provider runtime parity
- local cutover

## Required Validation

- `just help`
- `just lint sc-boundary`
- `git diff --check`
