---
id: R.2
title: Python Wiring — Install Generation, Config, Observability
status: planned
branch: feature/pR-s2-python-wiring
worktree: ../schook-worktrees/feature/pR-s2-python-wiring
target: integrate/phase-R
---

# Sprint R.2 — Python Wiring: Install Generation, Config Surface, Observability

## Goal

Make the atm-core-owned Python hook entry points first-class citizens of
schook's dispatch surface — installable via `sc-hooks install`, configured
through one surface, observable through sc-observability — **without
duplicating a line of their logic**. atm-core AQ2.5 owns the scripts and
JSON contracts; schook owns the wiring.

## Hard Dependencies

- R1 corpus merged.
- **External**: atm-core AQ2.5 deliverables 1–2 landed (heartbeat CLI +
  `scripts/hooks/` entry points). The queue-get wiring half additionally
  waits on atm-core's AQ1 → AQ2 → AQ2.5 chain; this sprint lands the
  heartbeat half first and activates queue-get wiring when its external
  dependency merges (split noted, no schook-side redesign either way).

## Exact Targets

- `sc-hooks install` generation: Claude `settings.json` entries
  (`PreToolUse` → active heartbeat, `Stop` → debounced idle, `SessionEnd`
  → session-ended; `Stop` → queue-get pull for bare-CLI members) and the
  Codex `hooks.json` equivalents, all invoking the atm-core entry points
  by their documented paths — generated, never hand-maintained.
- One config surface (repo-convention TOML + env overrides) for the knobs
  AQ2.5 defines as env-overridable (state root, debounce seconds,
  timeouts); no hidden state files; any state file written atomically.
- Observability: hook invocations and failures land in the sc-hooks JSONL
  log per `docs/observability-contract.md`; a hook failure is visible but
  NEVER blocks the agent (fail-open is asserted by test).
- Replay tests: the R1 corpus is driven through the wired entry points
  (stub `atm` binary recording invocations); assertions cover emitted
  CLI calls, exit codes, and the never-block-on-empty rule.

## Acceptance Criteria

1. `sc-hooks install` on a clean fixture tree produces Claude + Codex
   entries that invoke the atm-core entry points; re-running is
   idempotent.
2. Replay of the full R1 corpus: expected `atm` invocations and exit
   codes for every fixture; daemon-unreachable simulation exits 0 within
   the bounded timeout for every entry point.
3. Cross-platform: tests green on the repo's lanes; every subprocess
   capture and file I/O uses explicit `encoding="utf-8"`; no glibc-only
   strftime.
4. Live evidence on one host (Claude + Codex): heartbeats observed at the
   atm daemon (`RuntimeHealth` via `atm doctor` or equivalent), transcript
   retained.
5. `just lint` + CI lanes green.

## Out of Scope

- Editing anything under atm-core's `scripts/hooks/` (cross-repo PR if a
  contract gap is found — AQ2.5 is authoritative).
- Per-host migration of the existing `~/.codex/scripts/` installation
  (ops follow-up; R4 documents the flip).
