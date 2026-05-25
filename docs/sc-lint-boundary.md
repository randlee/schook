# `sc-lint-boundary` Wrapper

`Phase P` keeps two public `sc-lint`-family entrypoints in the curated `just`
surface:

```bash
just lint sc-boundary
just lint sc-portability
```

Those commands are wrappers over the private `just _lint-sc-boundary` and
`just _lint-sc-portability` recipes in the repo `justfile`. The wrappers keep
the public command surface aligned with the `../atm-core` help/lint pattern
while reserving the implementation detail for helper scripts:

```text
just lint sc-boundary
  -> .just/run_lint.py sc-boundary
    -> just _lint-sc-boundary
      -> .just/lint_sc_boundary.py

just lint sc-portability
  -> .just/run_lint.py sc-portability
    -> just _lint-sc-portability
      -> .just/lint_sc_portability.py
```

## Backend Selection

Current preferred boundary backend on this machine:

- discover `sc-lint-boundary` from `PATH`
- prefer the Homebrew-installed `sc-lint-boundary 0.1.0` when discovery succeeds

Explicit fallback:

- repo-local `../sc-lint`
- invoked through `cargo run --manifest-path <sc-lint>/Cargo.toml -p sc-lint-boundary -- ...`

Current preferred portability backend:

- discover `sc-lint-portability` from `PATH`
- otherwise use repo-local `../sc-lint`
- invoked through `cargo run --manifest-path <sc-lint>/Cargo.toml -p sc-lint-portability -- ...`

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

For `Phase P`, portability joins that same required lint gate. New runtime
surface work is not considered ready when `just lint sc-portability` is red.
