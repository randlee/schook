# sc-hooks Implementation Gaps

This document tracks gaps between the current codebase and the release-standard documentation set.

## Gap Closure Tracker

| Gap | Severity | Owner area | Verification method | Early retire / replace candidates |
| --- | --- | --- | --- | --- |
| GAP-001 | blocker | `sc-hooks-test`, `sc-hooks-cli` | Direct compliance assertions for timeout, invalid output, async misuse, matcher validity, and absent-payload behavior | retire duplicated compliance logic in `sc-hooks-cli/src/testing.rs`; remove the duplicate absent-payload pseudo-check in `sc-hooks-test/src/compliance.rs` |
| GAP-002 | important | `sc-hooks-sdk`, `sc-hooks-cli`, docs | One end-to-end `long_running` contract proven across manifest validation, runtime behavior, docs, and tests | retire or replace `sc-hooks-sdk::traits::LongRunning` and `AsyncContextSource` unless they become real release-contract surfaces |
| GAP-003 | important | docs, plugin source crates, release packaging | Supported-plugin claims match runtime installation, behavior, and tests | retire old "bundled plugin" language before promoting any source crate to shipped behavior |
| GAP-004 | important | docs, examples/setup, `sc-hooks-cli` | A checked-in example or setup guide proves the expected `.sc-hooks/` runtime layout | none yet |
| GAP-005 | important | `sc-hooks-cli` logging path and consumers | Log-shape union is either discriminated or directly tested as stable | replace ad-hoc mixed log records with an explicit stable union or discriminator |
| GAP-006 | deferred | `sc-hooks-cli`, `sc-hooks-core` | Exit-code tests and docs agree on any future split | none until the exit taxonomy changes |
| GAP-007 | important | `sc-hooks-cli`, sibling `../sc-observability` workspace | Dependency boundary is enforced in manifests/review and logger wiring stays at the CLI/application edge | replace in-workspace logger wiring only after the `sc-observability` boundary is adopted cleanly |

## GAP-001: Compliance Harness Overclaims Coverage

- Severity: `blocker`
- Source: `CLI-007`, `TST-007`
- Owner area:
  - `sc-hooks-test`, `sc-hooks-cli`
- Current behavior:
  - `sc-hooks-test` and `sc-hooks-cli test` verify manifest loading, basic contract compatibility, simple matcher checks, positive timeout shape, minimal JSON output, and a duplicate minimal-input invocation labeled as absent-payload handling.
- Expected behavior:
  - The reusable compliance harness should directly verify the behaviors the release docs promise, including async misuse, timeout behavior, invalid JSON, multi-object stdout handling, and real absent-payload behavior.
- Verification method:
  - direct compliance assertions for timeout, invalid output, async misuse, matcher validity, and absent-payload behavior
- Recommended fix path:
  - Expand `sc-hooks-test` first, then align `sc-hooks-cli test` output and `docs/traceability.md`.
  - Consolidate the duplicated compliance code so the CLI delegates to one shared compliance engine instead of maintaining a second implementation.
- Early retire / replace candidates:
  - `sc-hooks-cli/src/testing.rs`
  - the duplicate absent-payload pseudo-check in `sc-hooks-test/src/compliance.rs`

## GAP-002: `LongRunning` SDK Contract Does Not Match Host Reality

- Severity: `important`
- Source: `TMO-004`
- Owner area:
  - `sc-hooks-sdk`, `sc-hooks-cli`, docs
- Current behavior:
  - The host honors manifest fields `long_running`, `timeout_ms`, and `description`.
  - Audit rejects `long_running=true` for async handlers and requires a non-empty description.
  - `sc-hooks-sdk::traits::LongRunning` only exposes `description(&self) -> &str` and is not the real end-to-end public contract described in older docs.
- Expected behavior:
  - The docs, SDK convenience surface, and tests should agree on one release-grade `long_running` contract.
- Verification method:
  - one end-to-end `long_running` contract proven across manifest validation, runtime behavior, docs, and tests
- Recommended fix path:
  - Treat `long_running` as a host manifest feature for now, and either tighten the SDK to match or explicitly defer richer SDK ergonomics.
- Early retire / replace candidates:
  - `sc-hooks-sdk::traits::LongRunning`
  - `sc-hooks-sdk::traits::AsyncContextSource`

## GAP-003: Bundled Plugin Readiness Was Previously Overstated

- Severity: `important`
- Source: `BND-001`, `BND-002`
- Owner area:
  - docs, plugin source crates, release packaging
- Current behavior:
  - Source crates under `plugins/` respond to `--manifest`, read stdin, and return `{\"action\":\"proceed\"}`.
  - Runtime plugin discovery does not read from `plugins/`; it reads from `.sc-hooks/plugins/`.
- Expected behavior:
  - The docs must describe these crates as scaffolds or reference implementations until they ship real behavior, installation guidance, and direct tests.
- Verification method:
  - supported-plugin claims match runtime installation, behavior, and tests
- Recommended fix path:
  - Keep the docs honest now; later either promote specific plugins to supported runtime artifacts or move them to an examples-only posture.
- Early retire / replace candidates:
  - old "bundled plugin" language in contributor-facing docs and release notes

## GAP-004: No Checked-In Example Runtime Layout

- Severity: `important`
- Source: `CFG-001`, `RES-002`, `CLI-004`
- Owner area:
  - docs, examples/setup, `sc-hooks-cli`
- Current behavior:
  - The host expects `.sc-hooks/config.toml` and `.sc-hooks/plugins/`, but the repository does not currently include a checked-in example runtime layout.
- Expected behavior:
  - Contributors should have a minimal documented example config and runtime plugin layout.
- Verification method:
  - a checked-in example or setup guide proves the expected `.sc-hooks/` runtime layout
- Recommended fix path:
  - Add a minimal example `.sc-hooks/` tree or a clearly linked setup guide before release.

## GAP-005: One Log File, Two Record Shapes, No Discriminator

- Severity: `important`
- Source: `OBS-001`, `OBS-002`
- Owner area:
  - `sc-hooks-cli` logging path and consumers
- Current behavior:
  - The builtin `log` handler and the dispatcher append different JSON shapes to the same JSONL file.
- Expected behavior:
  - Downstream log consumers should either have an explicit discriminator field or a guaranteed stable union contract.
- Verification method:
  - log-shape union is either discriminated or directly tested as stable
- Recommended fix path:
  - Either add a `record_type` field to both shapes or keep the union stable and test it explicitly.
- Early retire / replace candidates:
  - ad-hoc mixed log record parsing assumptions in downstream consumers
  - the current undocumented union if the project standardizes on one record envelope later

## GAP-006: Exit-Code Taxonomy Is Coarse Around Resolution-Time Manifest Failures

- Severity: `deferred`
- Source: `EXC-004`, `DEF-004`
- Owner area:
  - `sc-hooks-cli`, `sc-hooks-core`
- Current behavior:
  - unresolved handlers and manifest-load or manifest-compatibility failures all map to the same resolution exit code (`4`)
- Expected behavior:
  - if the project wants finer-grained operational diagnosis, manifest incompatibility may eventually deserve a dedicated exit code
- Verification method:
  - exit-code tests and docs agree on any future split
- Recommended fix path:
  - keep the current behavior documented honestly unless and until the codebase introduces a new exit-code split

## GAP-007: sc-observability Logging Boundary Is Specified But Not Yet Adopted

- Severity: `important`
- Source: `OBS-006`, `OBS-007`, `OBS-008`
- Owner area:
  - `sc-hooks-cli`, sibling `../sc-observability` workspace
- Current behavior:
  - `schook` currently uses in-workspace logging code under `sc-hooks-cli` and does not yet depend on the sibling `../sc-observability` workspace
- Expected behavior:
  - the next logging integration pass should adopt the logging-only `sc-observability` crate, with ownership limited to `sc-hooks-cli` and final binary wiring
  - `sc-hooks-core`, `sc-hooks-sdk`, and `sc-hooks-test` should remain logging-implementation-agnostic
- Verification method:
  - dependency boundary is enforced in manifests/review and logger wiring stays at the CLI/application edge
- Recommended fix path:
  - add the dependency only at the CLI/application boundary and document or enforce the boundary in workspace manifests and review policy
- Early retire / replace candidates:
  - in-workspace logger initialization and sink ownership once the `sc-observability` boundary is adopted cleanly
