---
id: P.4
title: Lifecycle-Family Normalization Extension
status: planned
branch: feature/pP-s4-lifecycle-normalization-extension
worktree: ../schook-worktrees/feature/pP-s4-lifecycle-normalization-extension
target: integrate/phase-P
---

# Sprint P.4 — Lifecycle-Family Normalization Extension

## Goal

- extend the canonical runtime-normalization model so the newly retained
  lifecycle surfaces can run through one shared seam instead of provider-local
  one-offs

## Hard Dependencies

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
    Stop,
    Resume,
    // Fork appears only if `P.1` retains it as supported.
    Fork,
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
    pub(crate) canonical_payload: &'static str,
    pub(crate) available_variables: &'a [&'a str],
    pub(crate) provider_local_fields: &'a [&'a str],
}
```

The exact Rust type names may differ in the landed code, but `P.4` must
produce an equivalent typed contract plus the published table artifact above.
`P.4` extends the existing `CanonicalHook` / `CodexHook` / `GeminiHook`
hierarchy established by `Phase O`; it does not introduce a peer top-level
canonical hook enum unless a new ADR explicitly approves that architecture
change first. If `P.1` lands Codex `fork` as unsupported or explicitly
deferred, `P.4` must still record that disposition in the mapping table and
lifecycle-compatibility rules instead of silently dropping the surface from the
canonical inventory, and the Rust canonical hook type inventory must not carry
live variants for `P.1`-ruled-unsupported surfaces.

## Acceptance Criteria

- the newly retained Codex/Gemini lifecycle surfaces have one documented and
  tested canonical runtime path
- no provider bypass path is introduced around the existing normalization seam
- the compatibility table and boundary docs match the landed code
- the cross-agent mapping table exists and is sufficient to compare Claude,
  Codex, and Gemini parity surface-by-surface
- `P.4` extends the existing `CanonicalHook` hierarchy instead of introducing
  a peer canonical hook family, unless a new approved ADR explicitly records
  that architectural change
- any existing Codex- or Gemini-specific runtime bypass path is either removed
  in this sprint or called out as a blocking defect

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
