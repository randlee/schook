#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Documentation entry describing one CLI exit code.
pub struct ExitCodeInfo {
    /// Numeric exit code.
    pub code: i32,
    /// Symbolic exit code name.
    pub name: &'static str,
    /// Human-readable meaning of the code.
    pub meaning: &'static str,
    /// Suggested remediation for operators.
    pub remediation: &'static str,
}

/// Successful command completion.
pub const SUCCESS: i32 = 0;
/// A sync hook blocked execution.
pub const BLOCKED: i32 = 1;
/// Plugin protocol or runtime failure.
pub const PLUGIN_ERROR: i32 = 2;
/// Configuration parsing or validation failure.
pub const CONFIG_ERROR: i32 = 3;
/// Plugin resolution or manifest-load failure.
pub const RESOLUTION_ERROR: i32 = 4;
/// Metadata validation failure.
pub const VALIDATION_ERROR: i32 = 5;
/// Hook timeout.
pub const TIMEOUT: i32 = 6;
/// Audit command failure.
pub const AUDIT_FAILURE: i32 = 7;
/// Reserved for future use.
pub const RESERVED_8: i32 = 8;
/// Reserved for future use.
pub const RESERVED_9: i32 = 9;
/// Unexpected host/internal failure.
pub const INTERNAL_ERROR: i32 = 10;

/// Full exit-code reference table used by CLI help output.
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
        remediation: "Inspect observability output and run `sc-hooks test <plugin>`.",
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
        remediation: "Check hook chains and ensure referenced plugins exist and are executable.",
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
        remediation: "Inspect observability output and report a bug if reproducible.",
    },
];

/// Returns the complete exit-code reference table.
pub fn all() -> &'static [ExitCodeInfo] {
    &EXIT_CODE_TABLE
}

/// Renders a human-readable exit-code reference block.
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
    fn renders_full_exit_code_reference_with_remediation() {
        let rendered = render_reference();
        for entry in all() {
            assert!(
                rendered.contains(&entry.code.to_string()),
                "missing code {}",
                entry.code
            );
            assert!(
                rendered.contains(entry.name),
                "missing exit code name {}",
                entry.name
            );
            assert!(
                rendered.contains(entry.remediation),
                "missing remediation for {}",
                entry.name
            );
        }
        assert!(rendered.contains("Exit Code Reference:"));
    }
}
