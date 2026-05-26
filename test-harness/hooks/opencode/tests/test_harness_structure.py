from pathlib import Path

import pytest


@pytest.mark.provider_opencode
def test_required_opencode_subdirectories_exist(opencode_root: Path) -> None:
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
        assert (opencode_root / name).is_dir(), name
    assert (opencode_root / "captures" / "raw").is_dir(), "captures/raw"


@pytest.mark.provider_opencode
def test_paired_opencode_package_root_exists(opencode_root: Path) -> None:
    package_root = opencode_root.parents[2] / "test_harness" / "hooks" / "opencode"
    assert package_root.is_dir()
    assert (package_root / "__init__.py").is_file()
    assert (package_root / "models").is_dir()
