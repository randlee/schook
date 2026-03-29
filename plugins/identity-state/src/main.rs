fn main() {
    if std::env::args().any(|arg| arg == "--manifest") {
        let manifest = serde_json::json!({
            "contract_version": 1,
            "name": "identity-state",
            "mode": "sync",
            "hooks": ["PreToolUse","PostToolUse"],
            "matchers": ["*"],
            "requires": {},
            "timeout_ms": 5000
        });
        println!(
            "{}",
            serde_json::to_string(&manifest).expect("manifest should serialize")
        );
        return;
    }

    let mut input = String::new();
    use std::io::Read;
    let _ = std::io::stdin().read_to_string(&mut input);

    let result = serde_json::json!({"action":"proceed"});
    println!(
        "{}",
        serde_json::to_string(&result).expect("result should serialize")
    );
}
