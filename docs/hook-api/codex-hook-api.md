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
- `CODEX_THREAD_ID` carries the current thread UUID in the hook environment;
  it is the same value as `thread-id` in the payload
- frontmatter hooks should still avoid relative paths for the same reason as
  Claude hooks: current working directory is not a stable execution anchor
- project root should be resolved from the payload `cwd` field using
  `git -C <cwd> rev-parse --show-toplevel`, falling back to raw `cwd`; do not
  rely on shell cwd or `PWD` as the project anchor
- any Codex hook plan should treat path resolution as an explicit part of the
  contract instead of leaving it to shell cwd behavior

## Verified Hook Surfaces

### `notify` / `agent-turn-complete` — Turn-Complete Signal

**Verified 2026-05-22**: live Codex sessions fire `notify` reliably on turn
completion. This is the correct idle and debounce-start signal.

`Stop` did **not** fire in live Codex exec. Do not use `Stop` as the primary
turn-complete or idle-detection surface.

Verified payload fields:

| Field | Type | Notes |
|-------|------|-------|
| `type` | string | `"agent-turn-complete"` |
| `thread-id` | string | thread UUID; stable correlation key within a session |
| `cwd` | string | working directory at hook fire time |
| `state` | string | `"idle"` when present |

Verified environment fields:

| Variable | Notes |
|----------|-------|
| `CODEX_PROJECT_DIR` | project root path; preferred anchor over `cwd` |
| `CODEX_THREAD_ID` | same value as `thread-id` in payload |
| `ATM_IDENTITY` | agent identity when set in session environment |
| `ATM_TEAM` | team routing label when set in session environment |

### `PreToolUse` — Resumed-Activity Signal

**Verified 2026-05-22**: `PreToolUse` fires reliably before Codex tool
execution and is sufficient as a cancellation signal for pending debounce
timers.

Verified payload fields:

| Field | Type | Notes |
|-------|------|-------|
| `hook_event_name` | string | `"PreToolUse"` |
| `thread-id` | string | matches the notify correlation key |
| `cwd` | string | working directory at hook fire time |
| `session_id` | string | matched the turn-complete `thread-id` in live captures |
| `turn_id` | string | current turn UUID |

### `Stop` — Not Reliable

Live testing confirmed `Stop` did not fire in live Codex exec. Do not use
`Stop` as the primary idle-detection or turn-complete signal. Use `notify`
(`agent-turn-complete`) instead.

## Current Verified Codex Relay

Current Codex relay/event evidence is split across:

- `atm-hook-relay.py` for the notify-side JSONL append behavior
- `agent-team-mail` `hook_watcher.rs` for the Rust-side event model consumed by
  ATM daemon components

These relay-side fields are planning evidence only. `Phase N` approved fixture
inventory is limited to raw stdin payloads and hook-process environment fields
captured directly by `schook` harness scripts.

Current verified event types:

| Event type | Current source | Current meaning |
| --- | --- | --- |
| `agent-turn-complete` | `atm-hook-relay.py` + `hook_watcher.rs` | turn-complete / idle availability signal |
| `session-start` | `hook_watcher.rs` | lifecycle start event carrying session/process identity |
| `session-end` | `hook_watcher.rs` | lifecycle end event carrying session/process identity |

## Current Verified HookEvent Fields

Relay-only planning evidence — not part of approved `schook` fixture
inventory unless a future harness capture proves the same fields directly.

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

## Debounce Pattern — Verified Design

Live testing on 2026-05-22 verified the following debounce contract for
delaying work until a Codex agent is confirmed idle.

### Signal roles

| Hook | Role |
|------|------|
| `notify` (`agent-turn-complete`) | start debounce timer for `thread-id` |
| `PreToolUse` | cancel pending timer; agent has resumed |

### Timer lifecycle

```
notify fires
  → write pending record for thread-id due at (now + DEBOUNCE_SECONDS)
  → preserve current visible state until due time

PreToolUse fires before timer is due
  → delete pending record for thread-id
  → restore active-<ATM_IDENTITY>.json; remove idle marker if present

fire_pending.py runs (external cron or triggered subprocess)
  → for each due pending record: optionally invoke DEBOUNCE_COMMAND once
  → optionally send ATM idle notice if configured in .atm.toml
  → replace active marker with idle-<ATM_IDENTITY>.json and set idle_since
  → delete pending record (idempotent on repeat runs)
```

### Live cancel test evidence (2026-05-22)

```
02:03:35Z  notify (first stop)   → debounce timer started
02:03:51Z  PreToolUse            → timer cancelled; agent resumed
02:03:57Z  notify (second stop)  → new timer started
02:04:42Z  fire_pending.py due   → single CLI invocation fired (second stop only)
```

Two stops, one invocation — the debounce contract holds under real sessions.

## Session State Markers

The startup hook plus debounce implementation write state markers under the
project root so external tools can observe Codex idle state without inspecting
internal state files:

| File | Written when |
|------|-------------|
| `.sc/sessions/codex/idle-<ATM_IDENTITY>.json` | SessionStart fires, or fire_pending confirms debounce expiry |
| `.sc/sessions/codex/active-<ATM_IDENTITY>.json` | `PreToolUse` fires and activity resumes |

`idle` JSON includes the timestamp when the agent became idle (`idle_since`).
`active` is restored and `idle` removed when `PreToolUse` cancels a pending
timer or when first activity begins in a fresh session.

## Session Record As Source Of Truth

Codex SessionStart already writes a stable session record under:

- `.sc/sessions/codex/*-<session-id>.json`

That record currently includes:

- `native_session_id`
- `cwd`
- `project_dir`
- `transcript_path`

For post-start hooks, prefer this session record over live hook `cwd` when you
need the original session root. A safe lookup order is:

1. read `session_id` from `PreToolUse`, or `thread-id` from `notify`
2. locate `.sc/sessions/codex/*-<session-id>.json`
3. use the record's `cwd`
4. resolve the root with `git -C <recorded-cwd> rev-parse --show-toplevel`
5. fall back to the recorded `cwd` if Git resolution fails

Marker JSON structure:

```json
{
  "state": "active" | "idle",
  "project_dir": "/absolute/path/to/project",
  "thread_id": "uuid",
  "timestamp": "ISO-8601"
}
```

Project root is resolved from the payload `cwd` via `git rev-parse
--show-toplevel`. Markers are written under that resolved root, not the
hook's raw `cwd`.

## Session Correlation Model

Codex does have a locally verified SessionStart/session-record path in this
repo, but `schook` has not yet normalized that path into the generic provider
contract the way Claude already has. Treat Claude-equivalent continuity as
unapproved until `Phase N` captures and models the full Codex session
contract.

Current practical correlation inputs:

1. `thread-id` from the notify payload — stable within a session turn sequence
2. `CODEX_THREAD_ID` env var — same value, available without payload parsing
3. `ATM_TEAM` + `ATM_IDENTITY` as routing labels
4. explicit `session_id` when provided by `PreToolUse`

Design rule:

- do not claim Claude-equivalent session continuity for Codex until the
  session-record path and hook payload fields are captured and normalized into
  the generic provider contract

## Design Implications For `sc-hooks`

- Codex should be documented as a separate compatibility target, not squeezed
  into the Claude hook assumptions
- `notify` (not `Stop`) is the verified turn-complete surface for Codex; all
  idle-detection and debounce designs must use `notify`
- `PreToolUse` is the verified resumption signal and is sufficient for
  debounce cancellation
- frontmatter support makes Codex a better target for agent-local guard hooks
  than Claude, but the session-identity story is currently weaker
- the current Codex evidence includes both turn-complete and session lifecycle
  relay handling, so planning must use the verified event model rather than the
  narrower turn-complete script alone

## Test Harness

`test-harness/hooks/codex/` contains a pytest suite covering the debounce
contract (5 tests, all passing as of 2026-05-22):

This is a prototype baseline only. `Phase N` `N.1` must extend the harness so
the approved manifest and findings ledger record every audited Codex hook
surface, not just `notify` / `PreToolUse` debounce behavior.

| Test | What it verifies |
|------|-----------------|
| `test_codex_harness_layout_exists` | harness directory structure present |
| `test_stop_hook_schedules_pending_record` | notify writes pending record + active marker |
| `test_pretooluse_cancels_pending_debounce` | PreToolUse deletes pending, restores active, no CLI fire |
| `test_fire_pending_runs_command_once_after_due_time` | due timer fires CLI exactly once, flips to idle |
| `test_project_scope_blocks_outside_directories` | scope gate ignores hooks from outside project root |

Run:

```bash
pytest test-harness/hooks/codex/tests/ -m provider_codex -v
```

Environment variables:

| Variable | Purpose |
|----------|---------|
| `SCHOOK_CODEX_HOOK_STATE_ROOT` | pending record directory |
| `SCHOOK_CODEX_DEBOUNCE_SECONDS` | debounce window in seconds |
| `SCHOOK_CODEX_DEBOUNCE_COMMAND` | JSON-encoded command list to invoke on fire |
| `SCHOOK_CODEX_DEBOUNCE_AUTOSTART` | set `0` in tests to suppress subprocess timer |
| `SCHOOK_CODEX_HOOK_PROJECT_ROOT` | scope gate; hooks outside this path are ignored |
| `SCHOOK_HOOK_CAPTURE_ROOT` | raw payload + env JSON capture directory |

## Current Platform Gaps

- no provider-normalized Codex session identifier contract yet; the local
  SessionStart/session-record path is proven here but not yet modeled as a
  generic `schook` provider contract
- `Stop` hook unreliable in live exec; do not plan against it
- no verified upstream schema for Codex hook payload variants beyond
  `agent-turn-complete` and `PreToolUse`
- any future Codex planning should cite the runner or bundle source used to
  verify payload fields before those fields are promoted into `sc-hooks` docs
