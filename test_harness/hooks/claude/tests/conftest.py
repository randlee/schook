from pathlib import Path

import pytest

from test_harness.hooks.paths import CLAUDE_ROOT


@pytest.fixture(scope="session")
def claude_root() -> Path:
    return CLAUDE_ROOT
