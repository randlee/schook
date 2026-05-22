from pathlib import Path

import pytest


@pytest.mark.provider_gemini
def test_required_gemini_subdirectories_exist(gemini_root: Path) -> None:
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
        assert (gemini_root / name).is_dir(), name


@pytest.mark.provider_gemini
def test_canned_prompts_exist_for_expected_runs(gemini_root: Path) -> None:
    prompt_dir = gemini_root / "prompts"
    expected = [
        "lifecycle.md",
        "before-agent.md",
        "before-tool.md",
        "resume.md",
        "output-formats.md",
    ]
    for filename in expected:
        assert (prompt_dir / filename).is_file(), filename
