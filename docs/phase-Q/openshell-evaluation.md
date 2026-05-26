# Sprint Q.1 — openshell Evaluation

recommendation: reject_for_now

smoke_runner_impact: `Q.2` should keep the repo-owned smoke runner on the already planned `just` plus Python path (`.just/run_smoke.py` and `.just/smoke/`) instead of inserting `openshell` as a new execution dependency. The current smoke design is centered on offline replay or dry-run execution, accepted-baseline live records, and an explicit operator surface under `just smoke`; none of that requires a shell replacement layer. Keeping the smoke runner on the existing path avoids reworking the `Phase Q` ownership boundary in `ADR-SHK-010` and keeps the runner aligned with the rest of the repo-local `just` helper surface.

ci_impact: Generic CI should not depend on `openshell` because the current `Phase Q` contract already requires an offline smoke gate that runs without provider CLIs and without ad hoc shell snippets. Adding `openshell` here would introduce a new tool-install requirement and a second execution abstraction without closing an identified CI gap. The current repo also does not have `openshell` installed or checked in locally, while the existing `just` plus Python path is already present and cross-platform-aware at the entrypoint layer (`windows-shell` in `justfile`, Python runner wrappers under `.just/`).

follow_on_for_q2:
- implement `.just/run_smoke.py` as the single owning dispatcher for `just smoke`
- keep offline CI smoke fixtures under `.just/smoke/fixtures/` with no `openshell` dependency
- keep provider-live smoke modules under `.just/smoke/` and document any remaining shell-specific calls explicitly
- leave `sc-hooks-test` on its current executable-fixture path for now; do not expand `Q.2` into a shell-runtime migration sprint

Rationale and tradeoffs:

- `sc-hooks-test` currently uses executable fixture scripts and direct `Command::new(...)` host execution rather than a shared shell abstraction. The fixture builders still emit `#!/bin/sh` scripts, and the host/runtime tests invoke binaries or scripts directly.
- `Phase Q` already freezes the smoke execution boundary around `just smoke`, `.just/run_smoke.py`, and `.just/smoke/`. Replacing that with `openshell` would create a second ownership path and force `Q.2` to relitigate the execution model.
- A supplement-only adoption also does not help enough right now: the repo has no existing `openshell` contract, no local installation, and no requirement that the smoke runner accept arbitrary shell expressions.
- The main open cross-platform concern is not “missing shell abstraction”; it is making sure the smoke runner and later provider additions keep explicit Windows-safe and non-Unix-only behavior. That should be handled in `Q.2` through the repo-owned runner and the existing lint/portability gates, not by introducing a new shell layer mid-phase.

Rejected alternatives:

- replace: rejected because `sc-hooks-test` and the planned smoke runner do not currently share an `openshell` seam, so replacement would expand `Q.1` into implementation redesign rather than recommendation freeze.
- supplement: rejected because it would add a second execution path for `Q.2` without a clear need, while still leaving the existing fixture and host-dispatch surfaces in place.
