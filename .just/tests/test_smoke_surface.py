from __future__ import annotations

import contextlib
import importlib.util
import io
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


class SmokeSurfaceTests(unittest.TestCase):
    def test_discover_provider_modules_ignores_private_and_init_files(self) -> None:
        mod = load_module("run_smoke", ".just/run_smoke.py")
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            (root / "__init__.py").write_text("", encoding="utf-8")
            (root / "_private.py").write_text("", encoding="utf-8")
            (root / "claude.py").write_text("", encoding="utf-8")
            (root / "cursor_agent.py").write_text("", encoding="utf-8")
            modules = mod.discover_provider_modules(root)
        self.assertEqual(sorted(modules), ["claude", "cursor-agent"])

    def test_main_returns_zero_when_no_provider_modules_exist(self) -> None:
        mod = load_module("run_smoke", ".just/run_smoke.py")
        with tempfile.TemporaryDirectory() as tmp_dir:
            repo_root = Path(tmp_dir)
            smoke_root = repo_root / ".just" / "smoke"
            smoke_root.mkdir(parents=True)
            capture = io.StringIO()
            original_resolve = Path.resolve

            def fake_resolve(path_self: Path, *args, **kwargs):
                if path_self == Path(mod.__file__):
                    return repo_root / ".just" / "run_smoke.py"
                return original_resolve(path_self, *args, **kwargs)

            with contextlib.redirect_stdout(capture), mock.patch.object(
                Path, "resolve", fake_resolve
            ):
                code = mod.main(["run_smoke.py", "all", "--mode", "ci"])
        self.assertEqual(code, 0)
        self.assertIn("discovered=0 executed=0", capture.getvalue())

    def test_unknown_provider_raises_system_exit(self) -> None:
        mod = load_module("run_smoke", ".just/run_smoke.py")
        with tempfile.TemporaryDirectory() as tmp_dir:
            repo_root = Path(tmp_dir)
            smoke_root = repo_root / ".just" / "smoke"
            smoke_root.mkdir(parents=True)
            (smoke_root / "claude.py").write_text("def run(*, mode, repo_root):\n    return 0\n", encoding="utf-8")
            original_resolve = Path.resolve

            def fake_resolve(path_self: Path, *args, **kwargs):
                if path_self == Path(mod.__file__):
                    return repo_root / ".just" / "run_smoke.py"
                return original_resolve(path_self, *args, **kwargs)

            with mock.patch.object(Path, "resolve", fake_resolve):
                with self.assertRaises(SystemExit) as exc:
                    mod.main(["run_smoke.py", "unknown", "--mode", "ci"])
        self.assertIn("unknown smoke provider: unknown", str(exc.exception))


if __name__ == "__main__":
    unittest.main()
