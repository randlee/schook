from __future__ import annotations

import importlib.util
import tempfile
from pathlib import Path
import unittest
from unittest import mock


def load_module(name: str, relative_path: str):
    repo_root = Path(__file__).resolve().parents[2]
    path = repo_root / relative_path
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise ImportError(f"unable to load module from {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class ClaudeSmokeTests(unittest.TestCase):
    def test_env_path_uses_default_when_env_var_missing(self) -> None:
        mod = load_module("claude_smoke", ".just/smoke/claude.py")
        default = Path("/tmp/default-state")
        with mock.patch.dict("os.environ", {}, clear=False):
            self.assertEqual(mod._env_path("SC_HOOKS_STATE_DIR", default), default)

    def test_env_path_uses_override_when_env_var_present(self) -> None:
        mod = load_module("claude_smoke", ".just/smoke/claude.py")
        default = Path("/tmp/default-state")
        override = "/tmp/override-state"
        with mock.patch.dict("os.environ", {"SC_HOOKS_STATE_DIR": override}, clear=False):
            self.assertEqual(mod._env_path("SC_HOOKS_STATE_DIR", default), Path(override))

    def test_run_ci_accepts_current_fixture_contract(self) -> None:
        mod = load_module("claude_smoke", ".just/smoke/claude.py")
        repo_root = Path(__file__).resolve().parents[2]
        self.assertEqual(mod.run(mode="ci", repo_root=repo_root), 0)

    def test_run_ci_rejects_required_post_tool_event_drift(self) -> None:
        mod = load_module("claude_smoke", ".just/smoke/claude.py")
        repo_root = Path(__file__).resolve().parents[2]
        fixture = mod._load_fixture(repo_root)
        fixture["required_post_tool_event"] = "Write"
        with mock.patch.object(mod, "_load_fixture", return_value=fixture):
            with self.assertRaises(SystemExit) as err:
                mod.run(mode="ci", repo_root=repo_root)
        self.assertIn("required_post_tool_event drifted", str(err.exception))


if __name__ == "__main__":
    unittest.main()
