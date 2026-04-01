import json
import os
import stat
import subprocess
import sys
from pathlib import Path

import pytest


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
            f"{tool.replace('-', '_')}_version": recorded_version,
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
def test_version_detector_returns_zero_when_recorded_version_matches(tmp_path: Path) -> None:
    repo_root = _make_repo_root(tmp_path, "claude", "2.1.87 (Claude Code)")
    bin_dir = tmp_path / "bin"
    bin_dir.mkdir()
    _write_executable(
        bin_dir / "claude",
        "#!/bin/sh\nprintf '%s\\n' '2.1.87 (Claude Code)'\n",
    )

    script = Path(__file__).resolve().parents[4] / "scripts" / "verify-claude-hook-api.py"
    result = _run_detector(repo_root, script, "claude", str(bin_dir))

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

    script = Path(__file__).resolve().parents[4] / "scripts" / "verify-claude-hook-api.py"
    result = _run_detector(repo_root, script, "claude", str(bin_dir))

    assert result.returncode == 1
    assert "version bump detected" in result.stderr


@pytest.mark.provider_claude
def test_version_detector_skips_gracefully_when_tool_unavailable(tmp_path: Path) -> None:
    repo_root = _make_repo_root(tmp_path, "codex", None)
    empty_bin = tmp_path / "empty-bin"
    empty_bin.mkdir()
    script = Path(__file__).resolve().parents[4] / "scripts" / "verify-claude-hook-api.py"
    result = _run_detector(repo_root, script, "codex", str(empty_bin))

    assert result.returncode == 0
    assert "unavailable" in result.stderr
