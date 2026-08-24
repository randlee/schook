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

- R.2a and R.2b merged (wiring + replay tests exist for both halves;
  the corpus is the gate).

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
  Python entry points, where the comparison is precisely defined as:
  same daemon requests (captured via a stub endpoint), same stdout JSON
  (e.g. the Claude block shape), same exit codes, compared
  byte-for-byte **after masking the call-time-dependent fields the wire
  types carry — `pid` and every timestamp field
  (`TeamMemberHeartbeatRequest.observed_at` and any other
  `IsoTimestamp`)** — with the masked-field list enumerated in the
  harness and asserted non-empty per request type (so a new
  nondeterministic field fails loudly instead of being silently
  compared). Divergence outside the masked set is a failing test, not a
  judgment call.

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
5. `atm-extension` untouched (the load-bearing boundary from "Design
   decisions frozen here" is gated, not just asserted): `git diff
   develop... -- plugins/atm-extension/` is empty at PR time — the
   diff-empty check is the **sole** enforcement mechanism (schook's
   `sc-lint-boundary` does attribute-based source lints, not Cargo
   dependency-graph analysis, so no lint gate exists for this property
   and none is claimed).

## Out of Scope

- Cutover of any host (R4).
- Retiring the Python entry points (R4).
- Any atm-core-side change beyond consuming its published library API
  (gaps go back to atm-core as issues/PRs).
