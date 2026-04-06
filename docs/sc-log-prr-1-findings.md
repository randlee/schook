# SC-LOG-PRR-1 Findings

Branch assessed: `integrate/logging-improvements`  
Assessment scope: production-readiness review for the observability phase and
first crates.io publication posture.

## Verdict

`integrate/logging-improvements` is ready for phase-end merge, but not yet ready
for end-to-end crates.io publication of `sc-hooks-cli`.

The local packaging, metadata, and release-control gaps identified in the
original PRR pass are now closed. The remaining boundary is external: the CLI
still depends on `sc-observability` crates that are not yet published at the
required version, so the current crates.io wave is limited to
`sc-hooks-core` and `sc-hooks-sdk`.

## Status After `SC-LOG-PRR-FIX-R1`

- `PRR-001` resolved: publishable local crate dependencies are version-pinned,
  and dependency-aware batch packaging now validates `sc-hooks-core` plus
  `sc-hooks-sdk`.
- `PRR-003` and `PRR-004` resolved: publishable crates now inherit license and
  baseline crates.io metadata from the workspace package section.
- `PRR-005` resolved: release preflight now batch-runs `cargo package` and
  `cargo publish --dry-run` for the current publishable set instead of silently
  skipping dependency-aware package validation.
- `PRR-006` and `PRR-007` resolved: plugin publish posture and README/release
  docs now align with the control documents.
- `PRR-002` narrowed to the remaining external gate: `sc-hooks-cli` stays out of
  the current crates.io wave until the required `sc-observability` versions are
  published.

## Findings

### PRR-002 | Remaining External Gate | `sc-hooks-cli` still depends on unpublished external observability crates

The CLI crate still depends on sibling path crates for `sc-observability` and
`sc-observability-types`, and those crates are not yet published at the
required version on crates.io.

Evidence:
- `crates/sc-hooks-cli/Cargo.toml:18-19`
- `cargo package --allow-dirty -p sc-hooks-core -p sc-hooks-sdk --no-verify` passes
- `cargo publish --dry-run --allow-dirty -p sc-hooks-core -p sc-hooks-sdk --no-verify` passes
- `cargo package --workspace --allow-dirty --no-verify` still fails when it reaches `sc-hooks-cli` with `failed to select a version for the requirement 'sc-observability = "^1.0.0"'`

Impact:
The current repo is merge-ready for the observability phase, but crates.io
publication of the CLI crate is still blocked on external publication work. The
binary release channels remain buildable from source in this repo.

### PRR-008 | Closed In `SC-LOG-PRR-FIX-R1` | Control docs still reference nonexistent `DEF-017a`

The control docs still mention `DEF-017a`, but `docs/requirements.md` defines no
such requirement ID.

Evidence:
- `docs/logging-contract.md:15-17`
- `docs/phase-observability-plan.md:489-490`
- `docs/phase-observability-plan.md:516`
- `docs/requirements.md:169-180`

Impact:
- observability traceability is not fully internally consistent at phase end
- review and QA readers cannot map every cited requirement ID back to the
  authoritative requirement table

### PRR-009 | Minor | The documented `hooks` alias is not actually shipped

The requirements and plan describe `hooks` as the supported convenience CLI
alias, but the published binary configuration and release artifact inventory only
ship `sc-hooks`.

Evidence:
- `docs/requirements.md:180`
- `docs/project-plan.md:193-194`
- `crates/sc-hooks-cli/Cargo.toml:7-9`
- `release/publish-artifacts.toml:47-48`

Impact:
- the phase-end naming decision is only partially implemented
- public references to a supported alias currently overstate what the release
  artifacts provide

## Validation

Commands run during this assessment:
- `cargo metadata --no-deps --format-version 1`
- `cargo package --allow-dirty -p sc-hooks-core --no-verify`
- `cargo package --allow-dirty -p sc-hooks-sdk --no-verify`
- `cargo package --allow-dirty -p sc-hooks-cli --no-verify`
- `cargo package --allow-dirty -p agent-session-foundation --no-verify`

Required branch validation after adding this report:
- `cargo +1.94.1 clippy --all-targets --all-features -- -D warnings`
- `cargo +1.94.1 test --workspace`
- `cargo +1.94.1 test --workspace --release`

Result:
- all three required validation commands passed on `integrate/logging-improvements`
