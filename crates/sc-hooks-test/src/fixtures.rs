use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Creates an executable script at the given path.
pub fn create_executable_script(path: &Path, body: &str) {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).expect("script parent directory should be creatable");

    let mut temp =
        tempfile::NamedTempFile::new_in(parent).expect("temporary script file should be creatable");
    temp.write_all(body.as_bytes())
        .expect("script should be writable");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = temp
            .as_file()
            .metadata()
            .expect("script metadata should be available")
            .permissions();
        perms.set_mode(0o755);
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
