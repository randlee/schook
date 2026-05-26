//! Shared shell-fixture helpers used by `sc-hooks-test` and host-path
//! integration tests.

use sc_hooks_core::process::retry_executable_file_busy;
use std::fs;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};

/// Creates an executable script at the given path.
///
/// # Panics
///
/// Panics when the parent directory, temporary file, permissions update, or
/// final atomic persist step cannot be completed for the requested fixture path.
pub fn create_executable_script(path: impl AsRef<Path>, body: &str) {
    let path = path.as_ref();
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).expect("script parent directory should be creatable");

    let mut temp =
        tempfile::NamedTempFile::new_in(parent).expect("temporary script file should be creatable");
    temp.write_all(body.as_bytes())
        .expect("script should be writable");

    #[cfg(unix)]
    {
        let mut perms = temp
            .as_file()
            .metadata()
            .expect("script metadata should be available")
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut perms, 0o755);
        temp.as_file()
            .set_permissions(perms)
            .expect("script should be executable");
    }

    temp.as_file()
        .sync_all()
        .expect("script should sync before persist");
    temp.into_temp_path()
        .persist(path)
        .expect("script should be persisted atomically");
}

/// Spawns a shell fixture command with the short bounded retry used for freshly
/// persisted test scripts on Unix-like systems.
///
/// # Errors
///
/// Returns any process-spawn error from the underlying command. Transient
/// `ExecutableFileBusy` failures are retried through the shared runtime helper
/// before the final error is returned to the caller.
pub fn spawn_fixture_command(command: &mut Command) -> io::Result<Child> {
    retry_executable_file_busy(|| command.spawn())
}

/// Creates a shell plugin script that echoes a fixed JSON runtime payload.
///
/// # Panics
///
/// Panics when the plugin script cannot be rendered or persisted through
/// [`create_shell_plugin_script`].
pub fn create_shell_plugin(path: impl AsRef<Path>, manifest_json: &str, runtime_output_json: &str) {
    let runtime_body = format!("cat >/dev/null\ncat <<'JSON'\n{runtime_output_json}\nJSON\n");
    create_shell_plugin_script(path, manifest_json, &runtime_body);
}

/// Creates a shell plugin script with a custom runtime body.
///
/// # Panics
///
/// Panics when the composed script cannot be persisted through
/// [`create_executable_script`].
pub fn create_shell_plugin_script(path: impl AsRef<Path>, manifest_json: &str, runtime_body: &str) {
    let script = format!(
        "#!/bin/sh\nif [ \"$1\" = \"--manifest\" ]; then\n  cat <<'JSON'\n{manifest_json}\nJSON\n  exit 0\nfi\n{runtime_body}"
    );
    create_executable_script(path, &script);
}

/// Returns the runtime plugin path under a test root.
///
/// # Panics
///
/// Panics if the supplied root path cannot be represented as a normal runtime
/// fixture path. The current implementation only joins path segments and does
/// not perform fallible I/O.
pub fn plugin_path(root: impl AsRef<Path>, plugin_name: &str) -> PathBuf {
    let root = root.as_ref();
    root.join(".sc-hooks").join("plugins").join(plugin_name)
}

/// Writes the smallest valid `.sc-hooks/config.toml` for a single hook/plugin mapping.
///
/// # Panics
///
/// Panics when the config directory cannot be created or the config file cannot
/// be written for the requested test root.
pub fn write_minimal_config(root: impl AsRef<Path>, hook: &str, plugin_name: &str) {
    let root = root.as_ref();
    let config_path = root.join(".sc-hooks").join("config.toml");
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).expect("config parent directory should be creatable");
    }

    let config = format!("[meta]\nversion = 1\n\n[hooks]\n{hook} = [\"{plugin_name}\"]\n");
    fs::write(config_path, config).expect("config file should be writable");
}
