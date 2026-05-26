#!/usr/bin/env python3
from __future__ import annotations

import argparse
import importlib.util
import subprocess
import sys
from pathlib import Path


def discover_provider_modules(smoke_root: Path) -> dict[str, Path]:
    modules: dict[str, Path] = {}
    for path in sorted(smoke_root.glob("*.py")):
        if path.name == "__init__.py" or path.name.startswith("_"):
            continue
        modules[path.stem.replace("_", "-")] = path
    return modules


def load_module(module_path: Path):
    spec = importlib.util.spec_from_file_location(module_path.stem, module_path)
    if spec is None or spec.loader is None:
        raise ImportError(f"unable to load smoke module from {module_path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def run_provider(module_path: Path, mode: str, repo_root: Path) -> int:
    module = load_module(module_path)
    runner = getattr(module, "run", None)
    if runner is None:
        raise SystemExit(f"{module_path} must define run(mode: str, repo_root: Path) -> int")
    return int(runner(mode=mode, repo_root=repo_root))


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description="Run the curated smoke surface.")
    parser.add_argument("provider", help="provider name or 'all'")
    parser.add_argument("--mode", default="live", choices=("live", "ci"))
    args = parser.parse_args(argv[1:])

    repo_root = Path(__file__).resolve().parent.parent
    smoke_root = repo_root / ".just" / "smoke"
    modules = discover_provider_modules(smoke_root)

    if args.provider == "all":
        if not modules:
            print(f"smoke: mode={args.mode} providers=all discovered=0 executed=0")
            return 0

        executed = 0
        for provider, module_path in modules.items():
            print(f"smoke: running provider={provider} mode={args.mode}")
            code = run_provider(module_path, args.mode, repo_root)
            if code != 0:
                return code
            executed += 1
        print(
            f"smoke: mode={args.mode} providers=all discovered={len(modules)} executed={executed}"
        )
        return 0

    module_path = modules.get(args.provider)
    if module_path is None:
        discovered = ", ".join(sorted(modules)) if modules else "<none>"
        raise SystemExit(
            f"unknown smoke provider: {args.provider}; discovered providers: {discovered}"
        )

    return run_provider(module_path, args.mode, repo_root)


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
