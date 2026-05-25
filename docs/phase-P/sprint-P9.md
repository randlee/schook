---
id: P.9
title: Published sc-lint Portability Gate
status: complete
branch: feature/pP-s9-sc-lint-portability
worktree: ../schook-worktrees/feature/pP-s9-sc-lint-portability
target: integrate/phase-P
---

# Sprint P.9 — Published `sc-lint` Portability Gate

## Goal

- replace the source-checkout `../sc-lint` fallback in the public lint wrappers
  with the published `0.2.0` binaries
- add a CI gate that installs and runs the published boundary and portability
  analyzers
- prove the current `schook` workspace is already clean under the published
  portability analyzer

## Hard Dependencies

- accepted `P.8`

## Exact Targets

- `.just/lint_sc_boundary.py`
- `.just/lint_sc_portability.py`
- `.just/run_lint.py`
- `justfile`
- `.just/print_help.py`
- `.github/workflows/ci.yml`
- `docs/sc-lint-boundary.md`
- `docs/phase-P/sprint-P9.md`

## Deliverables

- `just lint boundary` uses only the published `sc-lint-boundary 0.2.0`
  binary path resolution with install guidance instead of a `../sc-lint`
  cargo fallback
- `just lint portability` is the user-facing portability gate and uses only the
  published `sc-lint-portability 0.2.0` binary path resolution
- CI has a dedicated `sc-lint` job that installs the published analyzers and
  runs both lint gates after `clippy`
- the current workspace is proved green under `just lint portability`

## Acceptance Criteria

- `cargo test --workspace` passes
- `cargo clippy --all-targets --all-features -- -D warnings` passes
- `cargo fmt --check --all` passes
- `just lint boundary` passes without any `../sc-lint` fallback path
- `just lint portability` passes with zero findings or recorded accepted gaps
- `.github/workflows/ci.yml` contains a valid `sc-lint` job that installs
  `sc-lint-boundary@0.2.0` and `sc-lint-portability@0.2.0`
- `git diff --check` is clean

## Out Of Scope

- new runtime hook surfaces
- portability rule changes inside the `sc-lint` repo itself
- reopening already-closed `Phase P` runtime parity work

## Required Validation

- `cargo fmt --check --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `just lint boundary`
- `just lint portability`
- `git diff --check`

## Sprint QA Checklist

- Which requirement IDs or gap IDs changed status?
- What code was removed early rather than left in parallel?
- Which files/crates were the owned write scope for the sprint?
- What validation commands and direct tests proved the new contract?
- What follow-on work is blocked or unblocked by this sprint?

## Sprint QA Checklist Answers

- Requirement and gap status changes:
  - none; `P.9` hardens the published lint/tooling path and CI gate without
    changing requirement or implementation-gap status.
- Code removed early:
  - the repo-local `../sc-lint` cargo fallback was removed from the two public
    lint wrappers instead of leaving a parallel local-source execution path in
    place.
- Owned write scope:
  - `.just/lint_sc_boundary.py`
  - `.just/lint_sc_portability.py`
  - `.just/run_lint.py`
  - `justfile`
  - `.just/print_help.py`
  - `.github/workflows/ci.yml`
  - `docs/sc-lint-boundary.md`
  - `docs/phase-P/sprint-P9.md`
- Validation that passed:
  - `cargo fmt --check --all`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test --workspace`
  - `just lint boundary`
  - `just lint portability`
  - `git diff --check`
- Follow-on status:
  - `P.9` removes the last local-source analyzer fallback from the public
    `just lint` surface and establishes the published `sc-lint` analyzer gate
    in CI, which unblocks a release-ready `integrate/phase-P` closeout without
    requiring a checkout of `../sc-lint`.
