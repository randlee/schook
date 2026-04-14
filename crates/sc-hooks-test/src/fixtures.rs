use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::thread;
use std::time::Duration;

/// Creates an executable script at the given path.
pub fn create_executable_script(path: &Path, body: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("script parent directory should be creatable");
    }

    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, body).expect("script should be writable");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&temp_path)
            .expect("script metadata should be available")
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&temp_path, perms).expect("script should be executable");
    }

    fs::rename(&temp_path, path).expect("script should move into place atomically");
}

/// Spawns a fixture command, retrying the transient Unix `ETXTBSY` race that
/// can happen immediately after a script fixture is written.
pub(crate) fn spawn_fixture_command(command: &mut Command) -> std::io::Result<Child> {
    const MAX_ATTEMPTS: usize = 5;
    const RETRY_DELAY_MS: u64 = 10;

    for attempt in 1..=MAX_ATTEMPTS {
        match command.spawn() {
            Ok(child) => return Ok(child),
            Err(err) if is_executable_file_busy(&err) && attempt < MAX_ATTEMPTS => {
                thread::sleep(Duration::from_millis(RETRY_DELAY_MS));
            }
            Err(err) => return Err(err),
        }
    }

    unreachable!("retry loop must return or error")
}

#[cfg(unix)]
fn is_executable_file_busy(err: &std::io::Error) -> bool {
    err.raw_os_error() == Some(26)
}

#[cfg(not(unix))]
fn is_executable_file_busy(_err: &std::io::Error) -> bool {
    false
}

/// Creates a shell plugin script that echoes a fixed JSON runtime payload.
pub fn create_shell_plugin(path: &Path, manifest_json: &str, runtime_output_json: &str) {
    let runtime_body = format!("cat >/dev/null\ncat <<'JSON'\n{runtime_output_json}\nJSON\n");
    create_shell_plugin_script(path, manifest_json, &runtime_body);
}

/// Creates a shell plugin script with a custom runtime body.
pub fn create_shell_plugin_script(path: &Path, manifest_json: &str, runtime_body: &str) {
    let script = format!(
        "#!/bin/sh\nif [ \"$1\" = \"--manifest\" ]; then\n  cat <<'JSON'\n{manifest_json}\nJSON\n  exit 0\nfi\n{runtime_body}"
    );
    create_executable_script(path, &script);
}

/// Returns the runtime plugin path under a test root.
pub fn plugin_path(root: &Path, plugin_name: &str) -> PathBuf {
    root.join(".sc-hooks").join("plugins").join(plugin_name)
}

/// Writes the smallest valid `.sc-hooks/config.toml` for a single hook/plugin mapping.
pub fn write_minimal_config(root: &Path, hook: &str, plugin_name: &str) {
    let config_path = root.join(".sc-hooks").join("config.toml");
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).expect("config parent directory should be creatable");
    }

    let config = format!("[meta]\nversion = 1\n\n[hooks]\n{hook} = [\"{plugin_name}\"]\n");
    fs::write(config_path, config).expect("config file should be writable");
}
