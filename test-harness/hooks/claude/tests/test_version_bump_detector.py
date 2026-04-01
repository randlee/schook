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


def _make_repo_root(tmp_path: Path, claude_version: str) -> Path:
    repo_root = tmp_path / "repo"
    manifest = repo_root / "test-harness" / "hooks" / "claude" / "fixtures" / "approved"
    manifest.mkdir(parents=True, exist_ok=True)
    (manifest / "manifest.json").write_text(
        json.dumps({"provider": "claude", "status": "test", "claude_version": claude_version}, indent=2) + "\n",
        encoding="utf-8",
    )
    return repo_root


def _script_path() -> Path:
    for parent in Path(__file__).resolve().parents:
        candidate = parent / "scripts" / "verify-claude-hook-api.py"
        if candidate.exists():
            return candidate
    raise AssertionError("could not locate scripts/verify-claude-hook-api.py from test path")


def _run_detector(repo_root: Path, path_env: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, str(_script_path())],
        text=True,
        capture_output=True,
        env={**os.environ, "PATH": path_env, "SC_HOOK_REPO_ROOT": str(repo_root)},
        cwd=repo_root,
        check=False,
    )


@pytest.mark.skipif(sys.platform == "win32", reason="POSIX shell stubs not supported on Windows")
def test_version_detector_returns_zero_when_manifest_matches(tmp_path: Path) -> None:
    repo_root = _make_repo_root(tmp_path, "2.1.87 (Claude Code)")
    bin_dir = tmp_path / "bin"
    bin_dir.mkdir()
    _write_executable(bin_dir / "claude", "#!/bin/sh\nprintf '%s\\n' '2.1.87 (Claude Code)'\n")
    result = _run_detector(repo_root, str(bin_dir))
    assert result.returncode == 0, result.stderr
    assert "matches approved manifest" in result.stdout


@pytest.mark.skipif(sys.platform == "win32", reason="POSIX shell stubs not supported on Windows")
def test_version_detector_returns_one_on_bump(tmp_path: Path) -> None:
    repo_root = _make_repo_root(tmp_path, "2.1.87 (Claude Code)")
    bin_dir = tmp_path / "bin"
    bin_dir.mkdir()
    _write_executable(bin_dir / "claude", "#!/bin/sh\nprintf '%s\\n' '2.1.89 (Claude Code)'\n")
    result = _run_detector(repo_root, str(bin_dir))
    assert result.returncode == 1
    assert "version changed" in result.stderr


@pytest.mark.skipif(sys.platform == "win32", reason="POSIX shell stubs not supported on Windows")
def test_version_detector_fails_when_manifest_version_missing(tmp_path: Path) -> None:
    repo_root = _make_repo_root(tmp_path, "")
    bin_dir = tmp_path / "bin"
    bin_dir.mkdir()
    _write_executable(bin_dir / "claude", "#!/bin/sh\nprintf '%s\\n' '2.1.87 (Claude Code)'\n")
    result = _run_detector(repo_root, str(bin_dir))
    assert result.returncode == 1
    assert '"error": "missing_claude_version"' in result.stderr


def test_version_detector_reports_invalid_manifest_json(tmp_path: Path) -> None:
    repo_root = tmp_path / "repo"
    manifest = repo_root / "test-harness" / "hooks" / "claude" / "fixtures" / "approved"
    manifest.mkdir(parents=True, exist_ok=True)
    (manifest / "manifest.json").write_text("{ invalid json\n", encoding="utf-8")
    result = _run_detector(repo_root, os.environ.get("PATH", ""))
    assert result.returncode == 1
    assert '"error": "invalid_manifest_json"' in result.stderr
    assert "Traceback" not in result.stderr


def test_version_detector_reports_missing_manifest(tmp_path: Path) -> None:
    repo_root = tmp_path / "repo"
    repo_root.mkdir()
    result = _run_detector(repo_root, os.environ.get("PATH", ""))
    assert result.returncode == 1
    assert '"error": "manifest_not_found"' in result.stderr
    assert "Traceback" not in result.stderr


def test_version_detector_reports_missing_claude_binary(tmp_path: Path) -> None:
    repo_root = _make_repo_root(tmp_path, "2.1.87 (Claude Code)")
    result = _run_detector(repo_root, "")
    assert result.returncode == 1
    assert '"error": "claude_version_exec_failed"' in result.stderr
    assert "Traceback" not in result.stderr
