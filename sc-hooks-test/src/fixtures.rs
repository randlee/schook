use std::fs;
use std::path::{Path, PathBuf};

pub fn create_shell_plugin(path: &Path, manifest_json: &str, runtime_output_json: &str) {
    let runtime_body = format!("cat >/dev/null\ncat <<'JSON'\n{runtime_output_json}\nJSON\n");
    create_shell_plugin_script(path, manifest_json, &runtime_body);
}

pub fn create_shell_plugin_script(path: &Path, manifest_json: &str, runtime_body: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("plugin parent directory should be creatable");
    }

    let script = format!(
        "#!/bin/sh\nif [ \"$1\" = \"--manifest\" ]; then\n  cat <<'JSON'\n{manifest_json}\nJSON\n  exit 0\nfi\n{runtime_body}"
    );
    fs::write(path, script).expect("plugin script should be writable");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)
            .expect("plugin metadata should be available")
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms).expect("plugin should be executable");
    }
}

pub fn plugin_path(root: &Path, plugin_name: &str) -> PathBuf {
    root.join(".sc-hooks").join("plugins").join(plugin_name)
}

pub fn write_minimal_config(root: &Path, hook: &str, plugin_name: &str) {
    let config_path = root.join(".sc-hooks").join("config.toml");
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).expect("config parent directory should be creatable");
    }

    let config = format!("[meta]\nversion = 1\n\n[hooks]\n{hook} = [\"{plugin_name}\"]\n");
    fs::write(config_path, config).expect("config file should be writable");
}
