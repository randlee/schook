use std::fs;
use std::path::{Path, PathBuf};

pub fn create_shell_plugin(path: &Path, manifest_json: &str, runtime_output_json: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("plugin parent directory should be creatable");
    }

    let script = format!(
        "#!/bin/sh\nif [ \"$1\" = \"--manifest\" ]; then\n  cat <<'JSON'\n{manifest_json}\nJSON\n  exit 0\nfi\ncat >/dev/null\ncat <<'JSON'\n{runtime_output_json}\nJSON\n"
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
