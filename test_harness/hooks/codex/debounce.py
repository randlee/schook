from __future__ import annotations

import json
import os
import shlex
import subprocess
import sys
import time
from dataclasses import dataclass
from datetime import datetime, timedelta, timezone
from pathlib import Path
import tomllib
from typing import Any
from uuid import uuid4

_ENV_REDACTION_EXACT_KEYS = {
    "ATM_IDENTITY",
    "ATM_OTEL_ENABLED",
    "ATM_OTEL_PROTOCOL",
    "ATM_TEAM",
    "CODEX_CI",
    "CODEX_MANAGED_BY_NPM",
    "CODEX_MANAGED_PACKAGE_ROOT",
    "CODEX_PROJECT_DIR",
    "CODEX_THREAD_ID",
}

_ENV_REDACTION_SUBSTRINGS = (
    "AUTH",
    "HEADER",
    "KEY",
    "PASSWORD",
    "SECRET",
    "TOKEN",
)


def _utc_now() -> datetime:
    return datetime.now(timezone.utc)


def _state_root() -> Path:
    configured = os.environ.get("SCHOOK_CODEX_HOOK_STATE_ROOT", "").strip()
    if configured:
        root = Path(configured).expanduser().resolve()
    else:
        root = (Path(__file__).resolve().parents[3] / "test-harness" / "hooks" / "codex" / "state").resolve()
    root.mkdir(parents=True, exist_ok=True)
    return root


def pending_root() -> Path:
    root = _state_root() / "pending"
    root.mkdir(parents=True, exist_ok=True)
    return root


def capture_root() -> Path:
    configured = os.environ.get("SCHOOK_HOOK_CAPTURE_ROOT", "").strip()
    if configured:
        root = Path(configured).expanduser().resolve()
    else:
        root = (Path(__file__).resolve().parents[3] / "test-harness" / "hooks" / "codex" / "captures" / "raw").resolve()
    root.mkdir(parents=True, exist_ok=True)
    return root


def _sanitize_file_token(value: str) -> str:
    token = "".join(ch if ch.isalnum() or ch in ("-", "_", ".") else "_" for ch in value.strip())
    return token or "global"


def _atomic_write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temp_path = path.with_name(f"{path.name}.{uuid4().hex}.tmp")
    temp_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    temp_path.replace(path)


def _append_jsonl(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a", encoding="utf-8") as handle:
        handle.write(json.dumps(payload, sort_keys=True) + "\n")


def _load_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def _normalize_payload(raw_text: str) -> dict[str, Any]:
    text = raw_text.strip()
    if not text:
        return {}
    try:
        parsed = json.loads(text)
    except json.JSONDecodeError:
        return {"_invalid_json": text}
    if isinstance(parsed, dict):
        return parsed
    return {"_payload": parsed}


def _safe_str(value: Any) -> str | None:
    if isinstance(value, str):
        value = value.strip()
        return value or None
    return None


def _redact_env_value(key: str, value: str) -> str:
    if key in _ENV_REDACTION_EXACT_KEYS:
        return value
    upper_key = key.upper()
    if any(token in upper_key for token in _ENV_REDACTION_SUBSTRINGS):
        return "<redacted>"
    return value


def _normalize_env_snapshot(hook_name: str, timestamp: str) -> dict[str, Any]:
    def filtered_env(prefix: str) -> dict[str, str]:
        return {
            key: _redact_env_value(key, value)
            for key, value in sorted(os.environ.items())
            if key.startswith(prefix)
        }

    return {
        "captured_at": timestamp,
        "hook_name": hook_name,
        "cwd_from_getcwd": os.getcwd(),
        "pwd_env": os.environ.get("PWD"),
        "codex_env": filtered_env("CODEX"),
        "atm_env": filtered_env("ATM"),
        "sc_hook_env": filtered_env("SC_HOOK"),
    }


def write_capture(hook_name: str, raw_text: str) -> None:
    timestamp = _utc_now().strftime("%Y%m%dT%H%M%S.%fZ")
    root = capture_root()
    _atomic_write_json(root / f"{timestamp}-{hook_name}.json", _normalize_payload(raw_text))
    _atomic_write_json(
        root / f"{timestamp}-{hook_name}.env.json",
        _normalize_env_snapshot(hook_name, timestamp),
    )


def capture_stop(raw_text: str) -> None:
    write_capture("stop", raw_text)


def _payload_project_dir(payload: dict[str, Any]) -> Path | None:
    for key in ("cwd", "project_dir", "projectDir"):
        value = str(payload.get(key, "")).strip()
        if value:
            return _resolve_project_dir(value)
    configured = os.environ.get("CODEX_PROJECT_DIR", "").strip()
    if configured:
        return _resolve_project_dir(configured)
    return None


def _payload_cwd(payload: dict[str, Any]) -> Path | None:
    value = _safe_str(payload.get("cwd"))
    if value:
        return Path(value).expanduser().resolve()
    return None


def _within_project_scope(payload: dict[str, Any]) -> bool:
    configured = os.environ.get("SCHOOK_CODEX_HOOK_PROJECT_ROOT", "").strip()
    if not configured:
        return True
    try:
        project_root = Path(configured).expanduser().resolve()
        payload_dir = _payload_project_dir(payload)
        if payload_dir is None:
            return False
        payload_dir.relative_to(project_root)
        return True
    except Exception:
        return False


def _resolve_project_dir(cwd: str) -> Path:
    base = Path(cwd).expanduser().resolve()
    try:
        result = subprocess.run(
            ["git", "-C", str(base), "rev-parse", "--show-toplevel"],
            check=True,
            capture_output=True,
            text=True,
        )
        root = result.stdout.strip()
        if root:
            return Path(root).expanduser().resolve()
    except Exception:
        pass
    return base


def _find_atm_toml(start_dir: Path) -> Path | None:
    current = start_dir.resolve()
    while True:
        candidate = current / ".atm.toml"
        if candidate.is_file():
            return candidate
        parent = current.parent
        if parent == current:
            return None
        current = parent


def _session_identifier(payload: dict[str, Any]) -> str | None:
    for key in ("session_id", "sessionId", "thread-id", "thread_id"):
        value = _safe_str(payload.get(key))
        if value:
            return value
    return None


def _session_record_path(start_dir: Path, session_id: str) -> Path | None:
    current = start_dir.resolve()
    while True:
        sessions_dir = current / ".sc" / "sessions" / "codex"
        if sessions_dir.is_dir():
            matches = sorted(sessions_dir.glob(f"*-{session_id}.json"))
            if matches:
                return matches[0]
        parent = current.parent
        if parent == current:
            return None
        current = parent


def _find_session_record(payload: dict[str, Any]) -> tuple[Path | None, dict[str, Any] | None]:
    session_id = _session_identifier(payload)
    if not session_id:
        return None, None

    candidate_dirs: list[Path] = []
    for candidate in (_payload_cwd(payload), _payload_project_dir(payload)):
        if candidate is not None and candidate not in candidate_dirs:
            candidate_dirs.append(candidate)

    for start_dir in candidate_dirs:
        path = _session_record_path(start_dir, session_id)
        if path is None:
            continue
        try:
            data = _load_json(path)
        except Exception:
            continue
        return path, data
    return None, None


def _canonical_project_dir(payload: dict[str, Any]) -> Path | None:
    _, record = _find_session_record(payload)
    if isinstance(record, dict):
        record_cwd = _safe_str(record.get("cwd"))
        if record_cwd:
            return _resolve_project_dir(record_cwd)
        record_project_dir = _safe_str(record.get("project_dir"))
        if record_project_dir:
            return Path(record_project_dir).expanduser().resolve()
    return _payload_project_dir(payload)


def _canonical_cwd(payload: dict[str, Any]) -> str | None:
    _, record = _find_session_record(payload)
    if isinstance(record, dict):
        record_cwd = _safe_str(record.get("cwd"))
        if record_cwd:
            return record_cwd
    current = _payload_cwd(payload)
    return str(current) if current is not None else None


def _correlation_key(payload: dict[str, Any]) -> str:
    for key in ("thread-id", "thread_id", "sessionId", "session_id", "turn-id", "turn_id"):
        value = str(payload.get(key, "")).strip()
        if value:
            return _sanitize_file_token(value)

    parts = [
        os.environ.get("ATM_TEAM", "").strip(),
        os.environ.get("ATM_IDENTITY", "").strip(),
    ]
    fallback = "--".join(part for part in parts if part)
    return _sanitize_file_token(fallback or "global")


def _record_path(key: str) -> Path:
    return pending_root() / f"{key}.json"


def _marker_identity() -> str:
    return _sanitize_file_token(os.environ.get("ATM_IDENTITY", "").strip() or "unknown")


def _marker_dir(project_dir: str | None) -> Path | None:
    if not project_dir:
        return None
    return Path(project_dir).expanduser().resolve() / ".sc" / "sessions" / "codex"


def _marker_paths(project_dir: str | None) -> tuple[Path | None, Path | None]:
    marker_dir = _marker_dir(project_dir)
    if marker_dir is None:
        return None, None
    identity = _marker_identity()
    return (
        marker_dir / f"active-{identity}.json",
        marker_dir / f"idle-{identity}.json",
    )


def _marker_payload(
    *,
    state: str,
    project_dir: str | None,
    payload: dict[str, Any],
    current: datetime,
) -> dict[str, Any]:
    marker = {
        "state": state,
        "updated_at": current.isoformat(),
        "atm_identity": os.environ.get("ATM_IDENTITY", "").strip() or None,
        "atm_team": os.environ.get("ATM_TEAM", "").strip() or None,
        "project_dir": project_dir,
        "cwd": _canonical_cwd(payload),
        "thread_id": payload.get("thread-id") or payload.get("thread_id"),
        "session_id": _session_identifier(payload),
        "turn_id": payload.get("turn-id") or payload.get("turn_id"),
        "hook_event_name": payload.get("hook_event_name"),
    }
    if state == "idle":
        marker["idle_since"] = current.isoformat()
    return marker


def _set_marker_state(
    *,
    state: str,
    project_dir: str | None,
    payload: dict[str, Any],
    current: datetime,
) -> Path | None:
    active_path, idle_path = _marker_paths(project_dir)
    if active_path is None or idle_path is None:
        return None

    target_path = active_path if state == "active" else idle_path
    other_path = idle_path if state == "active" else active_path
    target_path.parent.mkdir(parents=True, exist_ok=True)

    if other_path.exists():
        other_path.replace(target_path)

    _atomic_write_json(
        target_path,
        _marker_payload(
            state=state,
            project_dir=project_dir,
            payload=payload,
            current=current,
        ),
    )
    other_path.unlink(missing_ok=True)
    return target_path


def _command_from_env() -> list[str]:
    raw = os.environ.get("SCHOOK_CODEX_DEBOUNCE_COMMAND", "").strip()
    if not raw:
        return []
    if raw.startswith("["):
        parsed = json.loads(raw)
        if not isinstance(parsed, list) or not all(isinstance(item, str) for item in parsed):
            raise ValueError("SCHOOK_CODEX_DEBOUNCE_COMMAND JSON form must be a string array")
        return parsed
    return shlex.split(raw)


@dataclass(frozen=True)
class IdleNotifyConfig:
    recipient: str
    seconds: float
    team: str
    sender: str


def _float_value(value: Any) -> float | None:
    if value is None:
        return None
    if isinstance(value, (int, float)):
        return float(value)
    if isinstance(value, str):
        text = value.strip()
        if not text:
            return None
        return float(text)
    return None


def _resolve_idle_notify_config(project_dir: str | None) -> IdleNotifyConfig | None:
    atm_team = os.environ.get("ATM_TEAM", "").strip()
    atm_identity = os.environ.get("ATM_IDENTITY", "").strip()
    if not atm_team or not atm_identity or not project_dir:
        return None

    toml_path = _find_atm_toml(Path(project_dir))
    if toml_path is None:
        return None

    try:
        config = tomllib.loads(toml_path.read_text(encoding="utf-8"))
    except Exception:
        return None

    atm_section = config.get("atm")
    if not isinstance(atm_section, dict):
        return None

    idle_section = atm_section.get("idle_notify")
    if not isinstance(idle_section, dict):
        return None

    agent_map = idle_section.get("agent")
    agent_section: dict[str, Any] = {}
    if isinstance(agent_map, dict):
        candidate = agent_map.get(atm_identity)
        if isinstance(candidate, dict):
            agent_section = candidate

    if agent_section.get("enabled") is False:
        return None

    recipient = _safe_str(agent_section.get("recipient")) or _safe_str(idle_section.get("recipient")) or "team-lead"
    seconds = _float_value(agent_section.get("seconds"))
    if seconds is None:
        seconds = _float_value(idle_section.get("default_seconds"))
    if seconds is None or seconds <= 0:
        return None

    return IdleNotifyConfig(
        recipient=recipient,
        seconds=seconds,
        team=atm_team,
        sender=atm_identity,
    )


def _send_idle_notification(record: "PendingRecord", current: datetime) -> None:
    if record.idle_notify is None:
        return

    timestamp = current.strftime("%Y-%m-%dT%H:%M:%SZ")
    message = f"{record.idle_notify.sender} idle for {int(record.idle_notify.seconds) if record.idle_notify.seconds.is_integer() else record.idle_notify.seconds:g} seconds @ {timestamp}"
    command = [
        "atm",
        "send",
        record.idle_notify.recipient,
        message,
        "--team",
        record.idle_notify.team,
        "--from",
        record.idle_notify.sender,
    ]
    try:
        subprocess.run(
            command,
            cwd=record.project_dir or os.getcwd(),
            check=False,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
    except Exception as exc:
        _append_jsonl(
            _state_root() / "launcher-errors.jsonl",
            {
                "error": str(exc),
                "event": "idle_notify_failed",
                "ts": current.isoformat(),
            },
        )


def _fire_pending_script() -> Path:
    configured = os.environ.get("SCHOOK_CODEX_FIRE_PENDING_SCRIPT", "").strip()
    if configured:
        return Path(configured).expanduser().resolve()
    return (Path(__file__).resolve().parents[3] / "test-harness" / "hooks" / "codex" / "scripts" / "fire_pending.py").resolve()


def _launch_timer_worker(delay_seconds: float) -> None:
    command = [
        sys.executable,
        str(_fire_pending_script()),
        "--sleep-seconds",
        str(delay_seconds),
    ]
    try:
        subprocess.Popen(
            command,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            start_new_session=True,
            env=os.environ.copy(),
        )
    except Exception as exc:
        _append_jsonl(
            _state_root() / "launcher-errors.jsonl",
            {
                "error": str(exc),
                "event": "launch_timer_worker_failed",
                "ts": _utc_now().isoformat(),
            },
        )


@dataclass(frozen=True)
class PendingRecord:
    key: str
    due_at: datetime
    command: list[str]
    project_dir: str | None
    payload: dict[str, Any]
    idle_notify: IdleNotifyConfig | None

    @classmethod
    def from_json(cls, payload: dict[str, Any]) -> "PendingRecord":
        command = payload.get("command", [])
        if not isinstance(command, list) or not all(isinstance(item, str) for item in command):
            raise ValueError("pending record command must be a string array")
        idle_notify_payload = payload.get("idle_notify")
        idle_notify: IdleNotifyConfig | None = None
        if isinstance(idle_notify_payload, dict):
            idle_notify = IdleNotifyConfig(
                recipient=str(idle_notify_payload["recipient"]),
                seconds=float(idle_notify_payload["seconds"]),
                team=str(idle_notify_payload["team"]),
                sender=str(idle_notify_payload["sender"]),
            )
        return cls(
            key=str(payload["key"]),
            due_at=datetime.fromisoformat(str(payload["due_at"])),
            command=command,
            project_dir=payload.get("project_dir"),
            payload=dict(payload.get("payload", {})),
            idle_notify=idle_notify,
        )

    def to_json(self) -> dict[str, Any]:
        payload = {
            "key": self.key,
            "due_at": self.due_at.isoformat(),
            "command": self.command,
            "project_dir": self.project_dir,
            "payload": self.payload,
        }
        if self.idle_notify is not None:
            payload["idle_notify"] = {
                "recipient": self.idle_notify.recipient,
                "seconds": self.idle_notify.seconds,
                "team": self.idle_notify.team,
                "sender": self.idle_notify.sender,
            }
        return payload


def schedule_stop(raw_text: str, now: datetime | None = None) -> Path | None:
    payload = _normalize_payload(raw_text)
    if not _within_project_scope(payload):
        return None

    write_capture("notify", raw_text)
    current = now or _utc_now()
    key = _correlation_key(payload)
    canonical_project_dir = _canonical_project_dir(payload)
    project_dir = str(canonical_project_dir) if canonical_project_dir is not None else None
    idle_notify = _resolve_idle_notify_config(project_dir)
    debounce_seconds = idle_notify.seconds if idle_notify is not None else float(
        os.environ.get("SCHOOK_CODEX_DEBOUNCE_SECONDS", "60").strip() or "60"
    )
    record = PendingRecord(
        key=key,
        due_at=current + timedelta(seconds=debounce_seconds),
        command=_command_from_env(),
        project_dir=project_dir,
        payload=payload,
        idle_notify=idle_notify,
    )
    path = _record_path(key)
    _atomic_write_json(path, record.to_json())

    if os.environ.get("SCHOOK_CODEX_DEBOUNCE_AUTOSTART", "1").strip() != "0":
        _launch_timer_worker(debounce_seconds)

    return path


def cancel_on_pretooluse(raw_text: str) -> bool:
    payload = _normalize_payload(raw_text)
    if not _within_project_scope(payload):
        return False

    write_capture("pretooluse", raw_text)
    current = _utc_now()
    canonical_project_dir = _canonical_project_dir(payload)
    project_dir = str(canonical_project_dir) if canonical_project_dir is not None else None
    _set_marker_state(state="active", project_dir=project_dir, payload=payload, current=current)
    path = _record_path(_correlation_key(payload))
    if not path.exists():
        return False
    path.unlink()
    return True


def process_pending(now: datetime | None = None, sleep_seconds: float = 0.0) -> list[Path]:
    if sleep_seconds > 0:
        time.sleep(sleep_seconds)

    current = now or _utc_now()
    fired: list[Path] = []
    for path in sorted(pending_root().glob("*.json")):
        record = PendingRecord.from_json(_load_json(path))
        if record.due_at > current:
            continue

        _set_marker_state(
            state="idle",
            project_dir=record.project_dir,
            payload=record.payload,
            current=current,
        )
        if record.command:
            cwd = record.project_dir or os.getcwd()
            subprocess.run(record.command, cwd=cwd, check=False)

        _send_idle_notification(record, current)
        path.unlink(missing_ok=True)
        fired.append(path)
    return fired
