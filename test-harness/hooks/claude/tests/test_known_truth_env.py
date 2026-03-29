import json
from pathlib import Path


BASELINE_CLAUDE_ENV = {
    "CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS",
    "CLAUDE_CODE_TASK_LIST_ID",
    "CLAUDE_MCP_CONFIG",
    "CLAUDE_TEMPLATES_ROOT",
}


def _capture_root() -> Path:
    return Path(__file__).resolve().parents[1] / "captures" / "raw"


def _load_capture(stem: str) -> tuple[dict, dict]:
    capture_root = _capture_root()
    payload = json.loads((capture_root / f"{stem}.json").read_text(encoding="utf-8"))
    env = json.loads((capture_root / f"{stem}.env.json").read_text(encoding="utf-8"))
    return payload, env


def test_startup_capture_aligns_root_and_claude_project_dir() -> None:
    payload, env = _load_capture("20260329T203144.187831Z-session-start")

    assert payload["hook_event_name"] == "SessionStart"
    assert payload["source"] == "startup"
    assert payload["cwd"] == env["cwd_from_getcwd"] == env["pwd_env"]
    assert env["claude_env"]["CLAUDE_PROJECT_DIR"] == payload["cwd"]
    assert env["atm_env"]["ATM_IDENTITY"] == "chook"
    assert env["atm_env"]["ATM_TEAM"] == "atm-dev"
    assert env["claude_env"].get("CLAUDE_PLUGIN_ROOT") is None


def test_cwd_can_drift_while_claude_project_dir_stays_on_startup_root() -> None:
    startup_payload, startup_env = _load_capture("20260329T203144.187831Z-session-start")
    post_bash_payload, post_bash_env = _load_capture("20260329T203144.881734Z-posttooluse-bash")
    stop_payload, stop_env = _load_capture("20260329T203149.133073Z-stop")
    session_end_payload, session_end_env = _load_capture("20260329T203149.289632Z-session-end")

    startup_root = startup_payload["cwd"]
    drifted_dir = "/Users/randlee/Documents/github/schook-worktrees/feature-s9-hook-env-capture/test-harness/hooks/claude"

    assert post_bash_payload["cwd"] == drifted_dir
    assert post_bash_env["cwd_from_getcwd"] == drifted_dir
    assert post_bash_env["claude_env"]["CLAUDE_PROJECT_DIR"] == startup_root

    assert stop_payload["cwd"] == drifted_dir
    assert stop_env["cwd_from_getcwd"] == drifted_dir
    assert stop_env["claude_env"]["CLAUDE_PROJECT_DIR"] == startup_root

    assert session_end_payload["cwd"] == drifted_dir
    assert session_end_env["cwd_from_getcwd"] == drifted_dir
    assert session_end_env["claude_env"]["CLAUDE_PROJECT_DIR"] == startup_root

    assert startup_env["claude_env"]["CLAUDE_PROJECT_DIR"] == startup_root
    assert startup_root != drifted_dir


def test_agent_capture_preserves_project_and_atm_context() -> None:
    startup_payload, startup_env = _load_capture("20260329T203354.313004Z-session-start")
    payload, env = _load_capture("20260329T203357.612677Z-pretooluse-agent")

    assert payload["hook_event_name"] == "PreToolUse"
    assert payload["tool_name"] == "Agent"
    assert payload["tool_input"]["run_in_background"] is True
    assert payload["tool_input"]["subagent_type"] == "general-purpose"
    assert env["claude_env"]["CLAUDE_PROJECT_DIR"] == startup_payload["cwd"]
    assert env["atm_env"]["ATM_IDENTITY"] == "chook"
    assert env["atm_env"]["ATM_TEAM"] == "atm-dev"
    assert env["claude_env"].get("CLAUDE_PLUGIN_ROOT") is None
    assert startup_env["claude_env"]["CLAUDE_PROJECT_DIR"] == startup_payload["cwd"]


def test_resume_capture_preserves_session_id_and_project_dir() -> None:
    startup_payload, startup_env = _load_capture("20260329T204830.666482Z-session-start")
    resume_payload, resume_env = _load_capture("20260329T204833.204858Z-session-start")

    assert startup_payload["hook_event_name"] == "SessionStart"
    assert startup_payload["source"] == "startup"
    assert resume_payload["hook_event_name"] == "SessionStart"
    assert resume_payload["source"] == "resume"
    assert resume_payload["session_id"] == startup_payload["session_id"]
    assert resume_payload["cwd"] == startup_payload["cwd"]
    assert startup_env["claude_env"]["CLAUDE_PROJECT_DIR"] == startup_payload["cwd"]
    assert resume_env["claude_env"]["CLAUDE_PROJECT_DIR"] == startup_payload["cwd"]
    assert resume_env["atm_env"]["ATM_IDENTITY"] == "chook"
    assert resume_env["atm_env"]["ATM_TEAM"] == "atm-dev"


def test_compact_and_clear_captures_preserve_project_dir_without_implicit_atm_env() -> None:
    pre_compact_payload, pre_compact_env = _load_capture("20260329T211418.974201Z-pre-compact")
    compact_payload, compact_env = _load_capture("20260329T211435.352660Z-session-start")
    clear_end_payload, clear_end_env = _load_capture("20260329T211532.933293Z-session-end")
    clear_start_payload, clear_start_env = _load_capture("20260329T211532.964966Z-session-start")

    project_root = "/Users/randlee/Documents/github/schook-worktrees/feature-s9-hook-env-capture"

    assert pre_compact_payload["hook_event_name"] == "PreCompact"
    assert pre_compact_payload["cwd"] == project_root
    assert pre_compact_env["claude_env"]["CLAUDE_PROJECT_DIR"] == project_root
    assert pre_compact_env["atm_env"].get("ATM_IDENTITY") is None
    assert pre_compact_env["atm_env"].get("ATM_TEAM") is None

    assert compact_payload["hook_event_name"] == "SessionStart"
    assert compact_payload["source"] == "compact"
    assert compact_payload["cwd"] == project_root
    assert compact_env["claude_env"]["CLAUDE_PROJECT_DIR"] == project_root
    assert compact_env["atm_env"].get("ATM_IDENTITY") is None
    assert compact_env["atm_env"].get("ATM_TEAM") is None

    assert clear_end_payload["hook_event_name"] == "SessionEnd"
    assert clear_end_payload["reason"] == "clear"
    assert clear_end_env["claude_env"]["CLAUDE_PROJECT_DIR"] == project_root
    assert clear_end_env["atm_env"].get("ATM_IDENTITY") is None
    assert clear_end_env["atm_env"].get("ATM_TEAM") is None

    assert clear_start_payload["hook_event_name"] == "SessionStart"
    assert clear_start_payload["source"] == "clear"
    assert clear_start_payload["cwd"] == project_root
    assert clear_start_env["claude_env"]["CLAUDE_PROJECT_DIR"] == project_root
    assert clear_start_env["atm_env"].get("ATM_IDENTITY") is None
    assert clear_start_env["atm_env"].get("ATM_TEAM") is None


def test_sensitive_atm_values_are_redacted_in_committed_env_snapshots() -> None:
    _, env = _load_capture("20260329T203354.313004Z-session-start")

    for key, value in env["atm_env"].items():
        if "TOKEN" in key or "AUTH" in key:
            assert value == "<redacted>"


def test_claude_env_baseline_diff_summary_is_stable_for_current_capture_set() -> None:
    _, env = _load_capture("20260329T203354.313004Z-session-start")
    observed = set(env["claude_env"])

    assert BASELINE_CLAUDE_ENV <= observed
    assert {
        "CLAUDE_CODE_ENTRYPOINT",
        "CLAUDE_ENV_FILE",
        "CLAUDE_PROJECT_DIR",
    } <= observed
    assert "CLAUDE_PROJECT_DIR" not in BASELINE_CLAUDE_ENV
