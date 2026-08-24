---
id: R.2a
title: Python Wiring — Heartbeat Half (Install Generation, Config, Observability)
status: planned
branch: feature/pR-s2a-heartbeat-wiring
worktree: ../schook-worktrees/feature/pR-s2a-heartbeat-wiring
target: integrate/phase-R
---

# Sprint R.2a — Python Wiring: Heartbeat Half

## Goal

Make the atm-core-owned Python hook entry points first-class citizens of
schook's dispatch surface for the **heartbeat pipeline** — installable via
`sc-hooks install`, configured through one surface, observable through
sc-observability — **without duplicating a line of their logic**.
atm-core AQ2.5 owns the scripts and JSON contracts; schook owns the
wiring. Queue-get wiring is R.2b (split per PLAN-SCOPE-001: its external
dependency chain is longer and must not gate heartbeats).

## Hard Dependencies

- R1 corpus merged.
- **External**: atm-core AQ2.5 deliverables 1–2 landed (heartbeat CLI +
  `scripts/hooks/` entry points). Nothing else — the daemon Heartbeat
  route already exists.

## Exact Targets

- `sc-hooks install` generation: Claude `settings.json` entries
  (`PreToolUse` → active heartbeat, `Stop` → the single AQ2.5 Stop
  script, `SessionEnd` → session-ended) and the Codex `hooks.json`
  equivalents, all invoking the atm-core entry points by their
  documented paths — generated, never hand-maintained.
  **Stop-entry activation contract (single-script, per AQ2.5)**: AQ2.5
  ships ONE Claude Stop script that does both the debounced-idle
  heartbeat and the queue-get pull. The generator always emits exactly
  one Stop entry invoking that script; which halves are active is
  controlled by the env vars the generated entry carries, driven by the
  config keys below. **Per-provider queue-get gating (Codex has no
  injection surface — draining its FIFO would discard nudges with no
  way to deliver them, per AQ2.5's disclosed gap)**: the `queue_get`
  key applies to Claude only; the Codex generator NEVER emits the
  queue-get env, unconditionally, until a Codex injection surface
  exists (no config can enable it). This sprint ships Claude
  `queue_get = false` (heartbeat half only); R.2b flips the Claude
  default. No second Stop entry, no generator fork.
- **Config surface (normative skeleton)** — one TOML table plus 1:1 env
  overrides. **Naming honesty**: these names are **originated here, by
  schook**, inside the repo's existing canonical `SC_HOOKS_*` namespace
  (`docs/cross-platform-guidelines.md` already defines
  `SC_HOOKS_STATE_DIR`); AQ2.5's doc names no env vars, and the
  production baseline's `SCHOOK_CODEX_IDLE_*` names are superseded at
  migration. Cross-repo alignment item: atm-core AQ2.5 dev must adopt
  these names (or alias them) in its `scripts/hooks/` entry points —
  tracked as an explicit coordination task, not assumed:

  ```toml
  [atm-liveness]
  # default: "${SC_HOOKS_STATE_DIR}/atm-liveness"
  state_root = ""                 # SC_HOOKS_ATM_STATE_ROOT
  idle_debounce_seconds = 60      # SC_HOOKS_ATM_IDLE_DEBOUNCE_SECONDS
  daemon_timeout_ms = 500         # SC_HOOKS_ATM_DAEMON_TIMEOUT_MS
  autostart_timer = true          # SC_HOOKS_ATM_AUTOSTART_TIMER
  queue_get = false               # SC_HOOKS_ATM_QUEUE_GET — Claude only;
                                  # Codex: never emitted (see above)
  ```

  No hidden state files; any state file written atomically.
- Observability: hook invocations and failures land in the sc-hooks
  JSONL log per `docs/observability-contract.md`; a hook failure is
  visible but NEVER blocks the agent (fail-open asserted by test).
- Replay tests: the R1 heartbeat-relevant corpus driven through the
  wired entry points (stub `atm` binary recording invocations);
  assertions cover emitted CLI calls, env propagation from the config
  keys, and exit codes.

## Acceptance Criteria

1. `sc-hooks install` on a clean fixture tree produces Claude + Codex
   entries that invoke the atm-core entry points with the configured
   env; exactly one Stop entry per provider; re-running is idempotent.
2. Replay of the heartbeat corpus: expected `atm _internal-heartbeat`
   invocations and exit codes for every fixture; with `queue_get =
   false` no queue-get invocation occurs on any Stop fixture;
   daemon-unreachable simulation exits 0 within `daemon_timeout_ms` for
   every entry point.
3. Config round-trip: every TOML key above reaches the hook process as
   its documented env var; env override beats TOML (test per key).
4. Cross-platform: tests green on the repo's lanes; every subprocess
   capture and file I/O uses explicit `encoding="utf-8"`; no glibc-only
   strftime.
5. Live evidence on one host (Claude + Codex): heartbeats observed at
   the atm daemon (`RuntimeHealth` via `atm doctor` or equivalent),
   transcript retained.
6. `just lint` + CI lanes green.

## Out of Scope

- Queue-get wiring, evidence, and the `queue_get = true` default —
  **R.2b** (explicit non-closure; not a silent carry-forward).
- Editing anything under atm-core's `scripts/hooks/` (cross-repo PR if a
  contract gap is found — AQ2.5 is authoritative).
- Per-host migration of the existing `~/.codex/scripts/` installation
  (ops follow-up; R4 documents the flip).
