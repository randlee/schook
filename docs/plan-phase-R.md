# Phase R Plan — ATM Agent Liveness Hooks: Python Wiring, Rust Plugin, Cutover

## Goal

Make schook the production dispatcher for the ATM agent liveness/heartbeat
and queue-delivery-trigger pipeline, in two stages per Rand (2026-08-24):

1. **Durable Python first** — wire the atm-core-owned Python hook entry
   points (Claude Code + Codex) into schook's dispatch/install/observability
   surfaces so they are startable immediately and production-grade.
2. **Rust MVP second** — an sc-hooks plugin that links **atm-core as a
   library** (never shelling the `atm` CLI) and subsumes the Python entry
   points behind a byte-parity replay gate, then per-host cutover and
   Python retirement.

Phase R does **not** include:

- authoring the hook business logic or its JSON contracts — those are
  **owned by atm-core sprint AQ2.5**
  (`atm-core:docs/plans/phase-aq/sprint-AQ2-5-queue-delivery-triggers.md`):
  the `scripts/hooks/` Python entry points, `atm _internal-heartbeat`,
  `atm _internal-queue-get`, the `DeliveryChannel` classifier, and the
  bare-CLI RAM FIFO. schook consumes those contracts; it never redefines
  or forks them. On any mismatch, AQ2.5 is authoritative and the fix is a
  cross-repo coordination, not a schook-side divergence.
- graft receiver liveness — that is atm-core AQ1.5–AQ1.9's SQLite lease
  mechanism (separate by design; hooks cover agent/session liveness,
  leases cover graft receiver endpoints; both are read-time derivations).
- any new provider runtime-parity claims (Phase P/Q rules stand).

## Binding guardrails (Rand, 2026-08-24 — apply to every sprint)

- **Simplicity**: no hard-to-debug machinery, no redundant state machines;
  liveness is derived at read time from timestamped facts, never stored as
  a status that must be kept in sync (the v1 atm-daemon lesson).
- **Never wedge the agent**: every hook is fail-open — daemon unreachable
  or any error ⇒ log + exit 0 within a bounded timeout.
- **Script quality bar**: whole defect classes, well-tested, actionable
  errors, no dead branches.
- **Cross-platform traps** (bit atm-core CI twice on 2026-08-24): explicit
  `encoding="utf-8"` on every subprocess capture and file I/O; no
  glibc-only strftime formats; atomic writes (temp + rename) for any state
  file.
- **Repo code vs host migration**: committed deliverables are in-repo;
  per-host cutover (`~/.claude/settings.json`, `~/.codex/hooks.json`,
  `~/.codex/scripts/`) is documented ops, never a PR-gated deliverable
  (AQ2.5 hardening precedent).

## Baseline

- planning branch: `docs/phase-R-planning`
- integration branch: `integrate/phase-R`
- prerequisite baseline: `develop` after Phase Q planning merge (#155)
- grounding facts (verified 2026-08-24, fenix on rand-m4):
  - Codex `Stop` fires reliably — **schook#168** corrects the repo docs
    that classified it disposition-only/not-exercisable; the production
    baseline `~/.codex/scripts/schook_codex_idle.py` (Stop-debounce +
    cancel-on-PreToolUse + detached timer) passed an isolated end-to-end
    smoke test the same day.
  - The atm daemon's `Heartbeat` route exists with **no in-tree
    producer** (`TeamMemberHeartbeatRequest`,
    `atm-core:crates/atm-core/src/protocol.rs` ~:330; handler
    `storage_and_nudge_router.rs` ~:456, Local-ingress-only) — the hooks
    this phase wires are that producer.
  - atm-core PR #1011 (merged 2026-08-24) hardened AQ2.5 through three
    reviewer rounds; PR #1019 carries its final ownership fixes.

## External dependencies (atm-core)

| Phase R needs | atm-core owner | Status at planning time |
|---|---|---|
| `atm _internal-heartbeat` CLI + Python hook scripts (`scripts/hooks/`) | sprint AQ2.5 deliverables 1–2 | planned (plan merged, dev not started) |
| `atm _internal-queue-get` + bare-CLI FIFO | sprint AQ2.5 deliverable 3 (needs AQ1 `atm queue` + AQ2 chain first) | planned |
| atm-core crates consumable as a library dependency | AQ2.5 non-closure names the Rust plugin as its MVP successor | R3 decides git-dep vs path-dep vs published crate with Rand |

Sequencing consequence: R1 can start **now** (corpus + doc corrections need
nothing from atm-core). R2 starts when AQ2.5's heartbeat half lands; its
queue-get wiring activates when the AQ1→AQ2→AQ2.5 chain lands. R3/R4
follow R2.

## Sprints

| Sprint | Title | Depends |
|---|---|---|
| R1 | Codex Stop reclassification (#168) + verified payload replay corpus | — |
| R2 | Python wiring: install generation, config surface, observability (atm entry points; contracts owned by AQ2.5) | must_follow R1 · external: AQ2.5 heartbeat half (queue-get wiring gated on AQ1→AQ2→AQ2.5) |
| R3 | Rust plugin `schook-atm`: heartbeat + queue-get via atm-core library link; byte-parity replay gate over the R1/R2 corpus | must_follow R2 |
| R4 | Cutover + Python retirement: per-host dispatcher flip (ops-documented), live parity evidence (Claude + Codex), grep gate | must_follow R3 |

Branch pattern: `feature/pR-sN-<slug>` off `integrate/phase-R`, PR target
`integrate/phase-R`; phase completion PR `integrate/phase-R` → `develop`.

## QA history

| Round | Reviewer(s) | Commit | Verdict | Notes |
|---|---|---|---|---|
| 0 | — (initial draft, fenix) | (this commit) | DRAFT | Subject confirmed by Rand 2026-08-24 (liveness/heartbeat + delivery-trigger pipeline, per `schook-liveness-plan-handoff.md`). Hardening rounds pending. |
