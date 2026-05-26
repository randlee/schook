---
id: P.3
title: sc-lint Suite Adoption And Cross-Platform Gate
status: complete
branch: feature/pP-s3-lint-portability
worktree: ../schook-worktrees/feature/pP-s3-lint-portability
target: integrate/phase-P
---

# Sprint P.3 — sc-lint Suite Adoption And Cross-Platform Gate

## Goal

- adopt the available `sc-lint` lint suite in `schook` using the curated
  `just lint` pattern from `../atm-core`
- make portability and boundary enforcement first-class repo gates before new
  runtime surfaces land

## Hard Dependencies

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
  - `boundary`
  - `portability`
- explicit cross-platform gate so new Phase P runtime work is not allowed to be
  Mac/Unix-only by default

## Required Contract Samples

Expected curated `just lint` entrypoint shape:

```just
default: help

help:
    {{python_cmd}} .just/print_help.py

lint target='all':
    {{python_cmd}} .just/run_lint.py {{target}}
```

The landed `justfile` may include private helper recipes, but `P.3` must
preserve one public `just lint <target>` entrypoint that mirrors the curated
`../atm-core` pattern.

Expected delegated hook-test recipe shape:

```just
test target provider:
    {{python_cmd}} .just/run_test.py {{target}} {{provider}}
```

`P.3-fix-R1` replaced the older inline POSIX-shell branching with this Python
delegate shape so the public `just test hooks <provider>` surface remains
portable once `portability` becomes a hard gate.

## Acceptance Criteria

- `schook` exposes the adopted `sc-lint` suite through the same curated
  `just lint` structure used in `../atm-core`
- the planned lint surface explicitly includes:
  - `lint fmt`
  - `lint clippy`
  - `lint modules`
  - `lint deny`
  - `lint shear`
  - `lint version`
  - `lint manifests`
  - `lint spell`
  - `lint pytests`
  - `lint boundary`
  - `lint portability`
- existing `just test hooks claude`, `just test hooks codex`, and
  `just test hooks gemini` entrypoints remain functional after the `justfile`
  restructuring; the new `just lint` surface is additive
- the Phase P runtime sprints treat cross-platform portability as a hard gate,
  not a late review note

## Out Of Scope

- lifecycle normalization changes
- provider runtime parity implementation
- packaging/CLI alias work

## Required Validation

- `just help`
- `just lint modules`
- `just lint boundary`
- `just lint portability`
- `git diff --check`

## Sprint QA Checklist

- Which requirement IDs or gap IDs changed status?
- What code was removed early rather than left in parallel?
- Which files/crates were the owned write scope for the sprint?
- What validation commands and direct tests proved the new contract?
- What backend label and clean PASS result did `just lint portability`
  report on the final branch state?
- What follow-on work is blocked or unblocked by this sprint?

## Sprint QA Checklist Answers

- Requirement IDs or gap IDs changed status:
  none; `P.3` added repo lint/control surfaces and the hard portability gate
  without changing a requirement status row directly.
- Code removed early rather than left in parallel:
  the inline POSIX-shell branching in the old `just test` implementation was
  removed in favor of the shared Python delegate path.
- Owned write scope:
  `justfile`, `.just/run_lint.py`, `.just/run_test.py`,
  `docs/sc-lint-boundary.md`, `docs/cross-platform-guidelines.md`,
  `docs/phase-P/sprint-P3.md`, and `docs/plan-phase-P.md`.
- Validation commands and direct tests:
  `just help`, `just lint modules`, `just lint boundary`,
  `just lint portability`, `just test hooks claude`,
  `just test hooks codex`, `just test hooks gemini`, `cargo test --workspace`,
  and `git diff --check`.
- `just lint portability` backend label and PASS result:
  `backend=repo-local:../sc-lint status=pass findings=0`, matching the
  machine-recorded PASS that uses the local `../sc-lint` checkout as the
  fallback portability backend until the published CLI is available.
- Follow-on work blocked or unblocked:
  unblocks `P.4`, `P.5`, `P.6`, and `P.7` to treat
  `just lint portability` as a required hard gate for new runtime changes.
