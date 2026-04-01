#!/usr/bin/env bash
set -euo pipefail

fail() {
  echo "release-gate: FAIL - $*" >&2
  exit 1
}

info() {
  echo "release-gate: $*"
}

info "fetching origin refs and tags"
git fetch origin --prune --tags >/dev/null 2>&1 || fail "git fetch failed"

branch="$(git rev-parse --abbrev-ref HEAD)"
[[ "$branch" == "main" ]] || fail "release gate must run from main (got $branch)"

git diff --quiet || fail "working tree has unstaged changes"
git diff --cached --quiet || fail "working tree has staged changes"

workspace_version="$(python3 - <<'PY'
import tomllib
from pathlib import Path

data = tomllib.loads(Path("Cargo.toml").read_text(encoding="utf-8"))
print(data["workspace"]["package"]["version"])
PY
)"

info "workspace version: ${workspace_version}"

while IFS= read -r cargo_toml; do
  [[ -n "$cargo_toml" ]] || continue
  python3 - <<'PY' "$workspace_version" "$cargo_toml"
import sys
import tomllib
from pathlib import Path

workspace_version = sys.argv[1]
cargo_toml = Path(sys.argv[2])
data = tomllib.loads(cargo_toml.read_text(encoding="utf-8"))
version = data.get("package", {}).get("version")
if isinstance(version, str):
    if version != workspace_version:
        raise SystemExit(f"{cargo_toml}: version mismatch: {version} != {workspace_version}")
elif isinstance(version, dict):
    if version.get("workspace") is not True:
        raise SystemExit(f"{cargo_toml}: expected workspace version reference")
else:
    raise SystemExit(f"{cargo_toml}: missing or unsupported package.version")
PY
done < <(python3 scripts/release_artifacts.py list-cargo-tomls --manifest release/publish-artifacts.toml)

info "PASS - main branch, clean tree, and manifest versions are aligned"
