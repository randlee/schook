# Phase Q Smoke Surface

`Q.2` establishes one curated smoke surface for the supported runtime
providers:

- public entrypoint: `just smoke`
- implementation owner: `.just/run_smoke.py`
- provider-module root: `.just/smoke/`
- repo-owned offline assets: `.just/smoke/fixtures/`

Execution model:

- `just smoke all ci` is the CI-owned offline gate
- `mode='ci'` means offline replay or dry-run execution against repo-owned
  assets only; it does not require provider CLIs or live home-config mutation
- `mode='live'` means provider modules may execute accepted-baseline live smoke
  flows once those later provider sprints land
- generic CI does not require Claude, Codex, or Gemini CLIs
- accepted-baseline live provider smoke remains a later sprint concern:
  - `Q.3` Claude
  - `Q.4` Codex
  - `Q.5` Gemini

Provider Module Contract:

- every provider module under `.just/smoke/` must export:

```python
def run(*, mode: str, repo_root: Path) -> int:
    ...
```

- `mode='ci'` must stay offline and use only repo-owned fixtures or dry-run
  assets
- `mode='live'` may execute the accepted-baseline provider smoke flow once that
  provider sprint lands
- provider modules must return an integer process-style exit code and raise
  `SystemExit` only for explicit user-facing smoke failures

`atm-core` alignment record:

- adopted:
  - one curated public `just smoke` entrypoint
  - repo-owned Python dispatcher under `.just/`
  - documented help-surface exposure through `.just/print_help.py`
- rejected:
  - introducing `openshell` as a second execution dependency
  - ad hoc shell-script smoke entrypoints outside the repo-owned dispatcher
  - treating provider-live smoke as a generic CI requirement

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

Current offline asset state:

- `.just/smoke/fixtures/placeholder.json` exists as the repo-owned Q.2
  placeholder proving the offline fixture path is materialized before
  `Q.3`–`Q.5` add provider-specific accepted-baseline records

Out of scope for `Q.2`:

- provider-specific live smoke records
- Cursor Agent harness work
- opencode harness work
