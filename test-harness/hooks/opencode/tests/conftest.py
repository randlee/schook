from pathlib import Path

import pytest


@pytest.fixture(scope="session")
def opencode_root() -> Path:
    return Path(__file__).resolve().parent.parent


@pytest.fixture(scope="session")
def expected_surfaces() -> list[str]:
    return ["session.idle"]
