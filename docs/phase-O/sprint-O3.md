---
id: O.3
title: Runtime Normalization Foundation
status: planned
branch: feature/pO-s3-runtime-normalization-foundation
worktree: ../schook-worktrees/feature/pO-s3-runtime-normalization-foundation
target: integrate/phase-O
---

# Sprint O.3 — Runtime Normalization Foundation

## Goal

- implement the provider-to-canonical normalization layer for approved Codex
  and Gemini surfaces only
- require one sealed normalization trait boundary so provider-specific parsing
  cannot bypass the canonical runtime path

## Hard Dependencies

- `O.2` complete
- current `origin/integrate/phase-N` accepted baseline
- approved provider fixtures, models, and hook API docs from `Phase N`
- `CDR-B` merged to the execution baseline
- `RULING-NEEDED-ECR-001` disposition recorded as closed or explicitly
  deferred past `Phase O` before the `O.3` branch is cut
- `sc-lint-boundary` enforcement active in this repo

## Exact Targets

- `crates/sc-hooks-core/src/`
- `crates/sc-hooks-cli/src/`
- `crates/sc-hooks-cli/tests/`
- `docs/architecture.md`
- `test-harness/hooks/codex/`
- `test-harness/hooks/gemini/`

## Deliverables

- runtime normalization code for approved Codex and Gemini surfaces
- one sealed normalization trait boundary
- fixture-backed normalization tests
- one documented normalization boundary for provider-specific vs canonical data
- named normalization-error inventory with the chosen `HookError` integration
  path recorded explicitly
- `docs/architecture.md` runtime-boundary section describing the
  `ProviderHookNormalizer` seam, canonical type surface, and lint-enforcement
  policy

## Required Signatures

Normalization seam:

```rust
pub(crate) trait ProviderHookNormalizer: private::Sealed {
    fn normalize<'a>(
        &self,
        raw: ProviderHookInput<'a>,
    ) -> Result<NormalizedHookContext<'a>, HookError>;
}

pub(crate) struct ProviderHookInput<'a> {
    pub(crate) provider: ProviderHookSource,
    pub(crate) raw: &'a serde_json::Value,
    pub(crate) event: Option<&'a str>,
    pub(crate) metadata_path: Option<&'a Path>,
}

pub(crate) enum ProviderHookSource {
    Codex,
    Gemini,
}
```

Canonical normalized data:

```rust
pub(crate) struct NormalizedHookContext<'a> {
    pub(crate) hook: CanonicalHook,
    pub(crate) event: Option<HookEventName<'a>>,
    pub(crate) session_id: Option<SessionId<'a>>,
    pub(crate) project_root: Option<&'a Path>,
    pub(crate) current_dir: Option<&'a Path>,
    pub(crate) tool_name: Option<ToolName<'a>>,
    pub(crate) payload: CanonicalPayload<'a>,
}
```

Approved canonical hooks:

```rust
pub(crate) enum CanonicalHook {
    Codex(CodexHook),
    Gemini(GeminiHook),
}

pub(crate) enum CodexHook {
    SessionStart,
    PreToolUse,
}

pub(crate) enum GeminiHook {
    SessionStart,
    SessionEnd,
    BeforeAgent,
    BeforeTool,
    AfterTool,
}
```

Canonical payload contract:

```rust
pub(crate) enum CanonicalPayload<'a> {
    Empty,
    ToolUse { tool_name: ToolName<'a>, body: &'a serde_json::Value },
    SessionLifecycle { body: &'a serde_json::Value },
    AgentLifecycle { body: &'a serde_json::Value },
}
```

Canonical newtypes:

```rust
pub(crate) struct SessionId<'a>(pub(crate) Cow<'a, str>);
pub(crate) struct ToolName<'a>(pub(crate) Cow<'a, str>);
pub(crate) struct HookEventName<'a>(pub(crate) Cow<'a, str>);
```

Normalization error inventory:

```rust
pub(crate) enum NormalizationError {
    MissingRequiredField { field: &'static str },
    InvalidFieldValue { field: &'static str, reason: &'static str },
    InvalidPayloadForHook { hook: CanonicalHook, payload_kind: &'static str },
    RetryableGateInput { field: &'static str, reason: &'static str },
    UnsupportedApprovedSurface { provider: ProviderHookSource, hook: &'static str },
}
```

`CanonicalHook` is only required to support `Debug`, so any `thiserror`
formatting for `InvalidPayloadForHook` should use `{hook:?}` rather than
assuming a `Display` impl.

Linted boundary marker:

```rust
#[sc_lint(boundary.internal_only)]
mod private;

#[sc_lint(boundary.forbid_external_impls)]
pub(crate) trait ProviderHookNormalizer { /* ... */ }
```

## Acceptance Criteria

- provider raw payload handling enters the runtime only through one sealed
  normalization trait boundary
- the normalization trait is crate-private and a compile-fail boundary test
  proves external impls are rejected
- all canonical normalization types remain `pub(crate)` in the implemented
  crate
- all canonical normalization enum struct-variant fields carry explicit
  `pub(crate)` annotations or a code comment explaining any intentional
  omission
- `NormalizedHookContext` is the canonical provider-normalized input and feeds
  the existing `HookContext` construction path; `Phase O` does not create a
  second parallel dispatch model
- invalid hook/payload combinations are rejected explicitly through the locked
  compatibility table in `docs/architecture.md` section `3.4`, with
  `NormalizationError::InvalidPayloadForHook` as the required failure path
- the chosen normalization-error taxonomy is recorded explicitly and enters the
  host error surface as `HookError::Normalization(NormalizationError)` unless a
  newer explicit architecture ruling supersedes that choice before O.3 begins
- retryable-vs-fatal normalization failures are defined explicitly for the
  approved `HKR-010` gate surfaces
- the implementation records how retryable normalization failures carry
  structured recovery metadata before `RetryableGateInput` is promoted to code
- approved Codex fixtures normalize into canonical runtime hook/event data
- approved Gemini fixtures normalize into canonical runtime hook/event data
- Claude remains the existing baseline runtime path and is documented as the
  behavior Codex and Gemini normalize into for later parity sprints
- deferred `Phase N` surfaces are not implemented or implied as supported
- normalization tests cite approved fixtures and pass
- normalization tests include a deterministic compatibility/property check that
  rejects invalid approved-surface hook/payload combinations
- boundary lint fails if provider-specific parsing bypasses the normalization
  trait boundary or if external impls are introduced

## Out Of Scope

- Codex `notify`, `Stop`, `resume`, `fork`
- Gemini `AfterAgent`
- local deployment and cutover

## Required Validation

- `cargo fmt --check --all`
- `cargo test --workspace`
- `just lint sc-boundary`
- `pytest test-harness/hooks/codex/tests/ -q`
- `pytest test-harness/hooks/gemini/tests/ -q`
- `git diff --check`
