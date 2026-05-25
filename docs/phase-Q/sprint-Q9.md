---
id: Q.9
title: opencode API Doc And Pydantic Models
status: planned
branch: feature/pQ-s9-opencode-doc-models
worktree: ../schook-worktrees/feature/pQ-s9-opencode-doc-models
target: integrate/phase-Q
---

# Sprint Q.9 — opencode API Doc And Pydantic Models

## Purpose

- create the opencode provider API document
- add or tighten Pydantic models for retained opencode surfaces

## Entry Criteria

- accepted `Q.8` opencode harness output

## Exact Targets

- `docs/hook-api/opencode-agent-hook-api.md`
- opencode provider models and tests

## Deliverables

- opencode API doc
- Pydantic payload models for retained opencode surfaces

## Acceptance Criteria

- opencode API doc matches the retained harness-backed provider surfaces
- provider models and tests cover the retained surfaces explicitly

## Required Validation

- `cargo test --workspace`
- `git diff --check`
