#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExitCodeInfo {
    pub code: i32,
    pub name: &'static str,
    pub meaning: &'static str,
    pub remediation: &'static str,
}

pub const SUCCESS: i32 = 0;
pub const BLOCKED: i32 = 1;
pub const PLUGIN_ERROR: i32 = 2;
pub const CONFIG_ERROR: i32 = 3;
pub const RESOLUTION_ERROR: i32 = 4;
pub const VALIDATION_ERROR: i32 = 5;
pub const TIMEOUT: i32 = 6;
pub const AUDIT_FAILURE: i32 = 7;
pub const RESERVED_8: i32 = 8;
pub const RESERVED_9: i32 = 9;
pub const INTERNAL_ERROR: i32 = 10;

pub const EXIT_CODE_TABLE: [ExitCodeInfo; 11] = [
    ExitCodeInfo {
        code: SUCCESS,
        name: "SUCCESS",
        meaning: "All handlers returned proceed.",
        remediation: "No action required.",
    },
    ExitCodeInfo {
        code: BLOCKED,
        name: "BLOCKED",
        meaning: "A sync handler returned action=block.",
        remediation: "Review stderr for block reason and adjust plugin policy if appropriate.",
    },
    ExitCodeInfo {
        code: PLUGIN_ERROR,
        name: "PLUGIN_ERROR",
        meaning: "A handler returned action=error or violated protocol.",
        remediation: "Inspect dispatch logs and run `sc-hooks test <plugin>`.",
    },
    ExitCodeInfo {
        code: CONFIG_ERROR,
        name: "CONFIG_ERROR",
        meaning: "Config file missing, malformed, or invalid.",
        remediation: "Run `sc-hooks config` and fix `.sc-hooks/config.toml`.",
    },
    ExitCodeInfo {
        code: RESOLUTION_ERROR,
        name: "RESOLUTION_ERROR",
        meaning: "One or more handlers could not be resolved.",
        remediation: "Check hook chains and ensure builtins/plugins exist and are executable.",
    },
    ExitCodeInfo {
        code: VALIDATION_ERROR,
        name: "VALIDATION_ERROR",
        meaning: "Metadata validation failed for handler requirements.",
        remediation: "Fix missing/invalid metadata fields required by the handler manifest.",
    },
    ExitCodeInfo {
        code: TIMEOUT,
        name: "TIMEOUT",
        meaning: "A handler exceeded its timeout.",
        remediation: "Increase `timeout_ms` or optimize handler behavior.",
    },
    ExitCodeInfo {
        code: AUDIT_FAILURE,
        name: "AUDIT_FAILURE",
        meaning: "`sc-hooks audit` found validation errors.",
        remediation: "Resolve all audit findings and rerun `sc-hooks audit`.",
    },
    ExitCodeInfo {
        code: RESERVED_8,
        name: "RESERVED_8",
        meaning: "Reserved for future use.",
        remediation: "Consult future sc-hooks release notes for this code.",
    },
    ExitCodeInfo {
        code: RESERVED_9,
        name: "RESERVED_9",
        meaning: "Reserved for future use.",
        remediation: "Consult future sc-hooks release notes for this code.",
    },
    ExitCodeInfo {
        code: INTERNAL_ERROR,
        name: "INTERNAL_ERROR",
        meaning: "Unexpected host error (panic or I/O failure).",
        remediation: "Collect logs, rerun with debug logging, and report a bug if reproducible.",
    },
];

pub fn all() -> &'static [ExitCodeInfo] {
    &EXIT_CODE_TABLE
}

pub fn render_reference() -> String {
    let mut output = String::from("Exit Code Reference:\n");
    for entry in all() {
        output.push_str(&format!(
            "  {code:>2}  {name:<17} {meaning}\n      -> {remediation}\n",
            code = entry.code,
            name = entry.name,
            meaning = entry.meaning,
            remediation = entry.remediation,
        ));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn includes_all_codes_zero_through_ten() {
        let codes: Vec<i32> = all().iter().map(|entry| entry.code).collect();
        assert_eq!(codes, (0..=10).collect::<Vec<_>>());
    }

    #[test]
    fn renders_human_readable_reference() {
        let rendered = render_reference();
        assert!(rendered.contains("0  SUCCESS"));
        assert!(rendered.contains("10  INTERNAL_ERROR"));
        assert!(rendered.contains("Reserved for future use"));
    }
}
