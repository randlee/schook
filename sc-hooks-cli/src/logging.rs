use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

use crate::config::LogLevel;
use crate::errors::CliError;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct HandlerResultLog {
    pub handler: String,
    pub action: String,
    pub ms: u128,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DispatchLogEntry {
    pub ts: String,
    pub ts_millis: u128,
    pub hook: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,
    pub matcher: String,
    pub mode: String,
    pub handlers: Vec<String>,
    pub results: Vec<HandlerResultLog>,
    pub total_ms: u128,
    pub exit: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_notification: Option<String>,
    pub level: String,
}

pub fn append_dispatch_log(
    hook_log_path: &str,
    log_level: LogLevel,
    mut entry: DispatchLogEntry,
) -> Result<(), CliError> {
    let path = Path::new(hook_log_path);
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).map_err(|err| {
            CliError::internal(format!(
                "failed creating hook log directory {}: {err}",
                parent.display()
            ))
        })?;
    }

    entry.level = match log_level {
        LogLevel::Debug => "debug".to_string(),
        LogLevel::Info => "info".to_string(),
        LogLevel::Warn => "warn".to_string(),
        LogLevel::Error => "error".to_string(),
    };

    let rendered = serde_json::to_string(&entry)
        .map_err(|err| CliError::internal(format!("failed to serialize dispatch log: {err}")))?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|err| {
            CliError::internal(format!("failed opening hook log {}: {err}", path.display()))
        })?;
    writeln!(file, "{rendered}")
        .map_err(|err| CliError::internal(format!("failed writing dispatch log entry: {err}")))?;

    Ok(())
}

pub fn now_ts_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

pub fn now_ts_iso8601() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    #[cfg(unix)]
    {
        let raw = seconds as nix::libc::time_t;
        // SAFETY: `gmtime_r` writes to the provided `tm` for a valid `time_t` pointer.
        unsafe {
            let mut tm: nix::libc::tm = std::mem::zeroed();
            if nix::libc::gmtime_r(&raw, &mut tm).is_null() {
                return seconds.to_string();
            }
            format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
                tm.tm_year + 1900,
                tm.tm_mon + 1,
                tm.tm_mday,
                tm.tm_hour,
                tm.tm_min,
                tm.tm_sec
            )
        }
    }
    #[cfg(not(unix))]
    {
        seconds.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appends_structured_jsonl_entry() {
        let temp = tempfile::tempdir().expect("tempdir should create");
        let log_path = temp.path().join("logs").join("hooks.jsonl");

        let entry = DispatchLogEntry {
            ts: "2026-03-07T00:00:01Z".to_string(),
            ts_millis: 1,
            hook: "PreToolUse".to_string(),
            event: Some("Write".to_string()),
            matcher: "Write".to_string(),
            mode: "sync".to_string(),
            handlers: vec!["guard-paths".to_string()],
            results: vec![HandlerResultLog {
                handler: "guard-paths".to_string(),
                action: "proceed".to_string(),
                ms: 2,
                error_type: None,
                stderr: None,
                warning: None,
                disabled: None,
            }],
            total_ms: 2,
            exit: 0,
            ai_notification: None,
            level: String::new(),
        };

        append_dispatch_log(&log_path.display().to_string(), LogLevel::Info, entry)
            .expect("log append should succeed");

        let content = fs::read_to_string(log_path).expect("log file should be readable");
        let line = content.lines().next().expect("line should exist");
        let parsed: serde_json::Value = serde_json::from_str(line).expect("entry should be json");
        assert_eq!(parsed["ts"], "2026-03-07T00:00:01Z");
        assert_eq!(parsed["hook"], "PreToolUse");
        assert_eq!(parsed["event"], "Write");
        assert_eq!(parsed["matcher"], "Write");
        assert_eq!(parsed["mode"], "sync");
        assert_eq!(parsed["exit"], 0);
        assert_eq!(parsed["results"][0]["action"], "proceed");
        assert_eq!(parsed["level"], "info");
    }
}
