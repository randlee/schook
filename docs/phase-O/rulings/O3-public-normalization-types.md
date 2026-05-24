# O.3 Public Normalization Types

## Decision

`RuntimeProvider` and `NormalizedRuntimeDispatch` remain `pub` in
`crates/sc-hooks-core/src/normalization.rs`.

## Why

`sc-hooks-cli` is a separate crate and consumes the provider-normalization seam
across the crate boundary:

- `RuntimeProvider` selects the approved provider entrypoint before generic
  runtime dispatch
- `NormalizedRuntimeDispatch` carries the typed normalization result from
  `sc-hooks-core` into the `sc-hooks-cli` host runtime path

Making either type `pub(crate)` would force `sc-hooks-cli` to reimplement the
boundary contract locally or collapse the seam back into the CLI crate, which
would violate the O.3 boundary goal.

## Scope

This is a narrow exception only for:

- `RuntimeProvider`
- `NormalizedRuntimeDispatch`
- the `normalize_runtime_dispatch(...)` facade that carries them

All other canonical normalization types remain `pub(crate)` unless a later
Phase O ruling explicitly supersedes this one.
