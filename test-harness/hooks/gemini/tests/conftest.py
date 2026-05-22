from pathlib import Path
import sys

import pytest

REPO_ROOT = Path(__file__).resolve().parents[4]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from test_harness.hooks.paths import GEMINI_ROOT


@pytest.fixture(scope="session")
def gemini_root() -> Path:
    return GEMINI_ROOT


@pytest.fixture(scope="session")
def expected_surfaces() -> list[str]:
    return [
        "before-agent",
        "before-tool",
        "after-tool",
        "after-agent",
        "session-start-startup",
        "session-start-resume",
        "session-end",
    ]
