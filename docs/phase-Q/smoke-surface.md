# Phase Q Smoke Surface

`Q.2` establishes one curated smoke surface for the supported runtime
providers:

- public entrypoint: `just smoke`
- implementation owner: `.just/run_smoke.py`
- provider-module root: `.just/smoke/`
- repo-owned offline assets: `.just/smoke/fixtures/`

Execution model:

- `just smoke all ci` is the CI-owned offline gate
- generic CI does not require Claude, Codex, or Gemini CLIs
- accepted-baseline live provider smoke remains a later sprint concern:
  - `Q.3` Claude
  - `Q.4` Codex
  - `Q.5` Gemini

Operator guidance:

- use `just smoke all ci` to prove the public smoke surface and dispatcher
  path without mutating live home config or requiring provider installs
- use provider-specific live smoke only after the later provider sprints land
- `Q.2` rejects `openshell` as a new required execution dependency; the smoke
  runner stays on the repo-owned `just` plus Python path established in `Q.1`

Current deterministic zero-provider behavior:

- before any provider smoke modules are added, `just smoke all ci` succeeds
  and prints the zero-provider result
  - `smoke: mode=ci providers=all discovered=0 executed=0`

Out of scope for `Q.2`:

- provider-specific live smoke records
- Cursor Agent harness work
- opencode harness work
