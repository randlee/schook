# sc-hooks Implementation Gaps

This document tracks gaps between the current codebase and the release-standard documentation set.

## GAP-001: Compliance Harness Overclaims Coverage

- Severity: `blocker`
- Source: `CLI-007`, `TST-007`
- Current behavior:
  - `sc-hooks-test` and `sc-hooks-cli test` verify manifest loading, basic contract compatibility, simple matcher checks, positive timeout shape, minimal JSON output, and a duplicate minimal-input invocation labeled as absent-payload handling.
- Expected behavior:
  - The reusable compliance harness should directly verify the behaviors the release docs promise, including async misuse, timeout behavior, invalid JSON, multi-object stdout handling, and real absent-payload behavior.
- Recommended fix path:
  - Expand `sc-hooks-test` first, then align `sc-hooks-cli test` output and `docs/traceability.md`.

## GAP-002: `LongRunning` SDK Contract Does Not Match Host Reality

- Severity: `important`
- Source: `TMO-004`
- Current behavior:
  - The host honors manifest fields `long_running`, `timeout_ms`, and `description`.
  - Audit rejects `long_running=true` for async handlers and requires a non-empty description.
  - `sc-hooks-sdk::traits::LongRunning` only exposes `description(&self) -> &str` and is not the real end-to-end public contract described in older docs.
- Expected behavior:
  - The docs, SDK convenience surface, and tests should agree on one release-grade `long_running` contract.
- Recommended fix path:
  - Treat `long_running` as a host manifest feature for now, and either tighten the SDK to match or explicitly defer richer SDK ergonomics.

## GAP-003: Bundled Plugin Readiness Was Previously Overstated

- Severity: `important`
- Source: `BND-001`, `BND-002`
- Current behavior:
  - Source crates under `plugins/` respond to `--manifest`, read stdin, and return `{\"action\":\"proceed\"}`.
  - Runtime plugin discovery does not read from `plugins/`; it reads from `.sc-hooks/plugins/`.
- Expected behavior:
  - The docs must describe these crates as scaffolds or reference implementations until they ship real behavior, installation guidance, and direct tests.
- Recommended fix path:
  - Keep the docs honest now; later either promote specific plugins to supported runtime artifacts or move them to an examples-only posture.

## GAP-004: No Checked-In Example Runtime Layout

- Severity: `important`
- Source: `CFG-001`, `RES-002`, `CLI-004`
- Current behavior:
  - The host expects `.sc-hooks/config.toml` and `.sc-hooks/plugins/`, but the repository does not currently include a checked-in example runtime layout.
- Expected behavior:
  - Contributors should have a minimal documented example config and runtime plugin layout.
- Recommended fix path:
  - Add a minimal example `.sc-hooks/` tree or a clearly linked setup guide before release.

## GAP-005: One Log File, Two Record Shapes, No Discriminator

- Severity: `important`
- Source: `OBS-001`, `OBS-002`
- Current behavior:
  - The builtin `log` handler and the dispatcher append different JSON shapes to the same JSONL file.
- Expected behavior:
  - Downstream log consumers should either have an explicit discriminator field or a guaranteed stable union contract.
- Recommended fix path:
  - Either add a `record_type` field to both shapes or keep the union stable and test it explicitly.
