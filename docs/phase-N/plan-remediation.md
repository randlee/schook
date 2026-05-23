# Phase N Provider Harness Draft

## Goal

Replace the old finding-driven remediation split with a provider-deliverable
plan for the permanent harness assets owned by this repo.

Required deliverables:

- `docs/hook-api/claude-hook-api.md`
- Claude Pydantic models
- Claude approved fixtures and validation tests
- `docs/hook-api/codex-hook-api.md`
- Codex approved fixtures and validation tests
- Codex Pydantic models
- `docs/hook-api/gemini-hook-api.md`
- Gemini approved fixtures and validation tests
- Gemini Pydantic models
- `just test hooks claude`
- `just test hooks codex`
- `just test hooks gemini`

## Current Audit Snapshot

- planning branch: `feature/phase-N-fix-planning`
- target integration branch for execution: `integrate/phase-N`
- Claude global hooks are already in routine use on this machine
- Codex global hooks are already in routine use on this machine
- current `origin/integrate/phase-N` already contains provider docs, provider
  models, approved fixtures, and pytest coverage for Claude, Codex, and Gemini
- shared harness structure already exists under `test-harness/hooks/` and
  `test_harness/hooks/`

Current baseline by provider:

- Claude:
  - `docs/hook-api/claude-hook-api.md`
  - `test_harness/hooks/claude/models/payloads.py`
  - approved fixtures under `test-harness/hooks/claude/fixtures/approved/`
  - tests under `test-harness/hooks/claude/tests/`
- Codex:
  - `docs/hook-api/codex-hook-api.md`
  - `test_harness/hooks/codex/models/payloads.py`
  - approved fixtures under `test-harness/hooks/codex/fixtures/approved/`
  - tests under `test-harness/hooks/codex/tests/`
- Gemini:
  - `docs/hook-api/gemini-hook-api.md`
  - `test_harness/hooks/gemini/models/payloads.py`
  - approved fixtures under `test-harness/hooks/gemini/fixtures/approved/`
  - tests under `test-harness/hooks/gemini/tests/`

Implication:

- these sprints are verify/fix sprints, not blank-slate build sprints
- expected QA findings should therefore be narrow:
  - actual provider drift
  - doc/model/fixture mismatch
  - missed hook-surface coverage

## Draft Sprint Sequence

### `N.5` Claude Harness And Baseline Verification

- verify the Claude baseline still matches current local/global hook behavior
- tighten only what has drifted
- confirm the shared external harness contract Claude establishes for Codex and
  Gemini

Planned branch:

- `feature/pN-s5-claude-harness-verify`

### `N.6` Codex Harness Verification

- verify Codex harness capture, fixtures, and tests against current local
  Codex behavior
- tighten only what is missing, drifted, or externally inconsistent with the
  Claude baseline

Planned branch:

- `feature/pN-s6-codex-harness-verify`

### `N.7` Gemini Harness Verification

- verify Gemini harness capture, fixtures, and tests against current local
  Gemini behavior
- tighten only what is missing, drifted, or externally inconsistent with the
  Claude baseline

Planned branch:

- `feature/pN-s7-gemini-harness-verify`

### `N.8` Codex API Document And Models Verification

- verify `docs/hook-api/codex-hook-api.md`, Codex Pydantic models, and their
  tests against the approved Codex fixtures
- tighten only what is missing, drifted, or structurally inconsistent with the
  shared provider pattern

Planned branch:

- `feature/pN-s8-codex-api-models-verify`

### `N.9` Gemini API Document And Models Verification

- verify `docs/hook-api/gemini-hook-api.md`, Gemini Pydantic models, and their
  tests against the approved Gemini fixtures
- tighten only what is missing, drifted, or structurally inconsistent with the
  shared provider pattern

Planned branch:

- `feature/pN-s9-gemini-api-models-verify`

### `N.10` Harness `just` Integration

- wire the provider harness into `just`
- expose one stable entrypoint per provider:
  - `just test hooks claude`
  - `just test hooks codex`
  - `just test hooks gemini`

Planned branch:

- `feature/pN-s10-harness-just-integration`

## Reuse Rule

The harness should end with three provider-specific trees, but creation should
reuse existing shared structure aggressively:

- copy/adapt the Claude harness layout first
- reuse shared helpers and fixture-validation patterns where possible
- only add provider-local code when the schema or CLI behavior actually differs
- keep the external harness contract uniform across providers:
  - same directory conventions
  - same approved/raw fixture split
  - same test invocation shape
  - same report/output conventions

## Non-Goals

- runtime hook implementation work
- findings-only cleanup that does not affect provider API docs, provider
  Pydantic models, provider fixtures/tests, or the final `just` integration
