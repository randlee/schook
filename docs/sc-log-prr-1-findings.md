# SC-LOG-PRR-1 Findings

Branch assessed: `integrate/logging-improvements`  
Assessment scope: production-readiness review for the observability phase and
first crates.io publication posture.

## Verdict

`integrate/logging-improvements` is not ready for crates.io publication.

The main blockers are packaging and release-metadata gaps in the publishable
crates, plus release-control drift between manifests, workflows, and the public
docs.

## Findings

### PRR-001 | Blocking | Publishable crates still use versionless workspace-only dependencies

`sc-hooks-sdk` and `sc-hooks-cli` cannot be packaged for crates.io because their
publishable dependencies are declared as `path` only, without version
requirements.

Evidence:
- `crates/sc-hooks-sdk/Cargo.toml:11`
- `crates/sc-hooks-cli/Cargo.toml:18-20`
- `cargo package --allow-dirty -p sc-hooks-sdk --no-verify` fails with `all dependencies must have a version requirement specified when packaging`
- `cargo package --allow-dirty -p sc-hooks-cli --no-verify` fails with the same error

Impact:
- the first-release crates cannot complete a normal `cargo package` flow
- publication will fail even before crates.io upload

### PRR-002 | Blocking | `sc-hooks-cli` still depends on unpublished local-only crates

The first-release CLI crate still depends on sibling path crates that are not
publish-ready and on `sc-hooks-test`, which the release inventory explicitly
marks as unpublished.

Evidence:
- `crates/sc-hooks-cli/Cargo.toml:16-20`
- `crates/sc-hooks-cli/src/testing.rs:3-49`
- `crates/sc-hooks-cli/src/main.rs:55-56`
- `release/publish-artifacts.toml:25-45`
- `.github/workflows/release-preflight.yml:131-138`

Impact:
- `sc-hooks-cli` cannot be published in the documented first-release shape
- the current observability dependency model still requires a sibling checkout of
  `sc-observability`
- the current CLI release artifact also has a normal dependency on
  `sc-hooks-test` even though `sc-hooks-test` is explicitly not published

### PRR-003 | Blocking | Publishable crates have no license metadata

The publishable crates do not declare a license or license file. The workspace
package defaults define only edition, toolchain, and version, so the publishable
crates inherit no release-license metadata at all.

Evidence:
- `Cargo.toml:19-24`
- `crates/sc-hooks-core/Cargo.toml:1-5`
- `crates/sc-hooks-sdk/Cargo.toml:1-5`
- `crates/sc-hooks-cli/Cargo.toml:1-5`
- `cargo metadata --no-deps --format-version 1` reports `license = null` and
  `license_file = null` for `sc-hooks-core`, `sc-hooks-sdk`, and `sc-hooks-cli`

Impact:
- this is below a production publication bar for crates.io release
- release/legal posture is incomplete for every publishable crate

### PRR-004 | Important | Publishable crates are missing basic crates.io metadata and readmes

The publishable crates also lack description, readme, repository, homepage, and
documentation metadata, and there are no crate-local README files under
`crates/`.

Evidence:
- `Cargo.toml:19-24`
- `crates/sc-hooks-core/Cargo.toml:1-5`
- `crates/sc-hooks-sdk/Cargo.toml:1-5`
- `crates/sc-hooks-cli/Cargo.toml:1-5`
- `cargo package --allow-dirty -p sc-hooks-core --no-verify` warns that the
  manifest has no description, license, documentation, homepage, or repository
- `find crates plugins -maxdepth 2 -name README.md -o -name README` returns no
  README files beneath `crates/` or `plugins/`

Impact:
- crates.io pages for the publishable crates would ship with weak or absent
  package metadata
- release consumers would have no crate-scoped README landing page

### PRR-005 | Important | Release preflight does not package-check all publishable crates

The release inventory marks `sc-hooks-sdk` and `sc-hooks-cli` as publishable,
but preflight only runs `cargo package` and `cargo publish --dry-run` for
artifacts whose `preflight_check = "full"`. `sc-hooks-sdk` and `sc-hooks-cli`
are currently marked `locked`, so their packaging failures are not caught by the
preflight path.

Evidence:
- `release/publish-artifacts.toml:15-23`
- `release/publish-artifacts.toml:37-45`
- `.github/workflows/release-preflight.yml:141-169`
- `scripts/release_artifacts.py:178-185`

Impact:
- the current release gate misses real publication blockers for two of the three
  publishable crates
- packaging failure is deferred until the later release path instead of being
  stopped in preflight

### PRR-006 | Important | Plugin maturity and publish posture are inconsistent across manifests and docs

Four plugin source crates are still publishable by default and describe
themselves as runtime-implementation crates, while the control docs and README
present all `plugins/` crates as scaffold/reference and outside the first
release.

Evidence:
- `plugins/agent-session-foundation/Cargo.toml:1-6`
- `plugins/agent-spawn-gates/Cargo.toml:1-6`
- `plugins/atm-extension/Cargo.toml:1-6`
- `plugins/tool-output-gates/Cargo.toml:1-6`
- `docs/architecture.md:84-100`
- `docs/requirements.md:133-135`
- `docs/traceability.md:58-60`
- `README.md:104-121`

Impact:
- release posture is not internally honest or mechanically enforced
- contributors can read the workspace manifests and conclude a different release
  boundary than the control docs describe

### PRR-007 | Important | Public README is stale and contradicts the shipped CLI/config surface

The public README still documents the old `sc-hooks-cli` command posture,
contains an invalid install path from repo root, omits the `crates/` prefixes in
the workspace map, and states that observability sink routing is not config
driven even though `[observability]` layering is now implemented.

Evidence:
- `README.md:35-37`
- `README.md:59-61`
- `README.md:67`
- `README.md:74-93`
- `README.md:100-103`
- `README.md:145`
- `crates/sc-hooks-cli/Cargo.toml:7-9`
- `docs/requirements.md:41`
- `docs/requirements.md:172-179`
- `docs/architecture.md:45-50`
- `docs/architecture.md:231-247`

Impact:
- public installation and invocation guidance is inaccurate
- the README no longer matches the actual binary name or the implemented
  observability config model

### PRR-008 | Minor | Control docs still reference nonexistent `DEF-017a`

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
