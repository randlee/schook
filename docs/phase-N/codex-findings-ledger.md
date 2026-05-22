# Codex Findings Ledger

Status:
- `FROZEN`

Purpose:
- authoritative findings ledger for `N.1` Codex schema capture

| id | surface | finding | severity | disposition | notes |
| --- | --- | --- | --- | --- | --- |
| `CDX-N1-001` | `SessionStart` | Direct `SessionStart` raw stdin and env were captured in repo-owned fixtures; the earlier planning-only relay classification was too weak. | medium | `captured` | Approved fixtures: `session-start-startup*.json` and `session-start-cwd-drift*.json`. |
| `CDX-N1-002` | `notify` | `notify` is the real turn-complete surface; payload shape is `type=agent-turn-complete` with `thread-id` and `turn-id`. | medium | `captured` | Approved fixtures: `notify-agent-turn-complete*.json` and `notify-agent-turn-complete-cwd-drift*.json`. |
| `CDX-N1-003` | `PreToolUse` | Direct Bash-tool `PreToolUse` payloads were captured with stable `session_id`, `turn_id`, and `tool_use_id`. | medium | `captured` | Approved fixtures: `pretooluse-bash*.json` and `pretooluse-bash-cwd-drift*.json`. |
| `CDX-N1-004` | `--cd` | Startup-directory drift is directly observable: `cwd` in `SessionStart`, `PreToolUse`, and `notify` follows the `-C/--cd` launch root. | medium | `captured` | Use the `*-cwd-drift*.json` fixtures as the authoritative scenario proof. |
| `CDX-N1-005` | `Stop` | A direct `Stop` command hook was configured locally on 2026-05-22 and did not fire during `codex exec`; stderr showed `SessionStart` and `PreToolUse` only. | medium | `confirmed-not-exercisable` | Keep `Stop` out of approved runtime assumptions. |
| `CDX-N1-006` | `resume` | `codex resume <session-id> <prompt>` exits with `Error: stdin is not a terminal` under noninteractive harness execution. | medium | `confirmed-not-exercisable` | Treat resume continuity as a later interactive/manual capture topic, not a current automated harness surface. |
| `CDX-N1-007` | `fork` | `codex fork <session-id> <prompt>` exits with `Error: stdin is not a terminal` under noninteractive harness execution. | medium | `confirmed-not-exercisable` | `fork` remains documented, but not currently harness-capturable in this sprint’s automated flow. |
| `CDX-N1-008` | `env/session correlation` | `CODEX_THREAD_ID` in the hook environment remained a stale outer-session value and did not match the captured `session_id` / `thread-id` for the new `codex exec` sessions. | high | `provider-local only` | Do not normalize `CODEX_THREAD_ID` into canonical session identity in `N.3` without further provider evidence. |
| `CDX-N1-009` | `newtype candidates` | `session_id`, `thread-id`, `turn-id`, and `tool_use_id` behave as semantic identifiers and should not be treated as display-only strings in later Rust work. | low | `carry to N.3` | `transcript_path` is a provider-local path value, not a canonical identifier. |
