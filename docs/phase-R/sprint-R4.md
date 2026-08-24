---
id: R.4
title: Cutover + Python Retirement
status: planned
branch: feature/pR-s4-cutover-retirement
worktree: ../schook-worktrees/feature/pR-s4-cutover-retirement
target: integrate/phase-R
---

# Sprint R.4 — Cutover + Python Retirement

## Goal

Flip dispatch from the Python entry points to the `schook-atm` plugin,
prove live parity, and retire the Python path — leaving exactly one
implementation in service.

## Hard Dependencies

- R3 merged (byte-parity gate green).

## Exact Targets

- `sc-hooks install` generation switched to emit plugin-dispatch entries
  instead of Python entry-point invocations (one generator change; the
  per-host flip is re-running install — documented as ops).
- Cutover runbook (in-repo doc): per-host steps for `~/.claude` and
  `~/.codex`, including the dated-backup + `HOOK_CHANGES.md` ledger
  convention already in force on rand-m4 (`~/.scripts/README.md`
  policy), and rollback (restore backed-up entries, re-run install with
  the Python generator flag).
- Retirement: Python wiring targets from R2 that exist only to invoke
  the entry points are removed once no generated config references them.

## Acceptance Criteria

1. Live parity evidence on one host, Claude **and** Codex: heartbeats and
   (where the atm-core chain has landed) queue-get pulls observed via the
   plugin path, transcripts retained — same live-verify bar as R2 AC 4.
2. Grep gate: no generated or in-repo config references the Python entry
   points after cutover generation; the corpus and parity harness are
   retained (they remain the plugin's regression suite).
3. Rollback path exercised once in a fixture tree (install → flip →
   rollback → Python entries restored).
4. `just lint` + CI lanes green.

## Out of Scope

- Deleting the R1 corpus or parity harness (permanent regression assets).
- Any change to atm-core contracts.
