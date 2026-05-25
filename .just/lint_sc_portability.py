#!/usr/bin/env python3
from __future__ import annotations

import json
import shutil
import subprocess
import sys
from pathlib import Path


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def sc_lint_repo_candidates(root: Path) -> list[Path]:
    return [
        root.parent / "sc-lint",
        root.parent.parent / "sc-lint",
        root.parent.parent.parent / "sc-lint",
    ]


def resolve_backend(root: Path) -> tuple[list[str], str]:
    installed = shutil.which("sc-lint-portability")
    if installed:
        return [installed], f"path:{installed}"

    for candidate in sc_lint_repo_candidates(root):
        cargo_toml = candidate / "Cargo.toml"
        if cargo_toml.exists():
            return (
                [
                    "cargo",
                    "run",
                    "-q",
                    "--manifest-path",
                    str(cargo_toml),
                    "-p",
                    "sc-lint-portability",
                    "--",
                ],
                f"repo-local:{candidate}",
            )

    raise SystemExit(
        "sc-lint-portability unavailable: install the CLI or provide the ../sc-lint fallback"
    )


def analyze(command_prefix: list[str], target_root: Path) -> dict:
    command = [
        *command_prefix,
        "analyze",
        "--root",
        str(target_root),
        "--format",
        "json",
    ]
    completed = subprocess.run(
        command,
        cwd=repo_root(),
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if completed.returncode != 0:
        detail = completed.stderr.strip() or completed.stdout.strip() or "unknown failure"
        raise SystemExit(f"sc-lint-portability execution failed: {detail}")
    return json.loads(completed.stdout)


def main() -> int:
    root = repo_root()
    command_prefix, backend_label = resolve_backend(root)
    payload = analyze(command_prefix, root)
    findings = payload.get("findings", [])
    status = payload.get("status", "unknown")
    print(f"sc-lint-portability backend={backend_label} status={status} findings={len(findings)}")
    for finding in findings[:5]:
        rule_id = finding.get("rule_id", "UNKNOWN")
        message = finding.get("message", "").strip()
        print(f"- {rule_id}: {message}")
    return 0 if status == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
