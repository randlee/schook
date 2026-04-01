import json
import os
import stat
import subprocess
import sys
from pathlib import Path

import pytest


def _script_path() -> Path:
    for parent in Path(__file__).resolve().parents:
        candidate = parent / "scripts" / "verify-claude-hook-api.py"
        if candidate.exists():
            return candidate
    raise AssertionError("could not locate scripts/verify-claude-hook-api.py from test path")


def _version_field(tool: str) -> str:
    return f"{tool.replace('-', '_')}_version"


def _write_executable(path: Path, body: str) -> None:
    path.write_text(body, encoding="utf-8")
    path.chmod(path.stat().st_mode | stat.S_IEXEC)


def _make_repo_root(tmp_path: Path, tool: str, recorded_version: str | None) -> Path:
    repo_root = tmp_path / "repo"
    history_dir = repo_root / "test-harness" / "hooks" / tool / "drift-history"
    history_dir.mkdir(parents=True, exist_ok=True)
    if recorded_version is not None:
        payload = {
            "provider": tool,
            "run_timestamp": "20260331T184049.573585Z",
            "status": "PASS",
            "entries": [],
            _version_field(tool): recorded_version,
        }
        (history_dir / "20260331T184049.573585Z-drift.json").write_text(
            json.dumps(payload, indent=2) + "\n",
            encoding="utf-8",
        )
    return repo_root


def _run_detector(repo_root: Path, script: Path, tool: str, path_env: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, str(script), "--tool", tool, "--repo-root", str(repo_root)],
        text=True,
        capture_output=True,
        env={**os.environ, "PATH": path_env},
        check=False,
    )


@pytest.mark.provider_claude
@pytest.mark.parametrize(
    ("tool", "recorded_version"),
    [
        ("claude", "2.1.87 (Claude Code)"),
        ("codex", "codex-cli 0.117.0"),
        ("gemini", "gemini-cli 0.1.0"),
        ("opencode", "opencode 0.9.0"),
        ("cursor-agent", "cursor-agent 1.2.3"),
    ],
)
def test_version_detector_returns_zero_when_recorded_version_matches(
    tmp_path: Path,
    tool: str,
    recorded_version: str,
) -> None:
    repo_root = _make_repo_root(tmp_path, tool, recorded_version)
    bin_dir = tmp_path / "bin"
    bin_dir.mkdir()
    _write_executable(
        bin_dir / tool,
        f"#!/bin/sh\nprintf '%s\\n' '{recorded_version}'\n",
    )

    script = _script_path()
    result = _run_detector(repo_root, script, tool, str(bin_dir))

    assert result.returncode == 0, result.stderr
    assert "version matches recorded drift-history value" in result.stdout


@pytest.mark.provider_claude
def test_version_detector_returns_one_on_bump(tmp_path: Path) -> None:
    repo_root = _make_repo_root(tmp_path, "claude", "2.1.86 (Claude Code)")
    bin_dir = tmp_path / "bin"
    bin_dir.mkdir()
    _write_executable(
        bin_dir / "claude",
        "#!/bin/sh\nprintf '%s\\n' '2.1.87 (Claude Code)'\n",
    )

    script = _script_path()
    result = _run_detector(repo_root, script, "claude", str(bin_dir))

    assert result.returncode == 1
    assert "version bump detected" in result.stderr


@pytest.mark.provider_claude
def test_version_detector_skips_gracefully_when_tool_unavailable(tmp_path: Path) -> None:
    repo_root = _make_repo_root(tmp_path, "codex", None)
    empty_bin = tmp_path / "empty-bin"
    empty_bin.mkdir()
    script = _script_path()
    result = _run_detector(repo_root, script, "codex", str(empty_bin))

    assert result.returncode == 0
    assert "unavailable" in result.stderr


@pytest.mark.provider_claude
def test_version_detector_reports_invalid_drift_history_json(tmp_path: Path) -> None:
    repo_root = tmp_path / "repo"
    history_dir = repo_root / "test-harness" / "hooks" / "gemini" / "drift-history"
    history_dir.mkdir(parents=True, exist_ok=True)
    (history_dir / "20260331T184049.573585Z-drift.json").write_text(
        "{invalid json\n",
        encoding="utf-8",
    )
    bin_dir = tmp_path / "bin"
    bin_dir.mkdir()
    _write_executable(
        bin_dir / "gemini",
        "#!/bin/sh\nprintf '%s\\n' 'gemini-cli 0.1.0'\n",
    )

    script = _script_path()
    result = _run_detector(repo_root, script, "gemini", str(bin_dir))

    assert result.returncode == 1
    assert "invalid JSON in drift-history record" in result.stderr
