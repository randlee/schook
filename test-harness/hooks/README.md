# Hook Harness

## Purpose

This directory is the source of truth for the hook schema harness.

`docs/plugin-plan-s9.md` is the umbrella execution plan.

This directory is the harness sub-specification referenced by that plan.

It owns:

- harness documentation
- provider-specific capture plans
- provider-specific prompts
- local capture-hook scripts
- validation models
- generated schema artifacts
- raw captured payloads
- approved fixture snapshots
- formal run reports
- `pytest` test entry points

This harness exists to prevent guessed hook contracts from leaking into
implementation. No hook runtime code should depend on a field unless that field
has been promoted from captured evidence into a validated provider model.

## Core Rules

1. `pytest` is the harness test runner.
2. Raw payload capture comes before schema promotion.
3. Claude is the first implementation provider.
4. ATM-specific behavior stays documented separately in
   `docs/hook-api/atm-hook-extension.md`.
5. Codex, Gemini, and Cursor may be documented here before implementation, but
   they do not block the first Claude implementation path.
6. No manual copying or hand-moving of capture artifacts is allowed.
7. Every live run writes a formal output directory and a formal report.
8. Approved fixture snapshots are long-lived contract evidence.

## Directory Contract

```text
test-harness/hooks/
  README.md
  pytest.ini

  common/
    README.md

  claude/
    README.md
  codex/
    README.md
  gemini/
    README.md
  cursor-agent/
    README.md
```

Each provider directory owns the same conceptual substructure, whether all
subdirectories are created immediately or added as implementation begins:

```text
<provider>/
  prompts/
  hooks/
  models/
  schema/
  fixtures/
  captures/
  reports/
  scripts/
  tests/
```

Meaning of each subdirectory:

- `prompts/`: canned prompts used for repeatable live capture
- `hooks/`: local Python hooks used only for harness capture
- `models/`: provider-specific validation models
- `schema/`: generated schema artifacts derived from models
- `fixtures/`: approved fixture snapshots stored under version control
- `captures/`: raw run output from live capture runs
- `reports/`: machine-readable and human-readable run reports
- `scripts/`: harness runner scripts and report generation helpers
- `tests/`: `pytest` tests for the provider

## Execution Model

The harness has two test layers.

### 1. Fixture Validation Layer

This layer is fast and should be suitable for normal CI.

It validates:

- approved fixtures against provider models
- generated schema against fixture content
- drift classification logic
- report generation
- redaction behavior

This layer does not launch external AI runtimes.

### 2. Live Capture Layer

This layer launches the real provider with canned prompts and local capture
hooks.

It performs:

- raw payload capture
- model validation against captured payloads
- schema generation/update checks
- formal report generation

This layer is expected to be slower and may be gated separately from the fast
fixture-validation suite.

## `pytest` Contract

`pytest` is the required harness runner.

Expected split:

- default `pytest` run:
  - fixture/model/schema tests only
- `pytest -m live_capture`:
  - live provider launches and capture tests

Recommended markers:

- `live_capture`
- `provider_claude`
- `provider_codex`
- `provider_gemini`
- `provider_cursor_agent`

The harness should remain runnable long term without requiring manual edits to
test code or captured output locations.

## Capture And Report Lifecycle

Every live run must produce a formal run directory.

Recommended shape:

```text
<provider>/captures/<run-id>/
  raw/
  normalized/

<provider>/reports/<run-id>/
  validation-summary.json
  drift-report.json
  run-report.md
```

Rules:

- `raw/` stores exact captured payloads before normalization
- `normalized/` may contain sorted/sanitized views used for comparison
- `validation-summary.json` records pass/fail by hook payload
- `drift-report.json` records added, removed, and changed fields
- `run-report.md` is the human-readable QA/review artifact
- the harness itself chooses the output paths; no manual moving of files is
  part of the workflow

## Fixture Policy

There are three artifact classes:

1. raw captures
2. approved fixtures
3. generated reports

Policy:

- raw captures are evidence from a specific run
- approved fixtures are curated long-term contract snapshots
- generated reports summarize what happened in the run and whether the current
  models still match

Approved fixtures should only be updated through harness scripts and review, not
by hand-editing them into shape.

## Model And Schema Policy

Provider models should begin minimally:

- strict on known required fields
- permissive on unknown extra fields during early discovery
- explicit about which fields implementation is allowed to rely on

Schema artifacts should be generated from the provider models whenever possible.

The harness must distinguish between:

- observed payload shape
- validated current schema
- implementation-approved fields

Those three states must not be conflated.

## Drift Policy

Drift classes:

- required field removed: fail
- required field type changed: fail
- field added: report for review
- optional field removed: report for review unless implementation currently
  relies on it

The harness exists to make provider drift visible immediately after an upgrade.

## First Development Pass

The first development pass is Claude-only.

Required Claude capture set:

- `SessionStart`
- `SessionEnd`
- `PreToolUse(Bash)`
- `PostToolUse(Bash)`
- `PreToolUse(Agent)`
- `PermissionRequest`
- `Stop`

The sequence is:

1. add the Claude harness pieces in `test-harness/hooks/claude/`
2. capture Claude payloads
3. validate and report on those payloads
4. revise the hook plan from captured evidence
5. implement Claude hook crates

Deferred from the first development pass:

- Codex capture and implementation
- Gemini capture and implementation
- Cursor capture and implementation

## Review Gate

This harness documentation is ready for review when:

- directory ownership is clear
- `pytest` is the explicit runner
- canned prompts are required for live capture
- local Python hooks are the capture mechanism
- report generation is automatic
- fixture and drift policies are defined
- Claude-first scope is explicit

## Relationship To Main Docs

Main architecture and planning docs should reference this directory as the
harness source of truth.

Review rule:

- `docs/plugin-plan-s9.md` owns the execution sequence
- this README owns the harness operating rules
- provider hook API docs own platform-specific verified facts

Those docs should not duplicate the full harness contract here unless the
high-level release or architecture story requires it.
