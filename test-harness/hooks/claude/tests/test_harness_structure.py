from pathlib import Path

import pytest


@pytest.mark.provider_claude
def test_required_claude_subdirectories_exist(claude_root: Path) -> None:
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
        assert (claude_root / name).is_dir(), name


@pytest.mark.provider_claude
def test_canned_prompts_exist_for_each_surface(claude_root: Path) -> None:
    prompt_dir = claude_root / "prompts"
    expected = [
        "session-start.md",
        "session-end.md",
        "pretooluse-bash.md",
        "posttooluse-bash.md",
        "pretooluse-agent.md",
        "permission-request.md",
        "stop.md",
    ]
    for filename in expected:
        assert (prompt_dir / filename).is_file(), filename
