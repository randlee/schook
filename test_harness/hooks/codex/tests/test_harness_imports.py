"""Import indirection for Codex harness tests without package installation."""

from pathlib import Path
import runpy


_registry = runpy.run_path(
    str(Path(__file__).resolve().parents[4] / "test-harness" / "hooks" / "codex" / "models" / "registry.py")
)

EXPECTED_HOOKS = _registry["EXPECTED_HOOKS"]
