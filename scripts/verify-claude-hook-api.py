#!/usr/bin/env python3
"""Verify whether a supported AI CLI version has changed since the last stored hook-schema run."""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class ToolSpec:
    tool: str
    executable: str
    history_subdir: tuple[str, ...]
    version_field: str


TOOL_SPECS: dict[str, ToolSpec] = {
    "claude": ToolSpec(
        tool="claude",
        executable="claude",
        history_subdir=("test-harness", "hooks", "claude", "drift-history"),
        version_field="claude_version",
    ),
    "codex": ToolSpec(
        tool="codex",
        executable="codex",
        history_subdir=("test-harness", "hooks", "codex", "drift-history"),
        version_field="codex_version",
    ),
    "gemini": ToolSpec(
        tool="gemini",
        executable="gemini",
        history_subdir=("test-harness", "hooks", "gemini", "drift-history"),
        version_field="gemini_version",
    ),
    "opencode": ToolSpec(
        tool="opencode",
        executable="opencode",
        history_subdir=("test-harness", "hooks", "opencode", "drift-history"),
        version_field="opencode_version",
    ),
    "cursor-agent": ToolSpec(
        tool="cursor-agent",
        executable="cursor-agent",
        history_subdir=("test-harness", "hooks", "cursor-agent", "drift-history"),
        version_field="cursor_agent_version",
    ),
}


def repo_root_from_script() -> Path:
    return Path(__file__).resolve().parent.parent


def latest_drift_record(history_dir: Path) -> Path | None:
    candidates = sorted(history_dir.glob("*.json"))
    if not candidates:
        return None
    return candidates[-1]


def read_recorded_version(spec: ToolSpec, repo_root: Path) -> tuple[str | None, Path | None]:
    history_dir = repo_root.joinpath(*spec.history_subdir)
    record_path = latest_drift_record(history_dir)
    if record_path is None:
        return None, None

    try:
        payload = json.loads(record_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise SystemExit(
            f"{spec.tool}: invalid JSON in drift-history record {record_path}: {exc.msg}"
        ) from exc
    value = payload.get(spec.version_field)
    if value is None:
        return None, record_path
    if not isinstance(value, str):
        raise SystemExit(
            f"{spec.tool}: expected string field {spec.version_field!r} in {record_path}, got {type(value).__name__}"
        )
    return value.strip() or None, record_path


def read_current_version(spec: ToolSpec) -> str | None:
    if shutil.which(spec.executable) is None:
        return None

    result = subprocess.run(
        [spec.executable, "--version"],
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        return None
    version = result.stdout.strip()
    if version:
        return version
    stderr = result.stderr.strip()
    return stderr or None


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--tool",
        default="claude",
        choices=sorted(TOOL_SPECS),
        help="tool executable/provider to verify",
    )
    parser.add_argument(
        "--repo-root",
        type=Path,
        default=repo_root_from_script(),
        help="repo root to inspect (defaults to the script's parent repo root)",
    )
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    spec = TOOL_SPECS[args.tool]
    repo_root = args.repo_root.resolve()

    recorded_version, record_path = read_recorded_version(spec, repo_root)
    current_version = read_current_version(spec)

    if current_version is None:
        print(
            f"{spec.tool}: executable `{spec.executable}` is unavailable or did not return a version; skipping version-bump check",
            file=sys.stderr,
        )
        return 0

    if recorded_version is None:
        record_note = f" (latest record: {record_path})" if record_path else ""
        print(
            f"{spec.tool}: no recorded version found in {repo_root.joinpath(*spec.history_subdir)}{record_note}; rerun hook-schema capture before enforcing bumps",
            file=sys.stderr,
        )
        return 0

    if current_version == recorded_version:
        location = str(record_path) if record_path else "<unknown>"
        print(
            f"{spec.tool}: version matches recorded drift-history value `{recorded_version}` from {location}"
        )
        return 0

    location = str(record_path) if record_path else "<unknown>"
    print(
        f"{spec.tool}: version bump detected (recorded `{recorded_version}` in {location}, current `{current_version}`); rerun the hook-schema validation flow",
        file=sys.stderr,
    )
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
