from __future__ import annotations

import importlib.util
from pathlib import Path
import unittest


def load_module(name: str, relative_path: str):
    repo_root = Path(__file__).resolve().parents[2]
    path = repo_root / relative_path
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise ImportError(f"unable to load module from {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class LintSurfaceTests(unittest.TestCase):
    def test_run_lint_exports_phase_p_targets(self) -> None:
        mod = load_module("run_lint", ".just/run_lint.py")
        expected = {
            "fmt",
            "clippy",
            "modules",
            "deny",
            "shear",
            "version",
            "manifests",
            "spell",
            "pytests",
            "sc-boundary",
            "sc-portability",
        }
        self.assertTrue(expected.issubset(mod.TARGETS.keys()))

    def test_run_hook_tests_preserves_provider_surface(self) -> None:
        mod = load_module("run_hook_tests", ".just/run_hook_tests.py")
        self.assertEqual(set(mod.HOOK_TESTS.keys()), {"claude", "codex", "gemini"})


if __name__ == "__main__":
    unittest.main()
