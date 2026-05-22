"""Import indirection for harness tests without requiring package installation."""

from pathlib import Path
import runpy


_registry = runpy.run_path(
    str(Path(__file__).resolve().parents[4] / "test-harness" / "hooks" / "claude" / "models" / "registry.py")
)

EXPECTED_HOOKS = _registry["EXPECTED_HOOKS"]
