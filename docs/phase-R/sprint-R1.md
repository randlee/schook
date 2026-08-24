---
id: R.1
title: Codex Stop Reclassification + Verified Payload Replay Corpus
status: planned
branch: feature/pR-s1-stop-reclass-corpus
worktree: ../schook-worktrees/feature/pR-s1-stop-reclass-corpus
target: integrate/phase-R
---

# Sprint R.1 — Codex Stop Reclassification (#168) + Verified Payload Replay Corpus

## Goal

- close **schook#168**: reclassify Codex `Stop` from
  disposition-only/not-exercisable to verified/supported across every doc
  that carries the wrong claim
- commit a **verified payload replay corpus** — recorded, sanitized hook
  payloads for the events the ATM liveness pipeline consumes — as the
  fixture set R2 tests against and the byte-parity gate R3 must pass

## Hard Dependencies

- none (atm-core is not required; the corpus is provider-side only)

## Exact Targets

- `docs/hook-api/codex-hook-api.md` (lines ~72–73, ~118–129: "`Stop` did
  not fire" / "Not Reliable" / "confirmed-not-exercisable")
- `docs/architecture.md` (~:204 disposition-only retained-surface row)
- `docs/phase-P/canonical-hook-mapping.md` (~:19)
- `docs/requirements.md` (`HKR-006` acceptance condition 5) +
  `docs/traceability.md` (`HKR-006` row)
- new: corpus fixtures under the existing per-provider capture
  convention — `test-harness/hooks/claude/captures/atm-liveness/` and
  `test-harness/hooks/codex/captures/atm-liveness/` — holding: Claude
  `Stop` / `PreToolUse` / `SessionEnd`; Codex `Stop` / `PreToolUse` /
  `notify` (agent-turn-complete) — each payload recorded from a live
  session (the existing `test_live_capture.py` tooling), secrets/paths
  sanitized, with provenance notes (provider version, capture date,
  host)

## Acceptance Criteria

1. Grep gate: no doc in the repo still claims Codex `Stop` is
   disposition-only / not exercisable; #168 closed referencing the merge
   commit.
2. Corpus fixtures load and validate under the existing harness tooling;
   each fixture carries provenance metadata; Codex `Stop` fixtures match
   the fields the production baseline consumes (`session_id`,
   `thread-id`, `turn-id`, `cwd`).
3. `just lint` + existing CI lanes green.

## Evidence

- Capture transcript(s) retained on the branch (live-verify precedent:
  the 2026-08-24 rand-m4 verification that grounded #168).

## Out of Scope

- Any runtime/normalization code change; `notify` stays documented as
  valid — the correction is that `Stop` is ALSO exercisable.
