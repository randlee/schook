---
id: Q.7
title: Cursor Agent API Doc And Pydantic Models
status: planned
branch: feature/pQ-s7-cursor-doc-models
worktree: ../schook-worktrees/feature/pQ-s7-cursor-doc-models
target: integrate/phase-Q
---

# Sprint Q.7 — Cursor Agent API Doc And Pydantic Models

## Purpose

- promote the Cursor Agent API document from planning reference to
  harness-backed provider artifact
- add or tighten Pydantic models for retained Cursor surfaces

## Entry Criteria

- accepted `Q.6` Cursor harness output

## Exact Targets

- `docs/hook-api/cursor-agent-hook-api.md`
- Cursor provider models and tests

## Deliverables

- updated Cursor Agent API doc
- Pydantic payload models for retained Cursor surfaces

## Acceptance Criteria

- Cursor API doc matches the retained harness-backed provider surfaces
- provider models and tests cover the retained surfaces explicitly

## Required Validation

- `cargo test --workspace`
- `git diff --check`
