use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::json;

use crate::errors::CliError;

pub fn write_entry(
    hook_log_path: &str,
    hook: &str,
    event: Option<&str>,
    mode: sc_hooks_core::dispatch::DispatchMode,
) -> Result<(), CliError> {
    let log_path = Path::new(hook_log_path);
    if let Some(parent) = log_path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).map_err(|err| {
            CliError::internal(format!(
                "failed to create hook log directory {}: {err}",
                parent.display()
            ))
        })?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .map_err(|err| {
            CliError::internal(format!(
                "failed to open hook log file {}: {err}",
                log_path.display()
            ))
        })?;

    let ts_millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    let entry = json!({
        "ts_millis": ts_millis,
        "hook": hook,
        "event": event,
        "mode": mode.as_str(),
        "handler": "log",
        "action": "proceed"
    });
    let line = serde_json::to_string(&entry)
        .map_err(|err| CliError::internal(format!("failed serializing hook log entry: {err}")))?;
    writeln!(file, "{line}")
        .map_err(|err| CliError::internal(format!("failed writing hook log entry: {err}")))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_jsonl_entry_to_configured_hook_log_path() {
        let temp = tempfile::tempdir().expect("tempdir should create");
        let log_path = temp.path().join("logs").join("hooks.jsonl");

        write_entry(
            &log_path.display().to_string(),
            "PreToolUse",
            Some("Write"),
            sc_hooks_core::dispatch::DispatchMode::Sync,
        )
        .expect("log write should succeed");

        let rendered = fs::read_to_string(&log_path).expect("log file should be readable");
        let line = rendered.lines().next().expect("line should exist");
        let parsed: serde_json::Value = serde_json::from_str(line).expect("line should be json");

        assert_eq!(parsed["hook"], "PreToolUse");
        assert_eq!(parsed["event"], "Write");
        assert_eq!(parsed["mode"], "sync");
        assert_eq!(parsed["handler"], "log");
        assert_eq!(parsed["action"], "proceed");
    }
}
