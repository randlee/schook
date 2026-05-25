#!/usr/bin/env python3
from __future__ import annotations

from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
import argparse
import subprocess
import sys


CARGO_LINT_ORDER = ("fmt", "clippy", "modules", "deny", "shear")
PYTHON_LINT_ORDER = ("version", "manifests", "spell", "pytests")
EXTRA_LINTS = ("boundary", "portability")

def build_tasks(repo_root: Path) -> dict[str, list[str]]:
    python_executable = sys.executable
    return {
        "fmt": ["just", "_lint-fmt"],
        "clippy": ["just", "_lint-clippy"],
        "modules": ["just", "_lint-modules"],
        "deny": ["just", "_lint-deny"],
        "shear": ["just", "_lint-shear"],
        "version": [python_executable, str(repo_root / ".just/check_version_sync.py")],
        "manifests": [python_executable, str(repo_root / ".just/lint_manifests.py")],
        "spell": [python_executable, str(repo_root / ".just/lint_codespell.py")],
        "pytests": [python_executable, str(repo_root / ".just/run_pytests.py")],
        "boundary": [python_executable, str(repo_root / ".just/lint_sc_boundary.py")],
        "portability": [python_executable, str(repo_root / ".just/lint_sc_portability.py")],
    }


def resolve_task_names(target: str) -> list[str]:
    if target == "all":
        return [*CARGO_LINT_ORDER, *PYTHON_LINT_ORDER]
    valid = {"all", *CARGO_LINT_ORDER, *PYTHON_LINT_ORDER, *EXTRA_LINTS}
    if target not in valid:
        valid_display = ", ".join(sorted(valid))
        raise ValueError(f"unknown lint target: {target}; expected one of: {valid_display}")
    return [target]


def run_task(command: list[str], repo_root: Path) -> int:
    completed = subprocess.run(command, cwd=repo_root)
    return completed.returncode


def run_parallel(tasks: list[list[str]], repo_root: Path) -> list[int]:
    with ThreadPoolExecutor(max_workers=len(tasks)) as executor:
        futures = [executor.submit(run_task, task, repo_root) for task in tasks]
        return [future.result() for future in futures]


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description="Run repo lint targets.")
    parser.add_argument("target", nargs="?", default="all")
    args = parser.parse_args(argv[1:])

    repo_root = Path(__file__).resolve().parent.parent
    tasks = build_tasks(repo_root)
    try:
        task_names = resolve_task_names(args.target)
    except ValueError as error:
        print(str(error), file=sys.stderr)
        return 2

    if args.target == "all":
        for name in CARGO_LINT_ORDER:
            returncode = run_task(tasks[name], repo_root)
            if returncode != 0:
                return returncode
        python_results = run_parallel([tasks[name] for name in PYTHON_LINT_ORDER], repo_root)
        for returncode in python_results:
            if returncode != 0:
                return returncode
        return 0

    task = tasks[task_names[0]]
    return run_task(task, repo_root)


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
