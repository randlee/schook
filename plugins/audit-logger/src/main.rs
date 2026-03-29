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
