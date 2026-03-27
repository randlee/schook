"""Import indirection for harness tests without needing package installation."""

from pathlib import Path
import runpy


_registry = runpy.run_path(
    str(Path(__file__).resolve().parent.parent / "models" / "registry.py")
)

EXPECTED_HOOKS = _registry["EXPECTED_HOOKS"]
