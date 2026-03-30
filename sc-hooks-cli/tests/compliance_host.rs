use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

use sc_hooks_test::compliance::{
    ContractScenario, ContractScenarioResult, FnHostDispatchProbe, run_contract_behavior_suite,
};
use sc_hooks_test::fixtures;

struct CliBinaryProbe {
    binary: String,
}

impl CliBinaryProbe {
    fn new() -> Self {
        let current = std::env::current_exe().expect("test binary path should resolve");
        let binary = current
            .parent()
            .and_then(|deps| deps.parent())
            .map(|debug_dir| {
                debug_dir.join(if cfg!(windows) {
                    "sc-hooks-cli.exe"
                } else {
                    "sc-hooks-cli"
                })
            })
            .expect("target/debug directory should resolve");
        Self {
            binary: binary.display().to_string(),
        }
    }
}

impl CliBinaryProbe {
    fn run_scenario(&self, scenario: ContractScenario) -> Result<ContractScenarioResult, String> {
        let temp = tempfile::tempdir().map_err(|err| err.to_string())?;
        let root = temp.path();
        let plugin_name = "probe-plugin";
        let plugin_path = fixtures::plugin_path(root, plugin_name);
        let marker_path = root.join("runtime.marker");

        let (hook, event, mode, session_id, payload) = match scenario {
            ContractScenario::AbsentPayload => {
                fixtures::write_minimal_config(root, "PreToolUse", plugin_name);
                fixtures::create_shell_plugin_script(
                    &plugin_path,
                    r#"{"contract_version":1,"name":"probe-plugin","mode":"sync","hooks":["PreToolUse"],"matchers":["Write"],"requires":{}}"#,
                    r#"input="$(cat)"
if printf '%s' "$input" | grep -q '"payload"'; then
  printf '%s\n' '{"action":"error","message":"payload should be absent"}'
else
  printf '%s\n' '{"action":"proceed"}'
fi
"#,
                );
                ("PreToolUse", Some("Write"), "sync", None, None)
            }
            ContractScenario::InvalidOutput => {
                fixtures::write_minimal_config(root, "PreToolUse", plugin_name);
                fixtures::create_shell_plugin_script(
                    &plugin_path,
                    r#"{"contract_version":1,"name":"probe-plugin","mode":"sync","hooks":["PreToolUse"],"matchers":["Write"],"requires":{}}"#,
                    "cat >/dev/null\nprintf '%s' 'not-json'\n",
                );
                (
                    "PreToolUse",
                    Some("Write"),
                    "sync",
                    Some("session-invalid-output"),
                    Some(serde_json::json!({"tool_input":{"command":"echo hi"}})),
                )
            }
            ContractScenario::MultipleJsonObjects => {
                fixtures::write_minimal_config(root, "PreToolUse", plugin_name);
                fixtures::create_shell_plugin_script(
                    &plugin_path,
                    r#"{"contract_version":1,"name":"probe-plugin","mode":"sync","hooks":["PreToolUse"],"matchers":["Write"],"requires":{}}"#,
                    r#"cat >/dev/null
printf '%s' '{"action":"proceed"}{"action":"error"}'
"#,
                );
                (
                    "PreToolUse",
                    Some("Write"),
                    "sync",
                    None,
                    Some(serde_json::json!({"tool_input":{"command":"echo hi"}})),
                )
            }
            ContractScenario::AsyncBlockMisuse => {
                fixtures::write_minimal_config(root, "PostToolUse", plugin_name);
                fixtures::create_shell_plugin_script(
                    &plugin_path,
                    r#"{"contract_version":1,"name":"probe-plugin","mode":"async","hooks":["PostToolUse"],"matchers":["Write"],"requires":{}}"#,
                    r#"cat >/dev/null
printf '%s\n' '{"action":"block","reason":"no async block"}'
"#,
                );
                (
                    "PostToolUse",
                    Some("Write"),
                    "async",
                    Some("session-async-block"),
                    Some(serde_json::json!({"tool_input":{"command":"echo hi"}})),
                )
            }
            ContractScenario::MatcherFiltering => {
                fixtures::write_minimal_config(root, "PreToolUse", plugin_name);
                let runtime = format!(
                    "cat >/dev/null\n: > \"{}\"\nprintf '%s\\n' '{{\"action\":\"proceed\"}}'\n",
                    marker_path.display()
                );
                fixtures::create_shell_plugin_script(
                    &plugin_path,
                    r#"{"contract_version":1,"name":"probe-plugin","mode":"sync","hooks":["PreToolUse"],"matchers":["Write"],"requires":{}}"#,
                    &runtime,
                );
                (
                    "PreToolUse",
                    Some("Read"),
                    "sync",
                    None,
                    Some(serde_json::json!({"tool_input":{"command":"echo hi"}})),
                )
            }
            ContractScenario::Timeout => {
                fixtures::write_minimal_config(root, "PreToolUse", plugin_name);
                fixtures::create_shell_plugin_script(
                    &plugin_path,
                    r#"{"contract_version":1,"name":"probe-plugin","mode":"sync","hooks":["PreToolUse"],"matchers":["Write"],"timeout_ms":50,"requires":{}}"#,
                    r#"cat >/dev/null
sleep 1
printf '%s\n' '{"action":"proceed"}'
"#,
                );
                (
                    "PreToolUse",
                    Some("Write"),
                    "sync",
                    Some("session-timeout"),
                    Some(serde_json::json!({"tool_input":{"command":"echo hi"}})),
                )
            }
        };

        let mut command = Command::new(&self.binary);
        command.current_dir(root).arg("run").arg(hook);
        if let Some(event) = event {
            command.arg(event);
        }
        match mode {
            "async" => {
                command.arg("--async");
            }
            _ => {
                command.arg("--sync");
            }
        }
        if let Some(session_id) = session_id {
            command.env("SC_HOOK_SESSION_ID", session_id);
        }
        command.env("SC_HOOKS_STATE_DIR", root.join(".sc-hooks/state"));

        let output = if let Some(payload) = payload {
            let mut child = command
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|err| err.to_string())?;
            if let Some(mut stdin) = child.stdin.take() {
                use std::io::Write;
                let body = serde_json::to_vec(&payload).map_err(|err| err.to_string())?;
                stdin.write_all(&body).map_err(|err| err.to_string())?;
            }
            child.wait_with_output().map_err(|err| err.to_string())?
        } else {
            command.output().map_err(|err| err.to_string())?
        };

        let log_line = read_last_line(&root.join(sc_hooks_core::OBSERVABILITY_LOG_PATH));
        let session_state = fs::read_to_string(root.join(".sc-hooks/state/session.json")).ok();

        Ok(ContractScenarioResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            last_log_line: log_line,
            session_state,
            marker_exists: marker_path.exists(),
        })
    }
}

fn read_last_line(path: &Path) -> Option<String> {
    let rendered = fs::read_to_string(path).ok()?;
    rendered.lines().last().map(str::to_string)
}

#[test]
fn shared_compliance_suite_exercises_actual_host_dispatch_path() {
    let probe = CliBinaryProbe::new();
    let adapter = FnHostDispatchProbe::new(|scenario| probe.run_scenario(scenario));
    let checks = run_contract_behavior_suite(&adapter);

    for check in &checks {
        assert!(check.passed, "{}: {:?}", check.name, check.detail);
    }
}
