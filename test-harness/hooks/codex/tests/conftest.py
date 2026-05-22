from pathlib import Path
import sys

import pytest


REPO_ROOT = Path(__file__).resolve().parents[4]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from test_harness.hooks.paths import CODEX_ROOT


@pytest.fixture(scope="session")
def codex_root() -> Path:
    return CODEX_ROOT
