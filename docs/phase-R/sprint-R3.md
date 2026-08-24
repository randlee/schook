---
id: R.3
title: Rust Plugin schook-atm — atm-core Library Link + Byte-Parity Gate
status: planned
branch: feature/pR-s3-schook-atm-plugin
worktree: ../schook-worktrees/feature/pR-s3-schook-atm-plugin
target: integrate/phase-R
---

# Sprint R.3 — Rust Plugin `schook-atm`: atm-core Library Link + Byte-Parity Gate

## Goal

An sc-hooks plugin that subsumes the Python entry points — same events,
same observable emissions, same exit codes — implemented in Rust and
calling atm-core **as a library** (Rand: "simply reference atm-core, not
shell out to atm").

## Hard Dependencies

- R2 merged (wiring + replay tests exist; the corpus is the gate).

## Design decisions frozen here

- **New plugin crate `plugins/schook-atm`**, not an extension of
  `atm-extension`: verified 2026-08-24 that `atm-extension` has **no
  atm-core dependency** (relay/session-state only) — grafting a heavy
  library dependency onto it would entangle two unrelated concerns.
  `atm-extension`'s relay behavior is untouched.
- **Dependency form**: atm-core crates via git dependency pinned to a
  release tag (default; path-dep for local dev; publishing to crates.io
  is a later decision with Rand). The plugin calls the same client seam
  the atm CLI uses (`atm_daemon_client::resolve_daemon_local_ipc_endpoint`
  + `atm_http_runtime::preferred_local_client`) to emit
  `TeamMemberHeartbeatRequest` and the AQ2.5 queue-get envelope directly
  — no `atm` subprocess.
- **Simplicity mandate carries over**: the plugin is a thin translation
  from hook payload → one daemon request → one observable emission; no
  caching, no retry queues, no state beyond the debounce timestamps the
  Python version already keeps (same file locations, atomic writes).

## Exact Targets

- `plugins/schook-atm/` (manifest, sync/async handlers per the sc-hooks
  plugin protocol; boundaries entry per `boundaries/` conventions)
- Parity harness: every R1/R2 corpus fixture replayed through the Rust
  plugin must produce **byte-identical observable behavior** to the
  Python entry points — same daemon requests (captured via a stub
  endpoint), same stdout JSON (e.g. the Claude block shape), same exit
  codes. Divergence is a failing test, not a judgment call.

## Acceptance Criteria

1. Full-corpus byte-parity: zero divergences Python vs Rust across every
   fixture, including daemon-unreachable (bounded-timeout exit 0) and
   never-block-on-empty cases.
2. Plugin passes the sc-hooks-test compliance harness; manifest and
   boundary records accepted by the repo's lint surface.
3. `cargo test` workspace green on the repo's lanes; the atm-core
   dependency builds on all of them.
4. No `Command::new("atm")`/subprocess-to-atm anywhere in the plugin
   (grep gate).

## Out of Scope

- Cutover of any host (R4).
- Retiring the Python entry points (R4).
- Any atm-core-side change beyond consuming its published library API
  (gaps go back to atm-core as issues/PRs).
