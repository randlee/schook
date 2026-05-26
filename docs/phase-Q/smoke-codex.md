# Codex Smoke Record

Accepted baseline:

- branch: `feature/pQ-s4-codex-smoke`
- smoke contract owner: `Q.4`
- command: `just smoke codex live`
- timestamp (UTC): `2026-05-26T01:16:12Z`
- repo root: `/Users/randlee/Documents/github/schook-worktrees/feature/pQ-s4-codex-smoke`
- runtime root: `/Users/randlee/.local/share/sc-hooks/runtime-layout`

Observed install proof:

- `~/.codex/config.toml` wires `notify` to
  `~/.codex/scripts/schook-delay-notify.py`
- `~/.codex/hooks.json` wires:
  - `SessionStart` to the installed `sc-hooks` runtime under
    `~/.local/bin/sc-hooks`
  - `PreToolUse` to `~/.codex/scripts/schook-delay-pretooluse.sh`

Observed live path:

- probe prompt: `Run pwd using Bash exactly once, then reply with OK only.`
- Codex wrote the final response `OK`
- Codex stderr included the explicit tool transcript:
  - `exec`
  - `/bin/bash -c pwd in /Users/randlee/Documents/github/schook-worktrees/feature/pQ-s4-codex-smoke`
  - `/Users/randlee/Documents/github/schook-worktrees/feature/pQ-s4-codex-smoke`
- the smoke runner uses:
  - `ATM_IDENTITY=codex-smoke`
  - `ATM_TEAM=schook`
  - `SCHOOK_CODEX_IDLE_SECONDS=1`
  - an isolated temporary `SCHOOK_CODEX_IDLE_STATE_ROOT`
- one repo-local lifecycle marker was produced at:
  - `.sc/sessions/codex/idle-codex-smoke.json`
- the terminal Codex idle marker ended with:
  - `state = "idle"`
  - `project_dir = "/Users/randlee/Documents/github/schook-worktrees/feature/pQ-s4-codex-smoke"`
  - `cwd = "/Users/randlee/Documents/github/schook-worktrees/feature/pQ-s4-codex-smoke"`
  - non-empty `session_id`

Observed observability proof:

- log file: `~/.local/share/sc-hooks/runtime-layout/.sc-hooks/observability/logs/sc-hooks.log.jsonl`
- accepted-baseline observability proof included:
  - a recent `dispatch.complete` entry for `SessionStart`
  - fresh `dispatch.complete` entries for `PreToolUse`

Smoke verdict:

- install path: PASS
- retained Bash runtime dispatch path: PASS
- retained notify/idle lifecycle path: PASS
- observability path: PASS

Deferred beyond `Q.4`:

- Claude smoke (`Q.3`) remains separate
- Gemini smoke (`Q.5`)
- new Codex runtime-surface expansion
- Codex harness/doc-model expansion
