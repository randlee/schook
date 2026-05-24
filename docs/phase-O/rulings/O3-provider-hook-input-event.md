# O.3 ProviderHookInput Event Field

## Decision

`ProviderHookInput<'a>` intentionally omits the planned
`event: Option<&'a str>` field.

## Why

The landed `O.3` normalization seam derives the approved event projection from
provider payload evidence inside the normalizers rather than passing a parallel
event hint through the raw input envelope:

- Codex and Gemini approved surfaces already carry the hook/event information
  needed to choose the canonical runtime projection
- `NormalizedHookContext.event` remains the canonical output of the seam
- adding `ProviderHookInput.event` now would duplicate information without
  changing the approved runtime behavior

## Scope

This ruling applies only to the `ProviderHookInput<'a>` input envelope in
`crates/sc-hooks-core/src/normalization.rs`.

It does not change:

- `NormalizedHookContext.event`
- the locked `(provider × hook × payload)` compatibility table
- the approved Codex and Gemini runtime projections recorded in
  `docs/architecture.md`
