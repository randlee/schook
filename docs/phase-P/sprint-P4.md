---
id: P.4
title: Lifecycle-Family Normalization Extension
status: complete
branch: feature/pP-s4-canonical-hook-mapping
worktree: ../schook-worktrees/feature/pP-s4-canonical-hook-mapping
target: integrate/phase-P
---

# Sprint P.4 — Lifecycle-Family Normalization Extension

## Goal

- extend the canonical runtime-normalization model so the newly retained
  lifecycle surfaces can run through one shared seam instead of provider-local
  one-offs

## Hard Dependencies

- accepted `P.1`
- accepted `P.2`
- accepted `P.3`
- current `ProviderHookNormalizer` boundary from `Phase O`

## Exact Targets

- `crates/sc-hooks-core/`
- `crates/sc-hooks-cli/`
- `docs/architecture.md`
- `docs/requirements.md`
- `docs/traceability.md`
- `boundaries/`
- `docs/phase-P/canonical-hook-mapping.md`

## Deliverables

- extended `CanonicalHook` provider-specific type inventory for the newly
  retained surfaces
- updated provider-to-canonical compatibility rules
- updated `sc-lint-boundary` records if the normalization seam grows
- architecture and requirement updates that freeze the new lifecycle contract
- one authoritative cross-agent mapping table that records, for each retained
  lifecycle surface:
  - provider hook name
  - canonical hook mapping
  - canonical payload family
  - available variables
  - provider-local fields not promoted into the canonical contract
- explicit removal plan for any direct provider-specific runtime path that
  bypasses `ProviderHookNormalizer`
- same-PR architecture authorization for the canonical typed-inventory
  extension, recorded through `docs/architecture.md` under `ADR-SHK-009`
- explicit `docs/requirements.md` amendment note stating that `P.4` may expand
  the retained lifecycle surface inventory under `HKR-006` but does not, by
  itself, close the `HKR-010` runtime gate because provider runtime parity is
  still owned by `P.5` and `P.6`

## Required Contract Samples

Authoritative mapping-table row shape:

```md
| Provider | Provider Hook | Canonical Hook | Canonical Payload | Available Variables | Provider-Local Fields | Notes |
| --- | --- | --- | --- | --- | --- | --- |
```

Required runtime-boundary contract additions:

```rust
pub(crate) enum CanonicalHook {
    Codex(CodexHook),
    Gemini(GeminiHook),
}

pub(crate) enum CodexHook {
    SessionStart,
    PreToolUse,
    Notify,
}

pub(crate) enum GeminiHook {
    SessionStart,
    SessionEnd,
    BeforeAgent,
    BeforeTool,
    AfterTool,
    AfterAgent,
}

pub(crate) struct CanonicalHookMappingRow<'a> {
    pub(crate) provider: ProviderHookSource,
    pub(crate) provider_hook: &'a str,
    pub(crate) canonical_hook: CanonicalHook,
    pub(crate) canonical_payload_family: &'static str,
    pub(crate) available_variables: &'a [&'a str],
    pub(crate) provider_local_fields: &'a [&'a str],
}
```

The exact Rust type names may differ in the landed code, but `P.4` must
produce an equivalent typed contract plus the published table artifact above.
`P.4` extends the existing `CanonicalHook` / `CodexHook` / `GeminiHook`
hierarchy established by `Phase O`; it does not introduce a peer top-level
canonical hook enum unless a new ADR explicitly approves that architecture
change first. `P.4` records Codex `Stop`, `resume`, and `fork` as
disposition-only rows in the mapping table because `P.1` confirmed they are
not exercisable retained runtime surfaces. The Rust canonical hook inventory
therefore adds live variants only for Codex `notify` and Gemini `AfterAgent`.
The mapping-row sample carries a doc/reporting label for the payload family
only. It does not replace the typed `CanonicalPayload<'a>` runtime enum used
inside the normalization seam or authorize stringly typed payload handling in
the implementation.

## Acceptance Criteria

- the newly retained live Codex/Gemini lifecycle surfaces have one documented
  and tested canonical runtime path
- no provider bypass path is introduced around the existing normalization seam
- the compatibility table and boundary docs match the landed code
- the cross-agent mapping table exists and is sufficient to compare Claude,
  Codex, and Gemini parity surface-by-surface
- `docs/architecture.md` carries the same-PR authorization for the canonical
  typed-inventory extension through `ADR-SHK-009`
- `P.4` extends the existing `CanonicalHook` hierarchy instead of introducing
  a peer canonical hook family, unless a new approved ADR explicitly records
  that architectural change
- any existing Codex- or Gemini-specific runtime bypass path is either removed
  in this sprint or called out as a blocking defect
- `P.4` explicitly records that its seam additions are consistent with the
  current `RULING-NEEDED-ECR-001` deferral and that `P.7` cannot retroactively
  remove already-landed seam additions without a new breaking-change sprint

## Out Of Scope

- provider-specific runtime parity wiring
- packaging/CLI alias work
- broader error-surface cleanup

## Required Validation

- `cargo check --workspace`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `just lint sc-boundary`
- `git diff --check`

## Sprint QA Checklist

- Which requirement IDs or gap IDs changed status?
- What code was removed early rather than left in parallel?
- Which files/crates were the owned write scope for the sprint?
- What validation commands and direct tests proved the new contract?
- What follow-on work is blocked or unblocked by this sprint?

## Sprint QA Checklist Answers

- Requirement IDs changed:
  - `HKR-006`
  - `HKR-010`
- Code removed early:
  - none; provider-runtime parity wiring stayed out of scope for `P.4`
- Owned write scope:
  - `crates/sc-hooks-core/`
  - `crates/sc-hooks-cli/`
  - `docs/architecture.md`
  - `docs/requirements.md`
  - `docs/traceability.md`
  - `docs/phase-P/canonical-hook-mapping.md`
  - `boundaries/`
- Validation that passed:
  - `cargo check --workspace`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test --workspace`
  - `just lint sc-boundary`
  - `git diff --check`
- Follow-on work:
  - `P.5` closes Codex retained provider-runtime parity
  - `P.6` closes Gemini retained provider-runtime parity
