//! Reference scaffold plugin binary for policy-enforcer experiments.

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
            "name": "policy-enforcer",
            "mode": "sync",
            "hooks": ["PreToolUse"],
            "matchers": ["Write","Bash"],
            "requires": {},
            "timeout_ms": 5000
        });
        print_json(&manifest, "manifest");
        return;
    }

    use std::io::Read;
    let mut input = String::new();
    if let Err(err) = std::io::stdin().read_to_string(&mut input) {
        eprintln!("failed to read stdin: {err}");
        std::process::exit(1);
    }

    let result = serde_json::json!({"action":"proceed"});
    print_json(&result, "result");
}
