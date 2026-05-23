# Gemini Hook Harness

This directory owns the Gemini provider hook harness for `schook`: capture
hooks, approved fixtures, provider models, and pytest schema-proof tests.

## Status

- live-tested 2026-05-22 against local `gemini-cli 0.42.0`
- six hook surfaces captured directly from Gemini hook stdin
- headless `--resume latest` confirmed through `SessionStart.source = "resume"`
- hook payload/env schema stable across `text`, `json`, and `stream-json`

## Verified Hook Surfaces

- `SessionStart`
- `SessionEnd`
- `BeforeAgent`
- `BeforeTool`
- `AfterTool`
- `AfterAgent`

## Key Findings

- Gemini hook configuration is active through `~/.gemini/settings.json`
- `gemini hooks --help` currently exposes only `migrate`, so CLI subcommands do
  not reflect the full runtime hook surface
- isolated user-scope settings under a temporary `HOME` are the cleanest
  capture path because they avoid leaking the operator's normal global hooks
- `SessionStart` distinguishes `startup` vs `resume`
- observed hook payload/env fields do not change across `text`, `json`, and
  `stream-json` output modes

## Directory Layout

```
test-harness/hooks/gemini/
  hooks/
    session_start.py
    session_end.py
    before_agent.py
    before_tool.py
    after_tool.py
    after_agent.py
  scripts/
    run-capture.sh
  tests/
    conftest.py
    test_harness_structure.py
    test_payload_models.py
    test_fixture_validation.py
  fixtures/
    approved/
      manifest.json
      *.json
      *.env.json
  captures/
    raw/
  models/
  prompts/
  reports/
  schema/
```

## Capture Strategy

The harness uses an isolated Gemini user home for reproducible local runs:

1. copy auth files from the operator's `~/.gemini/`
2. write a temporary `settings.json` containing only harness hook commands
3. run Gemini from the target repo with `HOME=<scratch-home>`
4. capture raw stdin payload plus a redacted env snapshot for each hook

This avoids interference from the operator's normal global Gemini hooks while
still using the locally authenticated CLI.

## Running Tests

```bash
pytest test-harness/hooks/gemini/tests/ -m provider_gemini -v
```

## Re-running Local Capture

```bash
test-harness/hooks/gemini/scripts/run-capture.sh lifecycle
test-harness/hooks/gemini/scripts/run-capture.sh before-agent
test-harness/hooks/gemini/scripts/run-capture.sh tool
test-harness/hooks/gemini/scripts/run-capture.sh resume
test-harness/hooks/gemini/scripts/run-capture.sh format text
test-harness/hooks/gemini/scripts/run-capture.sh format json
test-harness/hooks/gemini/scripts/run-capture.sh format stream-json
```
