# CDR PR Assessment

Date: `2026-05-23`

Scope:
- PR `#100` `feature/cdr-a-observability-fix -> develop`
- PR `#101` `feature/cdr-plan-phase -> develop`
- PR `#102` `feature/cdr-b-doc-reconciliation -> integrate/cdr`
- PR `#105` `feature/cdr-3 -> integrate/cdr`

## Summary

These PRs should not be merged as a batch.

- `#100` is superseded by current `develop`.
- `#101` is superseded by current `develop`.
- `#102` contains intent that is partly already reflected on current `develop`,
  but its merge path through `integrate/cdr` is stale.
- `#105` depends on the stale `integrate/cdr` path and should not merge as-is.

The actual blocker before `O.1` is not “merge `integrate/cdr` first.” The real
blocker is that current control docs on `develop` still contain stale
`CDR-B pending merge` language even though much of the underlying CDR intent is
already present elsewhere in the repo.

## PR 100

PR:
- `#100` `fix: pin sc-observability to crates.io v1.0.0 and fix observability.rs typed API`

What it does:
- pins `sc-observability` and `sc-observability-types` to crates.io `1.0.0`
- fixes typed observability API usage
- includes ETXTBSY-related test-harness fixes

Current status against `develop`:
- the key observability dependency pin is already present in
  `crates/sc-hooks-cli/Cargo.toml`
- the ETXTBSY retry helper is already present and used by
  `crates/sc-hooks-test/src/fixtures.rs`
- the PR branch is old and not the right vehicle for current `develop`

Merge readiness:
- not merge-ready as a PR branch

Blockers:
- stale branch
- content already landed or was replaced by later merged work
- PR still targets old code/doc shape rather than current `develop`

Phase O dependency status:
- no remaining Phase O dependency

Recommended action:
- close as superseded

## PR 101

PR:
- `#101` `docs: add Change Drift Remediation (CDR) phase to project-plan.md`

What it does:
- adds CDR rows to `docs/project-plan.md`

Current status against `develop`:
- `docs/project-plan.md` already contains CDR rows plus later Phase N and Phase O
  planning
- the PR is conflicting and written against an older planning model

Merge readiness:
- not merge-ready

Blockers:
- conflicting with current `docs/project-plan.md`
- superseded by later planning merges
- would reintroduce older wording and older sequencing assumptions

Phase O dependency status:
- no direct dependency

Recommended action:
- close as superseded

## PR 102

PR:
- `#102` `CDR-B: doc/arch reconciliation — promote plugins to production-track, reconcile hook phase status`

What it does:
- promotes `agent-session-foundation`, `agent-spawn-gates`,
  `tool-output-gates`, and `atm-extension`
- reconciles hook-phase status and control docs
- targets `integrate/cdr`, not `develop`

Current status against `develop`:
- parts of the intended result are already visible on `develop`
  - README already describes the four runtime crates as runtime implementation
    source crates with direct tests
  - `docs/traceability.md` already marks their relevant requirements as
    implemented
  - `docs/project-plan.md` already shows Hook Phases 3-5 as completed
- other CDR-B-era assumptions are now stale
  - `docs/architecture.md` still classifies those crates as
    scaffold/reference in section `3.2`
  - `docs/project-plan.md` still says `CDR-B` is pending merge and still treats
    it as a Phase O dependency
  - `docs/requirements.md` still uses `pending CDR-B merge` language in several
    hook rows

Merge readiness:
- not merge-ready through the current PR path

Blockers:
- base branch `integrate/cdr` is stale and not the active integration path
- PR is conflicting
- branch carries a large older doc/code surface that should not be merged
  wholesale into current `develop`

Phase O dependency status:
- the PR itself is not the right dependency
- the remaining dependency is narrower:
  current `develop` still needs a fresh reconciliation pass that removes stale
  `CDR-B pending merge` language and aligns architecture/requirements/project
  plan to the already-landed runtime/plugin reality

Recommended action:
- do not merge `#102` as-is
- replace it with a fresh, narrow PR from current `develop` that:
  - updates `docs/architecture.md` section `3.2`
  - removes stale `CDR-B pending merge` blockers from
    `docs/project-plan.md` and `docs/requirements.md`
  - keeps only the still-correct reconciliation outcomes

## PR 105

PR:
- `#105` `CDR-3: residual gap review closeout`

What it does:
- closes residual CDR gap review items
- depends on `integrate/cdr` and the CDR-B path

Current status against `develop`:
- some of its intent is already present on `develop`
  - `docs/implementation-gaps.md` already contains `NT-CLI-002`,
    `HRN-005`, and `COW-003`
- the branch still depends on the stale `integrate/cdr` baseline

Merge readiness:
- not merge-ready as part of the old CDR stack

Blockers:
- depends on stale `integrate/cdr`
- inherits the obsolete merge path behind `#102`
- carries older plan/doc assumptions that current `develop` has already moved
  past

Phase O dependency status:
- no direct hard dependency once any still-missing residual doc points are
  carried into a fresh `develop` PR

Recommended action:
- do not merge `#105` as-is
- close or defer it after extracting any still-missing residual doc updates into
  a fresh `develop` PR if needed

## Recommended Sequencing

1. Close `#100` as superseded.
2. Close `#101` as superseded.
3. Do not merge `#102` or `#105` through `integrate/cdr`.
4. Create one fresh `develop` PR for the remaining CDR reconciliation that is
   still actually missing on current `develop`.

That fresh PR should:
- remove stale `CDR-B pending merge` language from control docs
- reconcile `docs/architecture.md` plugin classification with the README and
  traceability state already on `develop`
- update `docs/project-plan.md` so `Phase O` is not blocked on an obsolete
  `integrate/cdr` merge path

## Bottom Line

`integrate/cdr -> develop` is not the right next step.

The right next step is:
- treat `#100` and `#101` as obsolete
- treat `#102` and `#105` as stale source material
- land one new narrow reconciliation PR on top of current `develop`
