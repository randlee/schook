//! Reference scaffold plugin binary for audit logging experiments.

fn print_json(value: &serde_json::Value, label: &str) {
    match serde_json::to_string(value) {
        Ok(rendered) => println!("{rendered}"),
        Err(err) => {
            eprintln!("failed to serialize {label}: {err}");
            std::process::exit(1);
        }
    }
}

fn main() {
    if std::env::args().any(|arg| arg == "--manifest") {
        let manifest = serde_json::json!({
            "contract_version": 1,
            "name": "audit-logger",
            "mode": "async",
            "hooks": ["PreToolUse","PostToolUse","PreCompact","PostCompact"],
            "matchers": ["*"],
            "requires": {},
            "response_time": {"min_ms": 100, "max_ms": 1000}
        });
        print_json(&manifest, "manifest");
        return;
    }

    let mut input = String::new();
    use std::io::Read;
    let _ = std::io::stdin().read_to_string(&mut input);

    let result = serde_json::json!({"action":"proceed"});
    print_json(&result, "result");
}
