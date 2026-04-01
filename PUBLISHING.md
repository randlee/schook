# Publishing Guide

This repo uses a single source of truth for release artifacts:

- Manifest: `release/publish-artifacts.toml`
- Loader/generator: `scripts/release_artifacts.py`

Do not hardcode crate lists, publish order, or release binary lists in docs or
workflows. Update the manifest instead.

## Distribution Channels

- crates.io crates defined in `release/publish-artifacts.toml`
- GitHub Releases: <https://github.com/randlee/schook/releases>
- Homebrew tap formula: `Formula/sc-hooks.rb` in
  <https://github.com/randlee/homebrew-tap>
- WinGet package: `randlee.sc-hooks`

## Important Release Blocker

`crates/sc-hooks-cli` still depends on sibling `path` dependencies from
`../sc-observability`. That means a real crates.io release is blocked until:

1. `sc-observability` publish-prep is finished in
   <https://github.com/randlee/sc-observability/issues/30>
2. `schook` replaces those `path` dependencies with exact published version
   pins

Until that work is complete:

- you may run local release validation
- you may update release docs/workflows
- do not run the real crates.io publish workflow for `schook`

## Workflows

- Preflight: `.github/workflows/release-preflight.yml`
- Release: `.github/workflows/release.yml`

Both workflows are manual dispatch workflows.

## Standard Flow

1. Ensure `develop` contains the release version bump and is ready for merge.
2. Run preflight workflow with:
   - `version=<X.Y.Z or vX.Y.Z>`
   - `run_by_agent=publisher`
3. Preflight fails if any publishable crate in
   `release/publish-artifacts.toml` is already published at that version.
4. Merge `develop` to `main` once CI and preflight are green.
5. Run `scripts/release_gate.sh` locally or in CI from `main`.
6. Run release workflow with `version=<X.Y.Z or vX.Y.Z>`.
7. Release workflow gates, tags, builds archives, publishes crates from the
   manifest in manifest order, verifies publish outcomes, creates the GitHub
   release, updates Homebrew, and prepares the WinGet submission.

## Manifest-Driven Behavior

`release/publish-artifacts.toml` defines:

- crate artifact identity and package name
- crate `Cargo.toml` path
- required/publish flags
- publish order
- preflight check mode (`full` or `locked`)
- post-publish propagation wait seconds
- whether post-publish `cargo install` verification is required
- release binary list for archive packaging

Current planned publish order:

1. `sc-hooks-core`
2. `sc-hooks-sdk`
3. `sc-hooks-test` (tracked, not published)
4. `sc-hooks-cli`

## Local Validation Commands

```bash
# Show publish plan (package|wait_seconds)
python3 scripts/release_artifacts.py list-publish-plan \
  --manifest release/publish-artifacts.toml

# Show release binaries that will be archived
python3 scripts/release_artifacts.py list-release-binaries \
  --manifest release/publish-artifacts.toml

# List tracked Cargo manifests from the publish manifest
python3 scripts/release_artifacts.py list-cargo-tomls \
  --manifest release/publish-artifacts.toml

# Generate inventory JSON from manifest
python3 scripts/release_artifacts.py emit-inventory \
  --manifest release/publish-artifacts.toml \
  --version 0.1.0 \
  --tag v0.1.0 \
  --commit "$(git rev-parse HEAD)" \
  --source-ref refs/heads/develop \
  --output release/release-inventory.json

# Preflight guard: fail if version already exists on crates.io
python3 scripts/release_artifacts.py check-version-unpublished \
  --manifest release/publish-artifacts.toml \
  --version 0.1.0

# Release gate checks
./scripts/release_gate.sh
```

## Updating Release Artifacts

When adding, removing, or reordering release crates or binaries:

1. Update `release/publish-artifacts.toml`.
2. Run:
   - `python3 scripts/release_artifacts.py list-artifacts --manifest release/publish-artifacts.toml`
   - `python3 scripts/release_artifacts.py list-release-binaries --manifest release/publish-artifacts.toml`
   - `cargo test --workspace`
   - `cargo clippy --all-targets --all-features -- -D warnings`
3. Update `PUBLISHING.md` if the release flow changed materially.

## Homebrew Notes

The release workflow updates `Formula/sc-hooks.rb` in
`randlee/homebrew-tap`. If the formula does not exist yet, the workflow will
create it from the release metadata.

## WinGet Notes

The release workflow prepares a WinGet submission for package identifier:

- `randlee.sc-hooks`

The initial workflow uses `wingetcreate` in a basic update/submit flow. Keep
the package identifier stable once the first submission is accepted.
