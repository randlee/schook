# Codex Hook Harness

This directory owns the Codex provider hook harness for `schook`: capture
hooks, approved fixtures, provider models, debounce behavior, pytest tests,
and raw capture evidence.

## Status

- live-tested 2026-05-22 against real Codex sessions
- 11 pytest tests passing
- direct `SessionStart`, `PreToolUse`, and `notify` fixtures approved
- `Stop`, `resume`, and `fork` dispositioned from local harness evidence

## Key Finding: Stop Does Not Fire

`Stop` did not fire in live Codex exec. `notify` (`agent-turn-complete`) is
the verified turn-complete surface. See `docs/hook-api/codex-hook-api.md` for
the full verified payload and environment contract.

## Directory Layout

```
test-harness/hooks/codex/
  hooks/
    notify.py         — notify hook: schedules debounce + writes notify captures
    pre_tool_use.py   — PreToolUse hook: cancels pending timer, restores active
    session_start.py  — direct SessionStart capture wrapper
    stop.py           — direct Stop capture wrapper for exercisability probes
  scripts/
    fire_pending.py   — processes due pending records, fires CLI command, flips to idle
    record_invocation.py — harmless test CLI target; records a fired invocation
  tests/
    conftest.py
    test_debounce_hooks.py
    test_fixture_validation.py
  captures/
    raw/              — timestamped payload + env JSON files from live/test runs
  fixtures/
    approved/         — approved payload/env fixtures plus manifest
  models/             — provider-specific validation models
  prompts/            — Codex prompts (reserved)
  reports/            — test reports (reserved)
  schema/             — JSON schema (reserved)
```

## Debounce Prototype

The hooks implement a debounce pattern that delays a downstream CLI call until
a Codex agent has been idle for a configurable window:

- `hooks/notify.py` — on `agent-turn-complete`: write a pending record for the
  correlation key (`thread-id`) and, for ATM projects, apply any per-agent
  idle timeout from `.atm.toml`
- `hooks/pre_tool_use.py` — on `PreToolUse`: cancel any pending record for the
  key and switch the visible marker to `active`
- `scripts/fire_pending.py` — run by an external timer or cron: invoke the
  configured CLI command for each due pending record, send the optional ATM
  idle notice, then flip `active` to `idle`

State lives under `SCHOOK_CODEX_HOOK_STATE_ROOT`. Visible session markers
are written under `<project-root>/.sc/sessions/codex/`:

| File | Meaning |
|------|---------|
| `active-<ATM_IDENTITY>.json` | tool activity observed; agent is active |
| `idle-<ATM_IDENTITY>.json` | startup or debounce expiry confirmed idle state |

The startup hook writes `idle-<ATM_IDENTITY>.json` immediately at session
start. The first `PreToolUse` switches that file to `active-<ATM_IDENTITY>.json`.
When the debounce expires, the file switches back to `idle` and includes
`idle_since` in the JSON payload.

For ATM-managed projects, the hook prefers the existing session record at
`.sc/sessions/codex/*-<session-id>.json`, then uses that record's stable `cwd`
to resolve the project root. This avoids drift after later `cd` inside tools.

Idle ATM notify config lives in `.atm.toml`:

```toml
[atm.idle_notify]
recipient = "team-lead"

[atm.idle_notify.agent.chook]
seconds = 60
```

When enabled and `ATM_TEAM` is set, the due timer sends:

```bash
atm send team-lead "$ATM_IDENTITY idle for ${seconds} seconds @ ${timestamp}" --team "$ATM_TEAM" --from "$ATM_IDENTITY"
```

## Running Tests

```bash
pytest test-harness/hooks/codex/tests/ -m provider_codex -v
```

## Live Config (chook — schook project)

The live Codex session for `chook` on the `schook` project uses:

- `~/.codex/config.toml` notify → `~/.codex/scripts/schook-delay-notify.py`
- `~/.codex/hooks.json` PreToolUse → `~/.codex/scripts/schook-delay-pretooluse.py`
- `~/.codex/hooks.json` SessionStart → `~/.codex/scripts/session-start.py`

Session state writes to `$REPO_ROOT/.sc/sessions/codex/`.

Rollback:

```bash
cp ~/.codex/config.toml.schook-delay-idle-hook-testing.bak ~/.codex/config.toml
cp ~/.codex/hooks.json.schook-delay-idle-hook-testing.bak ~/.codex/hooks.json
```
