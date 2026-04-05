#![cfg(unix)]

use std::fs;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use sc_hooks_core::errors::RootDivergenceNotice;
use sc_hooks_core::events::HookType;
use sc_hooks_core::session::{AiRootDir, SessionId};
use sc_hooks_test::fixtures;
use serde_json::Value;

struct DispatchHarness {
    binary: String,
}

impl DispatchHarness {
    fn new() -> Self {
        let current = std::env::current_exe().expect("test binary path should resolve");
        let binary = current
            .parent()
            .and_then(|deps| deps.parent())
            .map(|debug_dir| {
                debug_dir.join(if cfg!(windows) {
                    "sc-hooks.exe"
                } else {
                    "sc-hooks"
                })
            })
            .expect("target/debug directory should resolve");
        Self {
            binary: binary.display().to_string(),
        }
    }

    fn run_sync(
        &self,
        root: &Path,
        hook: &str,
        event: Option<&str>,
        payload: Option<Value>,
        session_id: Option<&str>,
    ) -> Output {
        self.run_sync_with_env(root, hook, event, payload, session_id, &[])
    }

    fn run_sync_with_env(
        &self,
        root: &Path,
        hook: &str,
        event: Option<&str>,
        payload: Option<Value>,
        session_id: Option<&str>,
        extra_env: &[(&str, &str)],
    ) -> Output {
        let mut command = Command::new(&self.binary);
        command.current_dir(root).arg("run").arg(hook).arg("--sync");
        if let Some(event) = event {
            command.arg(event);
        }
        if let Some(session_id) = session_id {
            command.env("SC_HOOK_SESSION_ID", session_id);
        }
        if extra_env
            .iter()
            .any(|(key, _)| *key == "SC_HOOKS_TEST_FORCE_OBSERVABILITY_FAILURE")
        {
            command.env("SC_HOOKS_TEST_MODE", "1");
        }
        for (key, value) in extra_env {
            command.env(key, value);
        }

        if let Some(payload) = payload {
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
        } else {
            command.output().expect("dispatch command should execute")
        }
    }
}

fn read_last_log(root: &Path) -> Value {
    let rendered = fs::read_to_string(root.join(sc_hooks_core::OBSERVABILITY_LOG_PATH))
        .expect("observability log should be readable");
    let line = rendered.lines().last().expect("log line should exist");
    serde_json::from_str(line).expect("log line should parse")
}

fn read_all_logs(root: &Path) -> Vec<Value> {
    fs::read_to_string(root.join(sc_hooks_core::OBSERVABILITY_LOG_PATH))
        .expect("observability log should be readable")
        .lines()
        .map(|line| serde_json::from_str(line).expect("log line should parse"))
        .collect()
}

fn session_disables_plugin_for_reason(root: &Path, plugin: &str, reason: &str) -> bool {
    let state_path = root.join(".sc-hooks/state/session.json");
    let rendered = fs::read_to_string(state_path).expect("session state should exist");
    let parsed: Value = serde_json::from_str(&rendered).expect("session state should parse");
    parsed["sessions"].as_object().is_some_and(|sessions| {
        sessions
            .values()
            .any(|session| session["disabled_plugins"][plugin]["reason"].as_str() == Some(reason))
    })
}

fn console_lines(output: &Output) -> Vec<String> {
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn stderr_text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

fn read_full_audit_run(audit_root: &Path) -> (std::path::PathBuf, Value, Vec<Value>) {
    let runs_root = audit_root.join("runs");
    let mut entries = fs::read_dir(&runs_root)
        .expect("full audit runs dir should be readable")
        .map(|entry| entry.expect("run entry should be readable").path())
        .collect::<Vec<_>>();
    entries.sort();
    assert_eq!(entries.len(), 1, "expected exactly one full audit run dir");
    let run_dir = entries.pop().expect("run dir should exist");
    let meta = serde_json::from_str(
        &fs::read_to_string(run_dir.join("meta.json")).expect("full audit meta should read"),
    )
    .expect("full audit meta should parse");
    let events = fs::read_to_string(run_dir.join("events.jsonl"))
        .expect("full audit events should read")
        .lines()
        .map(|line| serde_json::from_str(line).expect("full audit line should parse"))
        .collect();
    (run_dir, meta, events)
}

fn list_full_audit_run_names(audit_root: &Path) -> Vec<String> {
    let mut entries = fs::read_dir(audit_root.join("runs"))
        .expect("full audit runs dir should be readable")
        .map(|entry| entry.expect("run entry should read").file_name())
        .map(|name| name.to_string_lossy().to_string())
        .collect::<Vec<_>>();
    entries.sort();
    entries
}

fn create_fake_full_audit_run(audit_root: &Path, run_id: &str) {
    let run_dir = audit_root.join("runs").join(run_id);
    fs::create_dir_all(&run_dir).expect("fake run dir should create");
    fs::write(run_dir.join("meta.json"), "{}\n").expect("fake meta should write");
    fs::write(run_dir.join("events.jsonl"), "{}\n").expect("fake events should write");
}

fn write_full_observability_config(root: &Path, hook: &str, plugin_name: &str, retain_runs: u32) {
    fs::create_dir_all(root.join(".sc-hooks")).expect(".sc-hooks dir should create");
    fs::write(
        root.join(".sc-hooks/config.toml"),
        format!(
            "[meta]\nversion = 1\n\n[hooks]\n{hook} = [\"{plugin_name}\"]\n\n[observability]\nmode = \"full\"\nfull_profile = \"lean\"\nretain_runs = {retain_runs}\nretain_days = 14\n"
        ),
    )
    .expect("config file should write");
}

fn fake_run_id(age: Duration, suffix: usize) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("current time should follow unix epoch")
        .as_nanos();
    format!("{}-{suffix}", now.saturating_sub(age.as_nanos()))
}

#[test]
fn success_dispatch_emits_file_sink_log_event() {
    let temp = tempfile::tempdir().expect("tempdir should create");
    let root = temp.path();
    fixtures::write_minimal_config(root, "PreToolUse", "probe-plugin");
    fixtures::create_shell_plugin(
        &fixtures::plugin_path(root, "probe-plugin"),
        r#"{"contract_version":1,"name":"probe-plugin","mode":"sync","hooks":["PreToolUse"],"matchers":["Write"],"requires":{}}"#,
        r#"{"action":"proceed"}"#,
    );

    let output = DispatchHarness::new().run_sync(
        root,
        "PreToolUse",
        Some("Write"),
        Some(serde_json::json!({"tool_input": {"command": "echo hi"}})),
        None,
    );

    assert_eq!(
        output.status.code(),
        Some(sc_hooks_core::exit_codes::SUCCESS)
    );
    let log = read_last_log(root);
    assert_eq!(log["service"], "sc-hooks");
    assert_eq!(log["target"], "hook");
    assert_eq!(log["action"], "dispatch.complete");
    assert_eq!(log["outcome"], "proceed");
    assert_eq!(log["level"], "Info");
    assert_eq!(log["fields"]["hook"], "PreToolUse");
    assert_eq!(log["fields"]["event"], "Write");
    assert_eq!(log["fields"]["matcher"], "Write");
    assert_eq!(log["fields"]["mode"], "sync");
    assert_eq!(
        log["fields"]["handlers"],
        serde_json::json!(["probe-plugin"])
    );
    assert!(log["fields"]["total_ms"].as_u64().is_some());
    assert_eq!(log["fields"]["exit"], sc_hooks_core::exit_codes::SUCCESS);
    assert_eq!(log["fields"]["results"][0]["action"], "proceed");
    assert!(root.join(sc_hooks_core::OBSERVABILITY_LOG_PATH).exists());
}

#[test]
fn invalid_sink_toggle_warns_to_stderr_and_keeps_default_file_sink() {
    let temp = tempfile::tempdir().expect("tempdir should create");
    let root = temp.path();
    fixtures::write_minimal_config(root, "PreToolUse", "probe-plugin");
    fixtures::create_shell_plugin(
        &fixtures::plugin_path(root, "probe-plugin"),
        r#"{"contract_version":1,"name":"probe-plugin","mode":"sync","hooks":["PreToolUse"],"matchers":["Write"],"requires":{}}"#,
        r#"{"action":"proceed"}"#,
    );

    let output = DispatchHarness::new().run_sync_with_env(
        root,
        "PreToolUse",
        Some("Write"),
        Some(serde_json::json!({"tool_input": {"command": "echo hi"}})),
        None,
        &[("SC_HOOKS_ENABLE_FILE_SINK", "maybe")],
    );

    assert_eq!(
        output.status.code(),
        Some(sc_hooks_core::exit_codes::SUCCESS)
    );
    let stderr = stderr_text(&output);
    assert!(stderr.contains("unrecognized value for SC_HOOKS_ENABLE_FILE_SINK"));
    assert!(root.join(sc_hooks_core::OBSERVABILITY_LOG_PATH).exists());
}

#[test]
fn off_mode_suppresses_durable_observability_output() {
    let temp = tempfile::tempdir().expect("tempdir should create");
    let root = temp.path();
    fs::create_dir_all(root.join(".sc-hooks")).expect(".sc-hooks dir should create");
    fs::write(
        root.join(".sc-hooks/config.toml"),
        r#"
[meta]
version = 1

[hooks]
PreToolUse = ["probe-plugin"]

[observability]
mode = "off"
"#,
    )
    .expect("config should write");
    fixtures::create_shell_plugin(
        &fixtures::plugin_path(root, "probe-plugin"),
        r#"{"contract_version":1,"name":"probe-plugin","mode":"sync","hooks":["PreToolUse"],"matchers":["Write"],"requires":{}}"#,
        r#"{"action":"proceed"}"#,
    );

    let output = DispatchHarness::new().run_sync(
        root,
        "PreToolUse",
        Some("Write"),
        Some(serde_json::json!({"tool_input": {"command": "echo hi"}})),
        None,
    );

    assert_eq!(
        output.status.code(),
        Some(sc_hooks_core::exit_codes::SUCCESS)
    );
    assert!(!root.join(sc_hooks_core::OBSERVABILITY_LOG_PATH).exists());
    assert!(console_lines(&output).is_empty());
}

#[test]
fn full_mode_writes_run_scoped_audit_files_for_success_dispatch() {
    let temp = tempfile::tempdir().expect("tempdir should create");
    let root = temp.path();
    fs::create_dir_all(root.join(".sc-hooks")).expect(".sc-hooks dir should create");
    fs::write(
        root.join(".sc-hooks/config.toml"),
        r#"
[meta]
version = 1

[hooks]
PreToolUse = ["probe-plugin"]

[observability]
mode = "full"
full_profile = "lean"
"#,
    )
    .expect("config should write");
    fixtures::create_shell_plugin(
        &fixtures::plugin_path(root, "probe-plugin"),
        r#"{"contract_version":1,"name":"probe-plugin","mode":"sync","hooks":["PreToolUse"],"matchers":["Write"],"requires":{}}"#,
        r#"{"action":"proceed"}"#,
    );

    let output = DispatchHarness::new().run_sync(
        root,
        "PreToolUse",
        Some("Write"),
        Some(serde_json::json!({"tool_input": {"command": "echo hi"}})),
        None,
    );

    assert_eq!(
        output.status.code(),
        Some(sc_hooks_core::exit_codes::SUCCESS)
    );
    let (run_dir, meta, events) = read_full_audit_run(&root.join(".sc-hooks/audit"));
    assert!(run_dir.starts_with(root.join(".sc-hooks/audit/runs")));
    assert_eq!(meta["service"], "sc-hooks");
    assert_eq!(meta["profile"], "lean");
    let recorded_root = std::path::PathBuf::from(
        meta["project_root"]
            .as_str()
            .expect("project_root should be a string"),
    );
    assert_eq!(
        fs::canonicalize(recorded_root).expect("recorded root should canonicalize"),
        fs::canonicalize(root).expect("temp root should canonicalize")
    );
    assert_eq!(events.len(), 2);
    assert_eq!(events[0]["name"], "hook.invocation.received");
    assert_eq!(events[1]["name"], "hook.dispatch.completed");
    assert_eq!(events[1]["outcome"], "proceed");
    assert_eq!(events[1]["handler_count"], 1);
    assert_eq!(events[1]["mode"], "sync");
}

#[test]
fn full_mode_zero_match_writes_audit_record() {
    let temp = tempfile::tempdir().expect("tempdir should create");
    let root = temp.path();
    fs::create_dir_all(root.join(".sc-hooks")).expect(".sc-hooks dir should create");
    fs::write(
        root.join(".sc-hooks/config.toml"),
        r#"
[meta]
version = 1

[hooks]
PreToolUse = ["probe-plugin"]

[observability]
mode = "full"
"#,
    )
    .expect("config should write");
    fixtures::create_shell_plugin(
        &fixtures::plugin_path(root, "probe-plugin"),
        r#"{"contract_version":1,"name":"probe-plugin","mode":"sync","hooks":["PreToolUse"],"matchers":["Read"],"requires":{}}"#,
        r#"{"action":"proceed"}"#,
    );

    let output = DispatchHarness::new().run_sync(
        root,
        "PreToolUse",
        Some("Write"),
        Some(serde_json::json!({"tool_input": {"command": "echo hi"}})),
        None,
    );

    assert_eq!(
        output.status.code(),
        Some(sc_hooks_core::exit_codes::SUCCESS)
    );
    let (_, _, events) = read_full_audit_run(&root.join(".sc-hooks/audit"));
    assert_eq!(events.len(), 2);
    assert_eq!(events[0]["name"], "hook.invocation.received");
    assert_eq!(events[1]["name"], "hook.invocation.zero_match");
    assert_eq!(events[1]["outcome"], "zero_match");
    assert_eq!(events[1]["exit"], sc_hooks_core::exit_codes::SUCCESS);
    assert!(!root.join(sc_hooks_core::OBSERVABILITY_LOG_PATH).exists());
}

#[test]
fn full_mode_path_override_writes_pre_dispatch_failure_record() {
    let temp = tempfile::tempdir().expect("tempdir should create");
    let root = temp.path();
    fs::create_dir_all(root.join(".sc-hooks")).expect(".sc-hooks dir should create");
    fs::write(
        root.join(".sc-hooks/config.toml"),
        r#"
[meta]
version = 1

[hooks]
PreToolUse = ["probe-plugin"]

[observability]
mode = "full"
path = "tmp/evals"
"#,
    )
    .expect("config should write");
    fixtures::create_shell_plugin(
        &fixtures::plugin_path(root, "probe-plugin"),
        r#"{"contract_version":1,"name":"probe-plugin","mode":"sync","hooks":["PreToolUse"],"matchers":["Write"],"requires":{"required_field":{"type":"string","validate":"non_empty"}}}"#,
        r#"{"action":"proceed"}"#,
    );

    let output = DispatchHarness::new().run_sync(
        root,
        "PreToolUse",
        Some("Write"),
        Some(serde_json::json!({"tool_input": {"command": "echo hi"}})),
        None,
    );

    assert_eq!(
        output.status.code(),
        Some(sc_hooks_core::exit_codes::VALIDATION_ERROR)
    );
    let (_, _, events) = read_full_audit_run(&root.join("tmp/evals"));
    assert_eq!(events[0]["name"], "hook.invocation.received");
    assert_eq!(events[1]["name"], "hook.invocation.failed_pre_dispatch");
    assert_eq!(events[1]["stage"], "input_preparation");
    assert_eq!(events[1]["outcome"], "error");
    assert_eq!(events[1]["degraded"], true);
    assert!(!root.join(".sc-hooks/audit").exists());
}

#[test]
fn full_mode_prunes_run_directories_by_age_and_count() {
    let temp = tempfile::tempdir().expect("tempdir should create");
    let root = temp.path();
    fs::create_dir_all(root.join(".sc-hooks")).expect(".sc-hooks dir should create");
    fs::write(
        root.join(".sc-hooks/config.toml"),
        r#"
[meta]
version = 1

[hooks]
PreToolUse = ["probe-plugin"]

[observability]
mode = "full"
"#,
    )
    .expect("config should write");
    fixtures::create_shell_plugin(
        &fixtures::plugin_path(root, "probe-plugin"),
        r#"{"contract_version":1,"name":"probe-plugin","mode":"sync","hooks":["PreToolUse"],"matchers":["Write"],"requires":{}}"#,
        r#"{"action":"proceed"}"#,
    );

    let audit_root = root.join(".sc-hooks/audit");
    fs::create_dir_all(audit_root.join("runs")).expect("runs root should create");
    for index in 0..10 {
        create_fake_full_audit_run(&audit_root, &fake_run_id(Duration::from_secs(60), index));
    }
    let old_run_id = fake_run_id(Duration::from_secs(30 * 24 * 60 * 60), 99);
    create_fake_full_audit_run(&audit_root, &old_run_id);

    let output = DispatchHarness::new().run_sync(
        root,
        "PreToolUse",
        Some("Write"),
        Some(serde_json::json!({"tool_input": {"command": "echo hi"}})),
        None,
    );

    assert_eq!(
        output.status.code(),
        Some(sc_hooks_core::exit_codes::SUCCESS)
    );
    let run_names = list_full_audit_run_names(&audit_root);
    assert_eq!(run_names.len(), 10);
    assert!(
        !run_names.contains(&old_run_id),
        "aged-out run should be pruned"
    );
}

#[test]
fn standard_mode_logger_init_failure_is_non_blocking() {
    let temp = tempfile::tempdir().expect("tempdir should create");
    let root = temp.path();
    fixtures::write_minimal_config(root, "PreToolUse", "probe-plugin");
    fixtures::create_shell_plugin(
        &fixtures::plugin_path(root, "probe-plugin"),
        r#"{"contract_version":1,"name":"probe-plugin","mode":"sync","hooks":["PreToolUse"],"matchers":["Write"],"requires":{}}"#,
        r#"{"action":"proceed"}"#,
    );

    let output = DispatchHarness::new().run_sync_with_env(
        root,
        "PreToolUse",
        Some("Write"),
        Some(serde_json::json!({"tool_input": {"command": "echo hi"}})),
        None,
        &[("SC_HOOKS_TEST_FORCE_OBSERVABILITY_FAILURE", "logger_init")],
    );

    assert_eq!(
        output.status.code(),
        Some(sc_hooks_core::exit_codes::SUCCESS)
    );
    let stderr = stderr_text(&output);
    assert!(stderr.contains("sc-hooks: failed emitting observability event:"));
    assert!(stderr.contains("forced observability logger init failure"));
}

#[test]
fn standard_mode_emit_failure_is_non_blocking() {
    let temp = tempfile::tempdir().expect("tempdir should create");
    let root = temp.path();
    fixtures::write_minimal_config(root, "PreToolUse", "probe-plugin");
    fixtures::create_shell_plugin(
        &fixtures::plugin_path(root, "probe-plugin"),
        r#"{"contract_version":1,"name":"probe-plugin","mode":"sync","hooks":["PreToolUse"],"matchers":["Write"],"requires":{}}"#,
        r#"{"action":"proceed"}"#,
    );

    let output = DispatchHarness::new().run_sync_with_env(
        root,
        "PreToolUse",
        Some("Write"),
        Some(serde_json::json!({"tool_input": {"command": "echo hi"}})),
        None,
        &[("SC_HOOKS_TEST_FORCE_OBSERVABILITY_FAILURE", "emit")],
    );

    assert_eq!(
        output.status.code(),
        Some(sc_hooks_core::exit_codes::SUCCESS)
    );
    let stderr = stderr_text(&output);
    assert!(stderr.contains("sc-hooks: failed emitting observability event:"));
    assert!(stderr.contains("forced observability emit failure"));
}

#[test]
fn full_mode_append_failure_is_non_blocking() {
    let temp = tempfile::tempdir().expect("tempdir should create");
    let root = temp.path();
    fs::create_dir_all(root.join(".sc-hooks")).expect(".sc-hooks dir should create");
    fs::write(
        root.join(".sc-hooks/config.toml"),
        r#"
[meta]
version = 1

[hooks]
PreToolUse = ["probe-plugin"]

[observability]
mode = "full"
"#,
    )
    .expect("config should write");
    fixtures::create_shell_plugin(
        &fixtures::plugin_path(root, "probe-plugin"),
        r#"{"contract_version":1,"name":"probe-plugin","mode":"sync","hooks":["PreToolUse"],"matchers":["Write"],"requires":{}}"#,
        r#"{"action":"proceed"}"#,
    );

    let output = DispatchHarness::new().run_sync_with_env(
        root,
        "PreToolUse",
        Some("Write"),
        Some(serde_json::json!({"tool_input": {"command": "echo hi"}})),
        None,
        &[("SC_HOOKS_TEST_FORCE_OBSERVABILITY_FAILURE", "audit_append")],
    );

    assert_eq!(
        output.status.code(),
        Some(sc_hooks_core::exit_codes::SUCCESS)
    );
    let stderr = stderr_text(&output);
    assert!(stderr.contains("sc-hooks: full audit degraded:"));
    assert!(stderr.contains("forced full audit append failure"));
}

#[test]
fn full_mode_prune_failure_is_non_blocking() {
    let temp = tempfile::tempdir().expect("tempdir should create");
    let root = temp.path();
    fs::create_dir_all(root.join(".sc-hooks")).expect(".sc-hooks dir should create");
    fs::write(
        root.join(".sc-hooks/config.toml"),
        r#"
[meta]
version = 1

[hooks]
PreToolUse = ["probe-plugin"]

[observability]
mode = "full"
"#,
    )
    .expect("config should write");
    fixtures::create_shell_plugin(
        &fixtures::plugin_path(root, "probe-plugin"),
        r#"{"contract_version":1,"name":"probe-plugin","mode":"sync","hooks":["PreToolUse"],"matchers":["Write"],"requires":{}}"#,
        r#"{"action":"proceed"}"#,
    );

    let output = DispatchHarness::new().run_sync_with_env(
        root,
        "PreToolUse",
        Some("Write"),
        Some(serde_json::json!({"tool_input": {"command": "echo hi"}})),
        None,
        &[("SC_HOOKS_TEST_FORCE_OBSERVABILITY_FAILURE", "audit_prune")],
    );

    assert_eq!(
        output.status.code(),
        Some(sc_hooks_core::exit_codes::SUCCESS)
    );
    let stderr = stderr_text(&output);
    assert!(stderr.contains("sc-hooks: full audit degraded:"));
    assert!(stderr.contains("forced full audit prune failure"));

    let (_, _, events) = read_full_audit_run(&root.join(".sc-hooks/audit"));
    assert_eq!(events[1]["name"], "hook.dispatch.completed");
}

#[test]
fn full_mode_concurrent_agents_shard_runs_without_corruption() {
    const AGENT_COUNT: usize = 64;

    let temp = tempfile::tempdir().expect("tempdir should create");
    let root = Arc::new(temp.path().to_path_buf());
    write_full_observability_config(&root, "PreToolUse", "probe-plugin", AGENT_COUNT as u32 + 8);
    fixtures::create_shell_plugin(
        &fixtures::plugin_path(&root, "probe-plugin"),
        r#"{"contract_version":1,"name":"probe-plugin","mode":"sync","hooks":["PreToolUse"],"matchers":["Write"],"requires":{}}"#,
        r#"{"action":"proceed"}"#,
    );

    let harness = Arc::new(DispatchHarness::new());
    let mut workers = Vec::with_capacity(AGENT_COUNT);
    for worker_id in 0..AGENT_COUNT {
        let harness = Arc::clone(&harness);
        let root = Arc::clone(&root);
        workers.push(thread::spawn(move || {
            let session_id = format!("session-{worker_id}");
            let output = harness.run_sync(
                &root,
                "PreToolUse",
                Some("Write"),
                Some(serde_json::json!({"tool_input": {"command": format!("echo {worker_id}")}})),
                Some(&session_id),
            );
            assert_eq!(
                output.status.code(),
                Some(sc_hooks_core::exit_codes::SUCCESS),
                "worker {worker_id} failed: stdout={} stderr={}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
            );
        }));
    }

    for worker in workers {
        worker.join().expect("worker thread should join");
    }

    let runs_root = root.join(".sc-hooks/audit/runs");
    let mut run_dirs = fs::read_dir(&runs_root)
        .expect("full audit runs dir should be readable")
        .map(|entry| entry.expect("run entry should read").path())
        .collect::<Vec<_>>();
    run_dirs.sort();
    assert_eq!(
        run_dirs.len(),
        AGENT_COUNT,
        "expected one run dir per concurrent agent"
    );

    for run_dir in &run_dirs {
        let meta: Value = serde_json::from_str(
            &fs::read_to_string(run_dir.join("meta.json")).expect("meta file should read"),
        )
        .expect("meta json should parse");
        assert_eq!(meta["service"], "sc-hooks");
        assert_eq!(meta["profile"], "lean");
        assert!(meta["run_id"].as_str().is_some());
        assert!(meta["invocation_id"].as_str().is_some());

        let rendered_events =
            fs::read_to_string(run_dir.join("events.jsonl")).expect("events file should read");
        let events = rendered_events
            .lines()
            .map(|line| serde_json::from_str::<Value>(line).expect("event line should parse"))
            .collect::<Vec<_>>();
        assert_eq!(
            events.len(),
            2,
            "each concurrent run should emit invocation and completion records"
        );
        assert_eq!(events[0]["name"], "hook.invocation.received");
        assert_eq!(events[1]["name"], "hook.dispatch.completed");
        assert_eq!(events[0]["run_id"], events[1]["run_id"]);
        assert_eq!(events[0]["service"], "sc-hooks");
        assert_eq!(events[1]["service"], "sc-hooks");
    }
}

#[test]
fn full_mode_debug_profile_emits_machine_readable_strict_debug_fields() {
    let temp = tempfile::tempdir().expect("tempdir should create");
    let root = temp.path();
    fs::create_dir_all(root.join(".sc-hooks")).expect(".sc-hooks dir should create");
    fs::write(
        root.join(".sc-hooks/config.toml"),
        r#"
[meta]
version = 1

[hooks]
PreToolUse = ["probe-plugin"]

[observability]
mode = "full"
full_profile = "debug"
capture_stdio = "bounded"
"#,
    )
    .expect("config should write");
    fixtures::create_shell_plugin_script(
        &fixtures::plugin_path(root, "probe-plugin"),
        r#"{"contract_version":1,"name":"probe-plugin","mode":"sync","hooks":["PreToolUse"],"matchers":["Write"],"requires":{}}"#,
        r#"cat >/dev/null
printf '%s\n' 'debug stderr detail should be summarized instead of copied verbatim' >&2
cat <<'JSON'
{"action":"proceed"}
JSON
"#,
    );

    let output = DispatchHarness::new().run_sync(
        root,
        "PreToolUse",
        Some("Write"),
        Some(serde_json::json!({"tool_input": {"command": "echo hi"}})),
        None,
    );

    assert_eq!(
        output.status.code(),
        Some(sc_hooks_core::exit_codes::SUCCESS)
    );
    let (_, _, events) = read_full_audit_run(&root.join(".sc-hooks/audit"));
    let dispatch_record = &events[1];
    assert_eq!(dispatch_record["profile"], "debug");
    assert!(dispatch_record["config_source_summary"].is_object());
    assert!(dispatch_record["config_layer_resolution"].is_object());
    assert!(dispatch_record["decision_trace_summary"].is_object());
    assert!(dispatch_record["handler_stderr_excerpt"].is_array());
    assert!(dispatch_record["handler_stdout_excerpt"].is_array());
    assert!(dispatch_record["redaction_actions"].is_array());
    assert!(dispatch_record["payload_capture_state"].is_object());
    assert_eq!(
        dispatch_record["payload_capture_state"]["redaction"],
        "strict"
    );
    assert_eq!(
        dispatch_record["payload_capture_state"]["capture_payloads"],
        false
    );
    assert_eq!(
        dispatch_record["payload_capture_state"]["payloads_included"],
        false
    );
    assert_eq!(
        dispatch_record["decision_trace_summary"]["record"],
        "hook.dispatch.completed"
    );
    assert_eq!(
        dispatch_record["decision_trace_summary"]["profile"],
        "debug"
    );
    assert!(dispatch_record.get("payload_excerpt").is_none());
    assert_eq!(
        dispatch_record["handler_stderr_excerpt"][0]["redacted"],
        true
    );
    assert!(
        dispatch_record["handler_stderr_excerpt"][0]["excerpt"]
            .as_str()
            .expect("stderr excerpt should be a string")
            .starts_with("summary(")
    );
    assert_eq!(
        dispatch_record["handler_stdout_excerpt"][0]["redacted"],
        true
    );
}

#[test]
fn full_mode_debug_profile_permissive_still_requires_payload_capture_opt_in() {
    let temp = tempfile::tempdir().expect("tempdir should create");
    let root = temp.path();
    fs::create_dir_all(root.join(".sc-hooks")).expect(".sc-hooks dir should create");
    fs::write(
        root.join(".sc-hooks/config.toml"),
        r#"
[meta]
version = 1

[hooks]
PreToolUse = ["probe-plugin"]

[observability]
mode = "full"
full_profile = "debug"
redaction = "permissive"
capture_stdio = "bounded"
capture_payloads = false
"#,
    )
    .expect("config should write");
    fixtures::create_shell_plugin(
        &fixtures::plugin_path(root, "probe-plugin"),
        r#"{"contract_version":1,"name":"probe-plugin","mode":"sync","hooks":["PreToolUse"],"matchers":["Write"],"requires":{}}"#,
        r#"{"action":"proceed"}"#,
    );

    let output = DispatchHarness::new().run_sync(
        root,
        "PreToolUse",
        Some("Write"),
        Some(serde_json::json!({"tool_input": {"command": "secret-value"}})),
        None,
    );

    assert_eq!(
        output.status.code(),
        Some(sc_hooks_core::exit_codes::SUCCESS)
    );
    let (_, _, events) = read_full_audit_run(&root.join(".sc-hooks/audit"));
    let dispatch_record = &events[1];
    assert_eq!(
        dispatch_record["payload_capture_state"]["redaction"],
        "permissive"
    );
    assert_eq!(
        dispatch_record["payload_capture_state"]["capture_payloads"],
        false
    );
    assert_eq!(
        dispatch_record["payload_capture_state"]["payloads_included"],
        false
    );
    assert!(dispatch_record.get("payload_excerpt").is_none());
    assert!(
        dispatch_record["redaction_actions"]
            .as_array()
            .expect("redaction actions should be an array")
            .iter()
            .any(|value| value["action"] == "payload_capture_disabled")
    );
}

#[test]
fn full_mode_debug_profile_payload_capture_is_bounded_when_enabled() {
    let temp = tempfile::tempdir().expect("tempdir should create");
    let root = temp.path();
    fs::create_dir_all(root.join(".sc-hooks")).expect(".sc-hooks dir should create");
    fs::write(
        root.join(".sc-hooks/config.toml"),
        r#"
[meta]
version = 1

[hooks]
PreToolUse = ["probe-plugin"]

[observability]
mode = "full"
full_profile = "debug"
redaction = "permissive"
capture_stdio = "bounded"
capture_payloads = true
"#,
    )
    .expect("config should write");
    fixtures::create_shell_plugin(
        &fixtures::plugin_path(root, "probe-plugin"),
        r#"{"contract_version":1,"name":"probe-plugin","mode":"sync","hooks":["PreToolUse"],"matchers":["Write"],"requires":{}}"#,
        r#"{"action":"proceed"}"#,
    );

    let long_secret = "s".repeat(220);
    let output = DispatchHarness::new().run_sync(
        root,
        "PreToolUse",
        Some("Write"),
        Some(serde_json::json!({"tool_input": {"command": long_secret}})),
        None,
    );

    assert_eq!(
        output.status.code(),
        Some(sc_hooks_core::exit_codes::SUCCESS)
    );
    let (_, _, events) = read_full_audit_run(&root.join(".sc-hooks/audit"));
    let dispatch_record = &events[1];
    assert_eq!(
        dispatch_record["payload_capture_state"]["capture_payloads"],
        true
    );
    assert_eq!(
        dispatch_record["payload_capture_state"]["payloads_included"],
        true
    );
    let excerpt = dispatch_record["payload_excerpt"]["excerpt"]
        .as_str()
        .expect("payload excerpt should be present");
    assert!(excerpt.len() <= 160);
    assert_eq!(dispatch_record["payload_excerpt"]["truncated"], true);
    assert_eq!(dispatch_record["payload_excerpt"]["redacted"], false);
}

#[test]
fn resolution_failure_emits_standard_degraded_signal() {
    let temp = tempfile::tempdir().expect("tempdir should create");
    let root = temp.path();
    fixtures::write_minimal_config(root, "PreToolUse", "missing-plugin");

    let output = DispatchHarness::new().run_sync(
        root,
        "PreToolUse",
        Some("Write"),
        Some(serde_json::json!({"tool_input": {"command": "echo hi"}})),
        None,
    );

    assert_eq!(
        output.status.code(),
        Some(sc_hooks_core::exit_codes::RESOLUTION_ERROR)
    );
    let stderr = stderr_text(&output);
    assert!(stderr.contains("standard observability degraded before dispatch.complete"));
    assert!(stderr.contains("stage=resolution"));
    assert!(stderr.contains("hook=PreToolUse"));
    assert!(stderr.contains("event=Write"));
    assert!(!root.join(sc_hooks_core::OBSERVABILITY_LOG_PATH).exists());
}

#[test]
fn pre_spawn_validation_failure_emits_standard_degraded_signal() {
    let temp = tempfile::tempdir().expect("tempdir should create");
    let root = temp.path();
    fs::create_dir_all(root.join(".sc-hooks")).expect(".sc-hooks dir should create");
    fs::write(
        root.join(".sc-hooks/config.toml"),
        r#"
[meta]
version = 1

[hooks]
PreToolUse = ["probe-plugin"]
"#,
    )
    .expect("config should write");
    fixtures::create_shell_plugin(
        &fixtures::plugin_path(root, "probe-plugin"),
        r#"{"contract_version":1,"name":"probe-plugin","mode":"sync","hooks":["PreToolUse"],"matchers":["Write"],"requires":{"required_field":{"type":"string","validate":"non_empty"}}}"#,
        r#"{"action":"proceed"}"#,
    );

    let output = DispatchHarness::new().run_sync(
        root,
        "PreToolUse",
        Some("Write"),
        Some(serde_json::json!({"tool_input": {"command": "echo hi"}})),
        None,
    );

    assert_eq!(
        output.status.code(),
        Some(sc_hooks_core::exit_codes::VALIDATION_ERROR)
    );
    let stderr = stderr_text(&output);
    assert!(stderr.contains("standard observability degraded before dispatch.complete"));
    assert!(stderr.contains("stage=input_preparation"));
    assert!(stderr.contains("hook=PreToolUse"));
    assert!(stderr.contains("event=Write"));
    assert!(!root.join(sc_hooks_core::OBSERVABILITY_LOG_PATH).exists());
}

#[test]
fn blocked_dispatch_emits_warn_log_event() {
    let temp = tempfile::tempdir().expect("tempdir should create");
    let root = temp.path();
    fixtures::write_minimal_config(root, "PreToolUse", "probe-plugin");
    fixtures::create_shell_plugin(
        &fixtures::plugin_path(root, "probe-plugin"),
        r#"{"contract_version":1,"name":"probe-plugin","mode":"sync","hooks":["PreToolUse"],"matchers":["Write"],"requires":{}}"#,
        r#"{"action":"block","reason":"blocked by policy"}"#,
    );

    let output = DispatchHarness::new().run_sync(
        root,
        "PreToolUse",
        Some("Write"),
        Some(serde_json::json!({"tool_input": {"command": "echo hi"}})),
        None,
    );

    assert_eq!(
        output.status.code(),
        Some(sc_hooks_core::exit_codes::BLOCKED)
    );
    let log = read_last_log(root);
    assert_eq!(log["outcome"], "block");
    assert_eq!(log["level"], "Warn");
    assert_eq!(log["fields"]["mode"], "sync");
    assert_eq!(
        log["fields"]["handlers"],
        serde_json::json!(["probe-plugin"])
    );
    assert!(log["fields"]["total_ms"].as_u64().is_some());
    assert_eq!(log["fields"]["exit"], sc_hooks_core::exit_codes::BLOCKED);
    let fields = log["fields"]
        .as_object()
        .expect("fields should be an object");
    let result = log["fields"]["results"][0]
        .as_object()
        .expect("result should be an object");
    assert_eq!(result["action"], "block");
    assert!(!result.contains_key("disabled"));
    assert!(!fields.contains_key("ai_notification"));
}

#[cfg(unix)]
#[test]
fn invalid_json_dispatch_emits_error_log_and_disables_plugin() {
    let temp = tempfile::tempdir().expect("tempdir should create");
    let root = temp.path();
    let state_root = temp.path().join(".sc-hooks/state");
    fixtures::write_minimal_config(root, "PreToolUse", "probe-plugin");
    fixtures::create_shell_plugin_script(
        &fixtures::plugin_path(root, "probe-plugin"),
        r#"{"contract_version":1,"name":"probe-plugin","mode":"sync","hooks":["PreToolUse"],"matchers":["Write"],"requires":{}}"#,
        "cat >/dev/null\nprintf '%s' 'not-json'\n",
    );

    let output = DispatchHarness::new().run_sync_with_env(
        root,
        "PreToolUse",
        Some("Write"),
        Some(serde_json::json!({"tool_input": {"command": "echo hi"}})),
        Some("session-invalid-json"),
        &[(
            "SC_HOOKS_STATE_DIR",
            state_root.to_str().expect("state root should be utf8"),
        )],
    );

    assert_eq!(
        output.status.code(),
        Some(sc_hooks_core::exit_codes::PLUGIN_ERROR)
    );
    let log = read_last_log(root);
    assert_eq!(log["outcome"], "error");
    assert_eq!(log["level"], "Error");
    assert_eq!(
        log["fields"]["exit"],
        sc_hooks_core::exit_codes::PLUGIN_ERROR
    );
    assert_eq!(log["fields"]["results"][0]["action"], "error");
    assert_eq!(log["fields"]["results"][0]["error_type"], "invalid_json");
    assert_eq!(log["fields"]["results"][0]["disabled"], true);
    assert!(
        log["fields"]["ai_notification"]
            .as_str()
            .is_some_and(|message| message.contains("invalid JSON"))
    );
    assert!(session_disables_plugin_for_reason(
        root,
        "probe-plugin",
        "runtime-error"
    ));
}

#[cfg(unix)]
#[test]
fn timeout_dispatch_emits_error_log_and_disables_plugin() {
    let temp = tempfile::tempdir().expect("tempdir should create");
    let root = temp.path();
    let state_root = temp.path().join(".sc-hooks/state");
    fixtures::write_minimal_config(root, "PreToolUse", "probe-plugin");
    fixtures::create_shell_plugin_script(
        &fixtures::plugin_path(root, "probe-plugin"),
        r#"{"contract_version":1,"name":"probe-plugin","mode":"sync","hooks":["PreToolUse"],"matchers":["Write"],"timeout_ms":50,"requires":{}}"#,
        r#"cat >/dev/null
sleep 1
printf '%s\n' '{"action":"proceed"}'
"#,
    );

    let output = DispatchHarness::new().run_sync_with_env(
        root,
        "PreToolUse",
        Some("Write"),
        Some(serde_json::json!({"tool_input": {"command": "echo hi"}})),
        Some("session-timeout"),
        &[(
            "SC_HOOKS_STATE_DIR",
            state_root.to_str().expect("state root should be utf8"),
        )],
    );

    assert_eq!(
        output.status.code(),
        Some(sc_hooks_core::exit_codes::TIMEOUT)
    );
    let log = read_last_log(root);
    assert_eq!(log["outcome"], "error");
    assert_eq!(log["level"], "Error");
    assert_eq!(log["fields"]["exit"], sc_hooks_core::exit_codes::TIMEOUT);
    assert_eq!(log["fields"]["results"][0]["action"], "error");
    assert_eq!(log["fields"]["results"][0]["error_type"], "timeout");
    assert_eq!(log["fields"]["results"][0]["disabled"], true);
    assert!(
        log["fields"]["ai_notification"]
            .as_str()
            .is_some_and(|message| message.contains("timed out after 50ms"))
    );
    assert!(session_disables_plugin_for_reason(
        root,
        "probe-plugin",
        "runtime-error"
    ));
}

#[test]
fn root_divergence_notice_emits_structured_log_event() {
    let temp = tempfile::tempdir().expect("tempdir should create");
    let root = temp.path();
    fixtures::write_minimal_config(root, "Stop", "probe-plugin");
    let notice = RootDivergenceNotice::new(
        AiRootDir::new(root).expect("root dir"),
        root.join("mismatched"),
        SessionId::new("session-root-divergence").expect("session id"),
        HookType::Stop,
    )
    .expect("notice should validate")
    .encode()
    .expect("notice should encode");
    let runtime_output = serde_json::json!({
        "action": "proceed",
        "additionalContext": notice,
    })
    .to_string();
    fixtures::create_shell_plugin(
        &fixtures::plugin_path(root, "probe-plugin"),
        r#"{"contract_version":1,"name":"probe-plugin","mode":"sync","hooks":["Stop"],"matchers":["*"],"requires":{}}"#,
        &runtime_output,
    );

    let output = DispatchHarness::new().run_sync(
        root,
        "Stop",
        None,
        Some(serde_json::json!({
            "session_id": "session-root-divergence",
            "cwd": root,
            "stop_hook_active": false
        })),
        None,
    );

    assert_eq!(
        output.status.code(),
        Some(sc_hooks_core::exit_codes::SUCCESS)
    );
    let logs = read_all_logs(root);
    assert_eq!(logs.len(), 2);
    assert_eq!(logs[0]["action"], "session.root_divergence");
    assert_eq!(logs[0]["level"], "Error");
    assert_eq!(logs[0]["outcome"], "error");
    assert_eq!(
        logs[0]["fields"]["immutable_root"],
        root.to_str().expect("utf8")
    );
    assert_eq!(logs[0]["fields"]["session_id"], "session-root-divergence");
    assert_eq!(logs[0]["fields"]["hook_event"], "Stop");
    assert_eq!(logs[1]["action"], "dispatch.complete");
    assert_eq!(
        logs[1]["fields"]["results"][0]["warning"],
        "divergence in CLAUDE_PROJECT_DIR from ".to_string()
            + root.to_str().expect("utf8")
            + " to "
            + root.join("mismatched").to_str().expect("utf8")
            + " on Stop"
    );
}

#[test]
fn success_dispatch_emits_console_sink_line() {
    let temp = tempfile::tempdir().expect("tempdir should create");
    let root = temp.path();
    fixtures::write_minimal_config(root, "PreToolUse", "probe-plugin");
    fixtures::create_shell_plugin(
        &fixtures::plugin_path(root, "probe-plugin"),
        r#"{"contract_version":1,"name":"probe-plugin","mode":"sync","hooks":["PreToolUse"],"matchers":["Write"],"requires":{}}"#,
        r#"{"action":"proceed"}"#,
    );

    let output = DispatchHarness::new().run_sync_with_env(
        root,
        "PreToolUse",
        Some("Write"),
        Some(serde_json::json!({"tool_input": {"command": "echo hi"}})),
        None,
        &[("SC_HOOKS_ENABLE_CONSOLE_SINK", "1")],
    );

    assert_eq!(
        output.status.code(),
        Some(sc_hooks_core::exit_codes::SUCCESS)
    );
    let lines = console_lines(&output);
    assert_eq!(lines.len(), 1);
    let line = &lines[0];
    assert!(line.contains("INFO"));
    assert!(line.contains("hook"));
    assert!(line.contains("dispatch.complete"));
    assert!(line.contains("mode=sync"));
    assert!(line.contains("outcome=proceed"));
}

#[test]
fn blocked_dispatch_emits_console_sink_line() {
    let temp = tempfile::tempdir().expect("tempdir should create");
    let root = temp.path();
    fixtures::write_minimal_config(root, "PreToolUse", "probe-plugin");
    fixtures::create_shell_plugin(
        &fixtures::plugin_path(root, "probe-plugin"),
        r#"{"contract_version":1,"name":"probe-plugin","mode":"sync","hooks":["PreToolUse"],"matchers":["Write"],"requires":{}}"#,
        r#"{"action":"block","reason":"blocked by policy"}"#,
    );

    let output = DispatchHarness::new().run_sync_with_env(
        root,
        "PreToolUse",
        Some("Write"),
        Some(serde_json::json!({"tool_input": {"command": "echo hi"}})),
        None,
        &[("SC_HOOKS_ENABLE_CONSOLE_SINK", "1")],
    );

    assert_eq!(
        output.status.code(),
        Some(sc_hooks_core::exit_codes::BLOCKED)
    );
    let lines = console_lines(&output);
    assert_eq!(lines.len(), 1);
    let line = &lines[0];
    assert!(line.contains("WARN"));
    assert!(line.contains("dispatch.complete"));
    assert!(line.contains("mode=sync"));
    assert!(line.contains("outcome=block"));
}

#[cfg(unix)]
#[test]
fn invalid_json_dispatch_emits_console_sink_line() {
    let temp = tempfile::tempdir().expect("tempdir should create");
    let root = temp.path();
    let state_root = temp.path().join(".sc-hooks/state");
    fixtures::write_minimal_config(root, "PreToolUse", "probe-plugin");
    fixtures::create_shell_plugin_script(
        &fixtures::plugin_path(root, "probe-plugin"),
        r#"{"contract_version":1,"name":"probe-plugin","mode":"sync","hooks":["PreToolUse"],"matchers":["Write"],"requires":{}}"#,
        "cat >/dev/null\nprintf '%s' 'not-json'\n",
    );

    let output = DispatchHarness::new().run_sync_with_env(
        root,
        "PreToolUse",
        Some("Write"),
        Some(serde_json::json!({"tool_input": {"command": "echo hi"}})),
        Some("session-invalid-json-console"),
        &[
            ("SC_HOOKS_ENABLE_CONSOLE_SINK", "1"),
            (
                "SC_HOOKS_STATE_DIR",
                state_root.to_str().expect("state root should be utf8"),
            ),
        ],
    );

    let lines = console_lines(&output);
    assert_eq!(lines.len(), 1);
    let line = &lines[0];
    assert!(line.contains("ERROR"));
    assert!(line.contains("dispatch.complete"));
    assert!(line.contains("mode=sync"));
    assert!(line.contains("outcome=error"));
}

#[cfg(unix)]
#[test]
fn timeout_dispatch_emits_console_sink_line() {
    let temp = tempfile::tempdir().expect("tempdir should create");
    let root = temp.path();
    let state_root = temp.path().join(".sc-hooks/state");
    fixtures::write_minimal_config(root, "PreToolUse", "probe-plugin");
    fixtures::create_shell_plugin_script(
        &fixtures::plugin_path(root, "probe-plugin"),
        r#"{"contract_version":1,"name":"probe-plugin","mode":"sync","hooks":["PreToolUse"],"matchers":["Write"],"timeout_ms":50,"requires":{}}"#,
        r#"cat >/dev/null
sleep 1
printf '%s\n' '{"action":"proceed"}'
"#,
    );

    let output = DispatchHarness::new().run_sync_with_env(
        root,
        "PreToolUse",
        Some("Write"),
        Some(serde_json::json!({"tool_input": {"command": "echo hi"}})),
        Some("session-timeout-console"),
        &[
            ("SC_HOOKS_ENABLE_CONSOLE_SINK", "1"),
            (
                "SC_HOOKS_STATE_DIR",
                state_root.to_str().expect("state root should be utf8"),
            ),
        ],
    );

    let lines = console_lines(&output);
    assert_eq!(lines.len(), 1);
    let line = &lines[0];
    assert!(line.contains("ERROR"));
    assert!(line.contains("dispatch.complete"));
    assert!(line.contains("mode=sync"));
    assert!(line.contains("outcome=error"));
}
