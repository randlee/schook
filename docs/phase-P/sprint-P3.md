---
id: P.3
title: sc-lint Suite Adoption And Cross-Platform Gate
status: planned
branch: feature/pP-s3-sc-lint-suite-adoption
worktree: ../schook-worktrees/feature/pP-s3-sc-lint-suite-adoption
target: integrate/phase-P
---

# Sprint P.3 — sc-lint Suite Adoption And Cross-Platform Gate

## Goal

- adopt the available `sc-lint` lint suite in `schook` using the curated
  `just lint` pattern from `../atm-core`
- make portability and boundary enforcement first-class repo gates before new
  runtime surfaces land

## Hard Dependencies

- accepted `P.1`
- accepted `P.2`
- existing `sc-lint-boundary` baseline from `Phase O`
- local `../sc-lint` tool inventory

## Exact Targets

- `justfile`
- `.just/`
- `docs/sc-lint-boundary.md`
- `docs/cross-platform-guidelines.md`
- lint wrapper/config files needed to mirror the available `../sc-lint` tool
  surface in this repo

## Deliverables

- repo-local `just lint` structure aligned with `../atm-core`
- wrappers for the currently available `../sc-lint` lint targets:
  - `fmt`
  - `clippy`
  - `modules`
  - `deny`
  - `shear`
  - `version`
  - `manifests`
  - `spell`
  - `pytests`
  - `sc-boundary`
  - `sc-portability`
- explicit cross-platform gate so new Phase P runtime work is not allowed to be
  Mac/Unix-only by default

## Required Contract Samples

Expected curated `just lint` entrypoint shape:

```just
default: help

help:
    @python3 .just/print_help.py

lint target:
    @python3 .just/run_lint.py {{target}}
```

The landed `justfile` may include private helper recipes, but `P.3` must
preserve one public `just lint <target>` entrypoint that mirrors the curated
`../atm-core` pattern.

## Acceptance Criteria

- `schook` exposes the adopted `sc-lint` suite through the same curated
  `just lint` structure used in `../atm-core`
- the planned lint surface explicitly includes:
  - `lint modules`
  - `lint deny`
  - `lint shear`
  - `lint version`
  - `lint manifests`
  - `lint spell`
  - `lint pytests`
  - `lint sc-boundary`
  - `lint sc-portability`
- the Phase P runtime sprints treat cross-platform portability as a hard gate,
  not a late review note

## Out Of Scope

- lifecycle normalization changes
- provider runtime parity implementation
- packaging/CLI alias work

## Required Validation

- `just help`
- `just lint modules`
- `just lint sc-boundary`
- `just lint sc-portability`
- `git diff --check`
