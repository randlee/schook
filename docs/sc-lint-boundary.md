# `sc-lint-boundary` Wrapper

`Phase P` keeps two public `sc-lint`-family entrypoints in the curated `just`
surface:

```bash
just lint boundary
just lint portability
```

Those commands are wrappers over the private `just _lint-boundary` and
`just _lint-portability` recipes in the repo `justfile`. The wrappers keep
the public command surface aligned with the `../atm-core` help/lint pattern
while reserving the implementation detail for helper scripts:

```text
just lint boundary
  -> .just/run_lint.py boundary
    -> just _lint-boundary
      -> .just/lint_sc_boundary.py

just lint portability
  -> .just/run_lint.py portability
    -> just _lint-portability
      -> .just/lint_sc_portability.py
```

## Backend Selection

Current preferred boundary backend on this machine:

- discover `sc-lint-boundary` from `PATH`
- prefer the Homebrew-installed `randlee/tap/sc-lint` toolset or a direct
  `cargo install sc-lint-boundary@0.2.0` installation when discovery succeeds

Current preferred portability backend:

- discover `sc-lint-portability` from `PATH`
- prefer the Homebrew-installed `randlee/tap/sc-lint` toolset or a direct
  `cargo install sc-lint-portability@0.2.0` installation when discovery
  succeeds

If either backend is unavailable, the matching public lint command fails
immediately instead of silently skipping enforcement.

## What The Wrapper Enforces

The wrapper does two things:

1. runs a self-check fixture that proves the analyzer detects:
   - `boundary.internal_only` violations
   - `boundary.forbid_external_impls` violations
2. runs `sc-lint-boundary analyze` against this repo

The boundary record for the planned normalization seam lives at:

- `boundaries/sc-hooks-core/provider-normalization.toml`

The intended enforcement scope for `O.2` is narrow:

- internal-only protection for the private normalization module seam
- no external impls for the planned `ProviderHookNormalizer` trait

This sprint installs the enforcement path. `O.3` lands the actual normalization
trait and runtime types that will sit behind that boundary.

For `Phase P`, boundary and portability are paired required lint gates. New
runtime surface work is not considered ready when either `just lint boundary`
or `just lint portability` is red.
