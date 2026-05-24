# `sc-lint-boundary` Wrapper

`Phase O` uses one public boundary-lint entrypoint:

```bash
just lint sc-boundary
```

That command is a wrapper over the private `just _lint-sc-boundary` recipe in
the repo `justfile`. The wrapper keeps the public command surface aligned with
the `../atm-core` help/lint pattern while reserving the implementation detail
for one helper script:

```text
just lint sc-boundary
  -> .just/run_lint.py sc-boundary
    -> just _lint-sc-boundary
      -> .just/lint_sc_boundary.py
```

## Backend Selection

Current preferred backend on this machine:

- Homebrew-installed `sc-lint-boundary 0.1.0`
- binary path: `/opt/homebrew/bin/sc-lint-boundary`

Explicit fallback:

- repo-local `../sc-lint`
- invoked through `cargo run --manifest-path <sc-lint>/Cargo.toml -p sc-lint-boundary -- ...`

If neither backend is available, `just lint sc-boundary` fails immediately
instead of silently skipping boundary enforcement.

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
