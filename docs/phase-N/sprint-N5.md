---
id: N.5
title: Claude Harness And Baseline Verification
status: completed
branch: feature/pN-s5-claude-harness-verify
worktree: ../schook-worktrees/feature/pN-s5-claude-harness-verify
target: integrate/phase-N
---

# Sprint N.5 — Claude Harness And Baseline Verification

## Goal

- leave the Claude provider deliverables current against local/global Claude
  evidence
- freeze one shared external harness contract that Codex and Gemini must match

## Hard Dependencies

- current `origin/integrate/phase-N` branch head
- local Claude hooks remain available on this machine

## Exact Targets

- `docs/hook-api/claude-hook-api.md`
- `test-harness/hooks/claude/`
- `test_harness/hooks/claude/`
- `test-harness/hooks/README.md`

## Deliverables

- `docs/hook-api/claude-hook-api.md`
- `test_harness/hooks/claude/models/payloads.py`
- `test-harness/hooks/claude/fixtures/approved/`
- `test-harness/hooks/claude/tests/`
- `test-harness/hooks/README.md`

## Production-Ready Expectation

Every listed deliverable is expected to land at a production-ready level for
the verification scope this sprint claims.

## Contract Sample

```text
test-harness/hooks/claude/fixtures/approved/
test-harness/hooks/claude/tests/
test_harness/hooks/claude/models/payloads.py
docs/hook-api/claude-hook-api.md
```

## Acceptance Criteria

- `docs/hook-api/claude-hook-api.md` matches the approved Claude fixtures
- `test_harness/hooks/claude/models/payloads.py` validates the approved Claude
  fixtures
- `test-harness/hooks/claude/fixtures/approved/` is the approved Claude
  fixture set used by the tests
- `test-harness/hooks/claude/tests/` validates the approved Claude fixtures
- `test-harness/hooks/README.md` states the shared external harness contract
  later provider sprints must match

## Out Of Scope

- Codex deliverables
- Gemini deliverables
- `just` integration

## Required Validation

- `pytest test-harness/hooks/claude/tests/ -q`
- `pytest test-harness/hooks/ -q`
- `git diff --check`
