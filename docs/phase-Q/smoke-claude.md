# Claude Smoke Record

Accepted baseline:

- branch: `feature/pQ-s3-claude-smoke`
- smoke contract owner: `Q.3`
- command: `just smoke claude live`
- timestamp (UTC): `2026-05-26T01:08:22Z`
- repo root: `<repo-root>`
- runtime root: `<runtime-root>`

Observed install proof:

- `~/.claude/settings.json` contains `SessionStart`, `PreToolUse`,
  `PostToolUse`, and `SessionEnd` hook wiring to the installed
  `sc-hooks` runtime under `~/.local/bin/sc-hooks`

Observed live path:

- probe prompt: `Run pwd using Bash exactly once, then reply with OK only.`
- Claude returned stdout: `OK`
- one new session record was written under `~/.sc-hooks/state/`
- the terminal Claude session record ended with:
  - `provider = "claude"`
  - `ai_root_dir` matched the active `<repo-root>`
  - `agent_state = "ended"`
  - `last_hook_event = "SessionEnd"`

Observed observability proof:

- log file: `~/.local/share/sc-hooks/runtime-layout/.sc-hooks/observability/logs/sc-hooks.log.jsonl`
  (machine-local `SC_HOOKS_AUDIT_PATH` value; not committed to CI)
- observed `dispatch.complete` entries for:
  - `SessionStart`
  - `PreToolUse`
  - `PostToolUse(Bash)`
  - `SessionEnd`

Smoke verdict:

- install path: PASS
- hook dispatch path: PASS
- shared plugin-chain path: PASS
- observability path: PASS

Deferred beyond `Q.3`:

- Codex smoke (`Q.4`)
- Gemini smoke (`Q.5`)
- new Claude runtime-surface expansion
