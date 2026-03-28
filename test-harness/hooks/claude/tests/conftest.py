from pathlib import Path

import pytest

from test_harness_imports import EXPECTED_HOOKS


@pytest.fixture(scope="session")
def claude_root() -> Path:
    return Path(__file__).resolve().parent.parent


@pytest.fixture(scope="session")
def expected_hooks() -> dict[str, str]:
    return EXPECTED_HOOKS
