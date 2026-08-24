---
id: R.2b
title: Python Wiring — Queue-Get Half
status: planned
branch: feature/pR-s2b-queue-get-wiring
worktree: ../schook-worktrees/feature/pR-s2b-queue-get-wiring
target: integrate/phase-R
---

# Sprint R.2b — Python Wiring: Queue-Get Half

## Goal

Activate the queue-get (bare-CLI Stop-pull) half of the Stop entry that
R.2a installed default-off — flipping `queue_get` to `true` by default
and proving the pull path live. Same ownership rule: atm-core AQ2.5 owns
the script and contracts; schook owns the wiring.

## Hard Dependencies

- R.2a merged (the Stop entry, config surface, and replay harness exist;
  this sprint changes one default and adds coverage/evidence).
- **External**: atm-core's full AQ1 → AQ2 → AQ2.5 chain landed
  (`atm _internal-queue-get` + the bare-CLI FIFO exist only then).

## Exact Targets

- `queue_get` default flipped to `true` in the R.2a config skeleton
  (env override unchanged: `SC_ATM_QUEUE_GET=0` disables per host).
- Replay tests extended with the queue-get Stop fixtures from the R1
  corpus: pull-on-Stop emits the literal Claude block JSON when the stub
  daemon returns messages; never-block-on-empty; daemon-unreachable
  exits 0 within `daemon_timeout_ms`.

## Acceptance Criteria

1. Replay: queue-get fixtures produce the expected
   `atm _internal-queue-get` invocation and block/no-block behavior;
   `SC_ATM_QUEUE_GET=0` suppresses the pull with heartbeats unaffected.
2. Live evidence on one host: a real bare-CLI Claude member with two
   queued messages observed pulling one-per-stop through the generated
   entry; transcript retained.
3. `just lint` + CI lanes green.

## Out of Scope

- Any generator or config-surface redesign (R.2a's contract stands).
- Rust plugin work (R3) and cutover (R4).
