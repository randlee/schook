# Codex Hook Harness

This directory owns the Codex provider hook harness for `schook`: capture
hooks, debounce prototype, pytest tests, and raw captures.

## Status

- live-tested 2026-05-22 against real Codex sessions
- 5 pytest tests passing
- debounce contract verified under live cancel conditions

## Key Finding: Stop Does Not Fire

`Stop` did not fire in live Codex exec. `notify` (`agent-turn-complete`) is
the verified turn-complete surface. See `docs/hook-api/codex-hook-api.md` for
the full verified payload and environment contract.

## Directory Layout

```
test-harness/hooks/codex/
  hooks/
    stop.py           — notify hook: schedules debounce + writes active marker
    pre_tool_use.py   — PreToolUse hook: cancels pending timer, restores active
  scripts/
    fire_pending.py   — processes due pending records, fires CLI command, flips to idle
    record_invocation.py — harmless test CLI target; records a fired invocation
  tests/
    conftest.py
    test_debounce_hooks.py
  captures/
    raw/              — timestamped payload + env JSON files from live/test runs
  fixtures/           — approved fixture snapshots (reserved for future promotion)
  models/             — schema models (reserved)
  prompts/            — Codex prompts (reserved)
  reports/            — test reports (reserved)
  schema/             — JSON schema (reserved)
```

## Debounce Prototype

The hooks implement a debounce pattern that delays a downstream CLI call until
a Codex agent has been idle for a configurable window:

- `hooks/stop.py` — on `agent-turn-complete`: write a pending record for the
  correlation key (`thread-id`), write an `active` marker under the project root
- `hooks/pre_tool_use.py` — on `PreToolUse`: cancel any pending record for the
  key, restore the `active` marker
- `scripts/fire_pending.py` — run by an external timer or cron: invoke the
  configured CLI command for each due pending record, flip `active` to `idle`

State lives under `SCHOOK_CODEX_HOOK_STATE_ROOT`. Visible session markers
are written under `<project-root>/.sc/sessions/codex/`:

| File | Meaning |
|------|---------|
| `active-<ATM_IDENTITY>.json` | turn complete, timer pending |
| `idle-<ATM_IDENTITY>.json` | timer fired, agent confirmed idle |

## Running Tests

```bash
pytest test-harness/hooks/codex/tests/ -m provider_codex -v
```

## Live Config (chook — schook project)

The live Codex session for `chook` on the `schook` project uses:

- `~/.codex/config.toml` notify → `~/.codex/scripts/schook-delay-notify.py`
- `~/.codex/hooks.json` PreToolUse → `~/.codex/scripts/schook-delay-pretooluse.py`

Session state writes to `/Users/randlee/Documents/github/schook/.sc/sessions/codex/`.

Rollback:

```bash
cp ~/.codex/config.toml.schook-delay-idle-hook-testing.bak ~/.codex/config.toml
cp ~/.codex/hooks.json.schook-delay-idle-hook-testing.bak ~/.codex/hooks.json
```
