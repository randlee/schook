---
id: Q.8
title: opencode Hook Harness
status: planned
branch: feature/pQ-s8-opencode-harness
worktree: ../schook-worktrees/feature/pQ-s8-opencode-harness
target: integrate/phase-Q
---

# Sprint Q.8 — opencode Hook Harness

## Purpose

- add `opencode` as a maintained harness provider with approved fixtures,
  provider models, and harness tests
- land the missing control-doc ownership for opencode provider scope

## Entry Criteria

- accepted `Q.2` smoke infrastructure

## Exact Targets

- `test-harness/hooks/opencode/`
- `test_harness/hooks/opencode/`
- `docs/requirements.md`
- `docs/traceability.md`

## Deliverables

- approved opencode fixtures
- opencode provider models
- opencode harness tests
- control-doc updates that introduce and record the opencode provider scope

## Acceptance Criteria

- opencode is represented as a maintained harness provider
- the phase does not rely on undocumented implied opencode scope

## Required Validation

- `cargo test --workspace`
- `git diff --check`
