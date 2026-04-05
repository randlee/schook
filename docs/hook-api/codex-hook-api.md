# Codex Hook API

## Purpose

This document records the currently verified Codex-facing hook surfaces that
matter to `sc-hooks` planning. It is intentionally separate from the Claude
document because the execution model is materially different.

## Platform Rules

- Codex `ai_cli` supports agent frontmatter hooks
- Codex runs frontmatter `PreToolUse` command hooks before agent execution
- hook exit `0` allows execution
- hook exit `2` blocks execution
- Codex resolves agent files directly or via `.claude/agents/` lookup instead
  of assuming Claude built-ins

This is different from Claude Code, where `settings.json` hooks are the stable
surface and frontmatter hooks should not be relied on.

## Path And Environment Rules

- `CODEX_PROJECT_DIR` is the intended project-root anchor when available
- frontmatter hooks should still avoid relative paths for the same reason as
  Claude hooks: current working directory is not a stable execution anchor
- any Codex hook plan should treat path resolution as an explicit part of the
  contract instead of leaving it to shell cwd behavior

## Current Verified Codex Relay

Current Codex relay/event evidence is split across:

- `atm-hook-relay.py` for the notify-side JSONL append behavior
- `agent-team-mail` `hook_watcher.rs` for the Rust-side event model consumed by
  ATM daemon components

Current verified event types:

| Event type | Current source | Current meaning |
| --- | --- | --- |
| `agent-turn-complete` | `atm-hook-relay.py` + `hook_watcher.rs` | turn-complete / idle availability signal |
| `session-start` | `hook_watcher.rs` | lifecycle start event carrying session/process identity |
| `session-end` | `hook_watcher.rs` | lifecycle end event carrying session/process identity |

## Current Verified HookEvent Fields

The current Rust-side `HookEvent` model in
`agent-team-mail/crates/atm-daemon/src/plugins/worker_adapter/hook_watcher.rs`
contains these fields:

| Rust field | JSON key | Presence |
| --- | --- | --- |
| `event_type` | `type` | all event types |
| `agent` | `agent` | all event types when routing identity is available |
| `team` | `team` | all event types when routing identity is available |
| `thread_id` | `thread-id` | Codex/internal relay events with thread context |
| `turn_id` | `turn-id` | `agent-turn-complete` events |
| `received_at` | `received-at` | relay events when the relay adds a receipt timestamp |
| `state` | `state` | availability-signaling events such as `agent-turn-complete` |
| `timestamp` | `timestamp` | availability-signaling events |
| `idempotency_key` | `idempotency-key` | availability dedup events |
| `session_id` | `sessionId` | session lifecycle events (`session-start`, `session-end`) |
| `process_id` | `processId` | session lifecycle events that carry process identity; currently required on `session-start` and treated as part of the lifecycle event set |

## Session Correlation Model

Codex currently does not have the same verified SessionStart capture path that
Claude uses in this repo. Until that exists, treat Codex identity as a planned
gap rather than pretending it matches Claude.

Current practical correlation inputs:

1. explicit `session_id` if the runner injects one in the future
2. `thread-id` or equivalent turn/thread metadata for relay correlation
3. `ATM_TEAM` + `ATM_IDENTITY` as routing labels

Design rule:

- do not claim Claude-equivalent session continuity for Codex until there is a
  verified hook or runner surface that emits a stable session identifier

## Design Implications For `sc-hooks`

- Codex should be documented as a separate compatibility target, not squeezed
  into the Claude hook assumptions
- frontmatter support makes Codex a better target for agent-local guard hooks
  than Claude, but the session-identity story is currently weaker
- the current Codex evidence includes both turn-complete and session lifecycle
  relay handling, so planning must use the verified event model rather than the
  narrower turn-complete script alone

## Current Platform Gaps

- no standalone Codex hook stdin schema is documented in this repo yet
- no verified upstream schema for all Codex hook payloads is captured here yet
- any future Codex planning should cite the runner or bundle source used to
  verify payload fields before those fields are promoted into `sc-hooks` docs
