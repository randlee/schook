#![cfg(unix)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use sc_hooks_test::fixtures;
use serde_json::Value;

#[derive(Clone, Copy)]
enum ProviderCase {
    Claude,
    Codex,
    Gemini,
}

impl ProviderCase {
    fn as_str(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Gemini => "gemini",
        }
    }

    fn session_start_fixture(self) -> &'static str {
        "session-start-startup.json"
    }

    fn bash_fixture(self) -> &'static str {
        match self {
            Self::Claude | Self::Codex => "pretooluse-bash.json",
            Self::Gemini => "before-tool.json",
        }
    }

    fn fixture_dir(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Gemini => "gemini",
        }
    }
}

fn cli_binary() -> String {
    let current = std::env::current_exe().expect("test binary path should resolve");
    current
        .parent()
        .and_then(|deps| deps.parent())
        .map(|debug_dir| {
            debug_dir.join(if cfg!(windows) {
                "sc-hooks.exe"
            } else {
                "sc-hooks"
            })
        })
        .expect("target/debug directory should resolve")
        .display()
        .to_string()
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("repo root should resolve")
        .to_path_buf()
}

fn fixture(provider: ProviderCase, name: &str) -> Value {
    let path = repo_root()
        .join("test-harness/hooks")
        .join(provider.fixture_dir())
        .join("fixtures/approved")
        .join(name);
    serde_json::from_str(&fs::read_to_string(path).expect("fixture should read"))
        .expect("fixture should parse")
}

fn write_session_start_config(root: &Path) {
    fs::create_dir_all(root.join(".sc-hooks")).expect(".sc-hooks should create");
    fs::write(
        root.join(".sc-hooks/config.toml"),
        r#"[meta]
version = 1

[hooks]
SessionStart = ["agent-session-foundation"]
"#,
    )
    .expect("config should write");
}

fn write_bash_gate_config(root: &Path) {
    fs::create_dir_all(root.join(".sc-hooks")).expect(".sc-hooks should create");
    fs::write(
        root.join(".sc-hooks/config.toml"),
        r#"[meta]
version = 1

[hooks]
SessionStart = ["agent-session-foundation"]
PreToolUse = ["atm-extension"]
"#,
    )
    .expect("config should write");
}

fn write_atm_config(root: &Path) {
    fs::write(
        root.join(".atm.toml"),
        r#"[core]
default_team = "schook"
identity = "chook"
"#,
    )
    .expect(".atm.toml should write");
}

fn plugin_wrapper_body(package: &str) -> String {
    let manifest = repo_root().join("Cargo.toml");
    format!(
        "#!/bin/sh\nexec cargo run --quiet --manifest-path '{}' -p {} -- \"$@\"\n",
        manifest.display(),
        package
    )
}

fn install_plugin_wrapper(root: &Path, plugin_name: &str, package: &str) {
    fixtures::create_executable_script(
        fixtures::plugin_path(root, plugin_name),
        &plugin_wrapper_body(package),
    );
}

fn read_last_log(root: &Path) -> Value {
    let rendered = fs::read_to_string(root.join(sc_hooks_core::OBSERVABILITY_LOG_PATH))
        .expect("observability log should be readable");
    let line = rendered.lines().last().expect("log line should exist");
    serde_json::from_str(line).expect("log line should parse")
}

fn run_provider_hook(
    provider: ProviderCase,
    root: &Path,
    hook: &str,
    event: Option<&str>,
    payload: Value,
) -> Output {
    let mut command = Command::new(cli_binary());
    command
        .current_dir(root)
        .arg("run")
        .arg(hook)
        .arg("--sync")
        .env("SC_HOOKS_STATE_DIR", root.join(".sc-hooks/state"))
        .env("SC_HOOK_AGENT_TYPE", provider.as_str())
        .env("SC_HOOK_AGENT_PID", "42")
        .env("ATM_TEAM", "schook")
        .env("ATM_IDENTITY", "chook");
    if let Some(event) = event {
        command.arg(event);
    }

    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("dispatch child should spawn");
    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        let body = serde_json::to_vec(&payload).expect("payload should serialize");
        stdin
            .write_all(&body)
            .expect("payload should write to stdin");
    }
    child
        .wait_with_output()
        .expect("dispatch output should be readable")
}

fn run_session_start(provider: ProviderCase, root: &Path) -> (String, String) {
    let payload = fixture(provider, provider.session_start_fixture());
    let session_id = payload["session_id"]
        .as_str()
        .expect("session id should be present")
        .to_string();
    let cwd = payload["cwd"]
        .as_str()
        .expect("cwd should be present")
        .to_string();

    let output = run_provider_hook(provider, root, "SessionStart", None, payload);
    assert_eq!(
        output.status.code(),
        Some(sc_hooks_core::exit_codes::SUCCESS)
    );

    (session_id, cwd)
}

#[test]
fn session_start_runtime_path_keeps_provider_parity() {
    for provider in [
        ProviderCase::Claude,
        ProviderCase::Codex,
        ProviderCase::Gemini,
    ] {
        let temp = tempfile::tempdir().expect("tempdir should create");
        let root = temp.path();
        write_session_start_config(root);
        install_plugin_wrapper(root, "agent-session-foundation", "agent-session-foundation");

        let (session_id, cwd) = run_session_start(provider, root);
        let rendered = fs::read_to_string(
            root.join(".sc-hooks/state")
                .join(format!("{session_id}.json")),
        )
        .expect("state file should read");
        let record: Value = serde_json::from_str(&rendered).expect("state should parse");
        assert_eq!(record["provider"], provider.as_str());
        assert_eq!(record["ai_root_dir"], cwd);
        assert_eq!(record["ai_current_dir"], cwd);
        assert_eq!(record["last_hook_event"], "SessionStart");
    }
}

#[test]
fn bash_gate_runtime_path_keeps_provider_parity_and_observability_shape() {
    for provider in [
        ProviderCase::Claude,
        ProviderCase::Codex,
        ProviderCase::Gemini,
    ] {
        let temp = tempfile::tempdir().expect("tempdir should create");
        let root = temp.path();
        write_bash_gate_config(root);
        write_atm_config(root);
        install_plugin_wrapper(root, "agent-session-foundation", "agent-session-foundation");
        install_plugin_wrapper(root, "atm-extension", "atm-extension");

        let (session_id, cwd) = run_session_start(provider, root);
        let mut payload = fixture(provider, provider.bash_fixture());
        payload["session_id"] = Value::String(session_id.clone());
        payload["cwd"] = Value::String(cwd);
        payload["tool_input"]["command"] = Value::String("atm read --team schook".to_string());

        let output = run_provider_hook(provider, root, "PreToolUse", Some("Bash"), payload);
        assert_eq!(
            output.status.code(),
            Some(sc_hooks_core::exit_codes::SUCCESS)
        );

        let rendered = fs::read_to_string(
            root.join(".sc-hooks/state")
                .join(format!("{session_id}.json")),
        )
        .expect("state file should read");
        let record: Value = serde_json::from_str(&rendered).expect("state should parse");
        assert_eq!(record["provider"], provider.as_str());
        assert_eq!(record["extensions"]["atm"]["atm_team"], "schook");
        assert_eq!(record["extensions"]["atm"]["atm_identity"], "chook");

        let log = read_last_log(root);
        assert_eq!(log["service"], "sc-hooks");
        assert_eq!(log["target"], "hook");
        assert_eq!(log["action"], "dispatch.complete");
        assert_eq!(log["outcome"], "proceed");
        assert_eq!(log["level"], "Info");
        assert_eq!(log["fields"]["hook"], "PreToolUse");
        assert_eq!(log["fields"]["event"], "Bash");
        assert_eq!(log["fields"]["mode"], "sync");
        assert_eq!(
            log["fields"]["handlers"],
            serde_json::json!(["atm-extension"])
        );
        assert_eq!(log["fields"]["exit"], sc_hooks_core::exit_codes::SUCCESS);
        assert_eq!(log["fields"]["results"][0]["action"], "proceed");
    }
}
