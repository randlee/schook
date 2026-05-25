---
id: Q.2
title: Smoke Infrastructure
status: planned
branch: feature/pQ-s2-smoke-infrastructure
worktree: ../schook-worktrees/feature/pQ-s2-smoke-infrastructure
target: integrate/phase-Q
---

# Sprint Q.2 — Smoke Infrastructure

## Purpose

- add the curated `just smoke` surface following the `../atm-core` pattern
- create the repo-owned smoke runner and CI ownership path
- keep smoke execution separate from both `just test` and `just lint`

## Entry Criteria

- accepted `Q.1` openshell recommendation

## Exact Targets

- `justfile`
- `.just/`
- `.github/workflows/ci.yml`
- smoke operator docs

## Deliverables

- `just smoke` public entrypoint
- repo-owned smoke runner implementation
- CI job or CI extension for smoke execution
- operator docs for the smoke surface

## Acceptance Criteria

- `just smoke` exists and is documented
- the implementation pattern follows the curated `../atm-core` style
- CI owns the smoke surface explicitly

## Required Validation

- `cargo test --workspace`
- `git diff --check`
