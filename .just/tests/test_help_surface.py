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


class HelpSurfaceTests(unittest.TestCase):
    def test_print_help_lists_curated_lint_targets(self) -> None:
        mod = load_module("print_help", ".just/print_help.py")
        rendered = mod.render_help("schook")
        self.assertIn("lint modules", rendered)
        self.assertIn("lint deny", rendered)
        self.assertIn("lint shear", rendered)
        self.assertIn("lint manifests", rendered)
        self.assertIn("lint spell", rendered)
        self.assertIn("lint pytests", rendered)
        self.assertIn("lint boundary", rendered)
        self.assertIn("lint portability", rendered)

    def test_print_help_keeps_hook_test_entrypoints(self) -> None:
        mod = load_module("print_help", ".just/print_help.py")
        rendered = mod.render_help("schook")
        self.assertIn("test hooks claude", rendered)
        self.assertIn("test hooks codex", rendered)
        self.assertIn("test hooks gemini", rendered)


if __name__ == "__main__":
    unittest.main()
