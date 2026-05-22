# Codex Hook Harness

Codex is documented as a provider reference in the current planning set, but it
is deferred from the first harness pass.

This directory exists now so the long-term harness remains organized by
provider, even before Codex capture work starts.

Current status:

- documented
- deferred from first harness implementation
- no verified provider schema is captured yet
- harness-only debounce prototype added for `agent-turn-complete` plus
  `PreToolUse` cancellation testing; this is not a promoted Codex contract

When Codex work starts later, this directory should own:

- Codex prompts
- Codex capture hooks or relay capture scripts
- Codex models and schema
- Codex fixtures
- Codex reports
- Codex `pytest` tests

## Debounce Prototype

The current Codex harness includes an experimental Python debounce prototype:

- `hooks/stop.py`
  - schedules delayed work for a correlation key such as `thread-id`
- `hooks/pre_tool_use.py`
  - cancels pending delayed work when activity resumes
- `scripts/fire_pending.py`
  - processes due records and invokes the configured CLI command
- `scripts/record_invocation.py`
  - harmless test CLI target that records a fired debounce invocation

Prototype rules:

- state is stored under `SCHOOK_CODEX_HOOK_STATE_ROOT`
- delayed work duration comes from `SCHOOK_CODEX_DEBOUNCE_SECONDS`
- the optional CLI tool comes from `SCHOOK_CODEX_DEBOUNCE_COMMAND`
- captures go to `SCHOOK_HOOK_CAPTURE_ROOT` when set
- hooks may be gated to one project root with `SCHOOK_CODEX_HOOK_PROJECT_ROOT`
- active or idle marker files are written under
  `<project-root>/.sc/sessions/codex/active-<ATM_IDENTITY>.json` and
  `<project-root>/.sc/sessions/codex/idle-<ATM_IDENTITY>.json`
- project root is resolved from the hook payload `cwd` by running
  `git -C <cwd> rev-parse --show-toplevel`, then falling back to raw `cwd`

This prototype exists to test the debounce model discussed in planning:

- treat Codex turn-complete as the idle/start-debounce signal
- treat `PreToolUse` as sufficient resumed-activity cancellation
- keep the actual timer firing outside the hook itself
- expose a simple visible active or idle state transition without needing to
  inspect delayed CLI output
