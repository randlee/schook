from pathlib import Path

import pytest


@pytest.fixture(scope="session")
def cursor_agent_root() -> Path:
    return Path(__file__).resolve().parent.parent


@pytest.fixture(scope="session")
def expected_surfaces() -> list[str]:
    return ["stop"]
