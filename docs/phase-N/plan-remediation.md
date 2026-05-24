# Phase N Provider Harness Draft

## Goal

Harden the permanent provider harness around the deliverables this repo
actually owns: provider hook API docs, provider Pydantic models, approved
fixtures, validation tests, and stable `just` entrypoints.

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

Planning rule:

- these sprints are verify/fix sprints, not blank-slate build sprints
- each sprint closes one provider-facing deliverable set and nothing broader
- if current artifacts already prove cleanly, the sprint may close with audit
  evidence and no structural code changes
- expected QA findings should therefore be narrow:
  - actual provider drift
  - doc/model/fixture mismatch
  - missed hook-surface coverage

## Draft Sprint Sequence

### `N.5` Claude Harness And Baseline Verification

- verify the existing Claude deliverable set:
  - hook API doc
  - Pydantic models
  - approved fixtures/tests
- refresh only proven drift
- freeze the shared external harness contract the later provider sprints must
  match

Planned branch:

- `feature/pN-s5-claude-harness-verify`

### `N.6` Codex Harness Verification

- verify the existing Codex harness deliverable set:
  - capture path
  - approved fixtures
  - fixture-validation tests
- tighten only proven drift or external mismatch with the Claude baseline

Planned branch:

- `feature/pN-s6-codex-harness-verify`

### `N.7` Gemini Harness Verification

- verify the existing Gemini harness deliverable set:
  - capture path
  - approved fixtures
  - fixture-validation tests
- tighten only proven drift or external mismatch with the Claude baseline

Planned branch:

- `feature/pN-s7-gemini-harness-verify`

### `N.8` Codex API Document And Models Verification

- verify `docs/hook-api/codex-hook-api.md`, Codex Pydantic models, and their
  tests against the approved Codex fixtures
- tighten only proven drift or structural mismatch with the shared provider
  pattern

Planned branch:

- `feature/pN-s8-codex-api-models-verify`

### `N.9` Gemini API Document And Models Verification

- verify `docs/hook-api/gemini-hook-api.md`, Gemini Pydantic models, and their
  tests against the approved Gemini fixtures
- tighten only proven drift or structural mismatch with the shared provider
  pattern

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

## Shared Harness Contract

The harness ends with three provider-specific trees, but from the outside they
must present one shared contract:

- same directory conventions
- same approved/raw fixture split
- same test invocation shape
- same report/output conventions

Contract sample:

```text
test-harness/hooks/<provider>/
  fixtures/approved/
  captures/raw/
  tests/

test_harness/hooks/<provider>/models/payloads.py
```

Implementation rule:

- copy/adapt the Claude harness layout first
- reuse shared helpers and fixture-validation patterns where possible
- only add provider-local code when the schema or CLI behavior actually differs

## Non-Goals

- runtime hook implementation work
- findings-only cleanup outside the provider deliverables listed above
