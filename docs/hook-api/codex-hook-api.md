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
- `CODEX_THREAD_ID` was present in captured hook environments, but in the
  2026-05-22 `N.1` fixtures it remained a stale outer-session value and did
  **not** match the captured `SessionStart.session_id`, `PreToolUse.session_id`,
  or `notify.thread-id`; do not normalize it into canonical correlation
  identity yet
- frontmatter hooks should still avoid relative paths for the same reason as
  Claude hooks: current working directory is not a stable execution anchor
- project root should be resolved from the payload `cwd` field using
  `git -C <cwd> rev-parse --show-toplevel`, falling back to raw `cwd`; do not
  rely on shell cwd or `PWD` as the project anchor
- any Codex hook plan should treat path resolution as an explicit part of the
  contract instead of leaving it to shell cwd behavior

## Verified Hook Surfaces

### `SessionStart` — Direct Startup Surface

**Verified 2026-05-22**: local Codex `SessionStart` command hooks fire
directly and deliver raw stdin payload plus hook-process environment.

Verified payload fields:

| Field | Type | Notes |
|-------|------|-------|
| `hook_event_name` | string | `"SessionStart"` |
| `cwd` | string | launch working directory; changes under `-C/--cd` |
| `model` | string | captured as `"gpt-5.4"` |
| `permission_mode` | string | captured as `"bypassPermissions"` |
| `session_id` | string | stable startup session id; matches later `notify.thread-id` in the same run |
| `source` | string | captured as `"startup"` |
| `transcript_path` | string | Codex transcript path for the session; captured with a leading `~`, so expand it before absolute filesystem use |

Verified environment fields:

| Variable | Notes |
|----------|-------|
| `ATM_IDENTITY` | captured and stable |
| `ATM_TEAM` | captured and stable |
| `CODEX_CI` | present in local `codex exec` runs |
| `CODEX_MANAGED_BY_NPM` | present in local runs |
| `CODEX_MANAGED_PACKAGE_ROOT` | present in local runs |
| `CODEX_THREAD_ID` | present but stale; not the captured startup session id |

### `notify` / `agent-turn-complete` — Turn-Complete Signal

**Verified 2026-05-22**: live Codex sessions fire `notify` reliably on turn
completion. This is the correct idle and debounce-start signal.

`Stop` did **not** fire in live Codex exec. Do not use `Stop` as the primary
turn-complete or idle-detection surface.

Verified payload fields:

| Field | Type | Notes |
|-------|------|-------|
| `client` | string | captured as `codex_exec` |
| `type` | string | `"agent-turn-complete"` |
| `thread-id` | string | thread UUID; stable correlation key within a session |
| `turn-id` | string | turn UUID for the completed turn |
| `cwd` | string | working directory at hook fire time |
| `input-messages` | array | user prompt list for the completed turn |
| `last-assistant-message` | string | final assistant message for the turn |

Verified environment fields:

| Variable | Notes |
|----------|-------|
| `ATM_IDENTITY` | agent identity when set in session environment |
| `ATM_TEAM` | team routing label when set in session environment |
| `CODEX_CI` | present in local `codex exec` runs |
| `CODEX_MANAGED_BY_NPM` | present in local runs |
| `CODEX_MANAGED_PACKAGE_ROOT` | present in local runs |
| `CODEX_THREAD_ID` | present but stale in `N.1` captures; not equal to `thread-id` |

### `PreToolUse` — Resumed-Activity Signal

**Verified 2026-05-22**: `PreToolUse` fires reliably before Codex tool
execution and is sufficient as a cancellation signal for pending debounce
timers.

Verified payload fields:

| Field | Type | Notes |
|-------|------|-------|
| `hook_event_name` | string | `"PreToolUse"` |
| `cwd` | string | working directory at hook fire time |
| `model` | string | captured as `"gpt-5.4"` |
| `session_id` | string | matched the turn-complete `thread-id` in captured runs |
| `turn_id` | string | current turn UUID |
| `tool_name` | string | captured as `"Bash"` |
| `tool_use_id` | string | stable tool invocation id |
| `tool_input.command` | string | captured command for Bash tool use |
| `transcript_path` | string | Codex transcript path for the session; captured with a leading `~`, so expand it before absolute filesystem use |

### `Stop` — Not Reliable

Live testing confirmed `Stop` did not fire in live Codex exec. Do not use
`Stop` as the primary idle-detection or turn-complete signal. Use `notify`
(`agent-turn-complete`) instead.

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

Captured Codex `SessionStart` hooks plus the existing startup hook logic write
a stable session record under:

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

1. `session_id` from direct `SessionStart` payloads
2. `session_id` from `PreToolUse`
3. `thread-id` from `notify`
4. `ATM_TEAM` + `ATM_IDENTITY` as routing labels

Do **not** currently use `CODEX_THREAD_ID` as a canonical correlation field.
The `N.1` raw env fixtures captured it as a stale outer-session value.

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
- the current Codex evidence is limited to repo-owned harness captures and the
  approved fixture/model set in this repo; do not promote relay-only fields
  unless a future harness capture proves them directly

## Test Harness

`test-harness/hooks/codex/` now contains the `N.1` provider line: approved
fixtures, provider-specific models, and an 11-test pytest suite covering both
the debounce behavior and the approved fixture/manifest contract.

| Test | What it verifies |
|------|-----------------|
| `test_codex_harness_layout_exists` | harness directory structure present |
| `test_notify_hook_schedules_pending_record` | notify writes pending record + active marker |
| `test_pretooluse_cancels_pending_debounce` | PreToolUse deletes pending, restores active, no CLI fire |
| `test_fire_pending_runs_command_once_after_due_time` | due timer fires CLI exactly once, flips to idle |
| `test_notify_prefers_session_record_and_sends_atm_idle_notice` | notify uses the canonical session record and writes the idle ATM notice state |
| `test_manifest_has_required_top_level_keys` | approved manifest validates and records audited surfaces |
| `test_approved_payload_fixtures_validate_against_models` | approved payload fixtures validate against provider models |
| `test_approved_env_fixtures_validate_and_redact_sensitive_values` | approved env fixtures validate and redact sensitive values |
| `test_session_start_and_stop_capture_scripts_write_raw_files` | direct SessionStart and direct Stop wrappers write raw capture files |
| `test_manifest_records_cd_scenario_through_approved_fixtures` | `--cd` scenario is preserved in approved fixtures |
| `test_project_scope_blocks_outside_directories` | scope gate ignores hooks from outside project root |

Run:

```bash
pytest test-harness/hooks/codex/tests/ -q
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
- `CODEX_THREAD_ID` cannot yet be treated as a canonical Codex session field;
  current fixture evidence shows it can remain stale across new `codex exec`
  runs
- `Stop` hook unreliable in live exec; do not plan against it
- no verified upstream schema for Codex hook payload variants beyond the
  directly captured `SessionStart`, `PreToolUse`, and `notify`
- any future Codex planning should cite the runner or bundle source used to
  verify payload fields before those fields are promoted into `sc-hooks` docs

## N.3 Normalization Carry-Forward

Canonical mapping candidates proved in `N.3` from Codex fixture evidence plus
at least one other provider:

- `session_id`
- `cwd`
- `transcript_path`
- `tool_input.command` on shell-like tool surfaces

Fields that remain Codex-local after `N.3`:

- `thread-id`
- `turn-id`
- `tool_use_id`
- `client`
- `type`
- `input-messages`
- `last-assistant-message`
- `model`
- `permission_mode`
- `source`
