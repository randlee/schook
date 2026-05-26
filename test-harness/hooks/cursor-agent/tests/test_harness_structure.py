from pathlib import Path

import pytest


@pytest.mark.provider_cursor_agent
def test_required_cursor_agent_subdirectories_exist(cursor_agent_root: Path) -> None:
    required = [
        "prompts",
        "hooks",
        "models",
        "schema",
        "fixtures",
        "captures",
        "reports",
        "scripts",
        "tests",
    ]
    for name in required:
        assert (cursor_agent_root / name).is_dir(), name
    assert (cursor_agent_root / "captures" / "raw").is_dir(), "captures/raw"


@pytest.mark.provider_cursor_agent
def test_paired_cursor_agent_package_root_exists(cursor_agent_root: Path) -> None:
    package_root = cursor_agent_root.parents[2] / "test_harness" / "hooks" / "cursor_agent"
    assert package_root.is_dir()
    assert (package_root / "__init__.py").is_file()
    assert (package_root / "models").is_dir()
