# Gemini Smoke Record

Accepted baseline:

- branch: `feature/pQ-s5-gemini-smoke`
- smoke contract owner: `Q.5`
- command: `just smoke gemini live`
- timestamp (UTC): `2026-05-26T01:21:53Z`
- repo root: `<repo-root>`
- runtime root: `<runtime-root>`

Observed install proof:

- `~/.gemini/settings.json` wires:
  - `SessionStart` to the installed `sc-hooks` runtime under
    `$HOME/.local/bin/sc-hooks`
  - `BeforeAgent` to the installed `sc-hooks` runtime under
    `$HOME/.local/bin/sc-hooks`
  - `SessionEnd` to the installed `sc-hooks` runtime under
    `$HOME/.local/bin/sc-hooks`

Observed live path:

- probe prompt: `Reply with OK only.`
- Gemini returned stdout: `OK`
- one new canonical runtime state record was written under
  `$HOME/.sc-hooks/state/`
- the terminal Gemini state record ended with:
  - `provider = "gemini"`
  - `ai_root_dir = "<repo-root>"`
  - `agent_state = "ended"`
  - `last_hook_event = "SessionEnd"`

Observed observability proof:

- log file: `$HOME/.local/share/sc-hooks/runtime-layout/.sc-hooks/observability/logs/sc-hooks.log.jsonl`
- fresh `dispatch.complete` entries were observed for:
  - `SessionStart`
  - `PreToolUse(Agent)`
  - `SessionEnd`

Smoke verdict:

- install path: PASS
- retained runtime dispatch path: PASS
- retained lifecycle path: PASS
- observability path: PASS

Deferred beyond `Q.5`:

- Claude smoke (`Q.3`) remains separate
- Codex smoke (`Q.4`) remains separate
- new Gemini runtime-surface expansion
- Gemini harness/doc-model expansion
