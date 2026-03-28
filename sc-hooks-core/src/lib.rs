pub mod conditions;
pub mod context;
pub mod dispatch;
pub mod errors;
pub mod events;
pub mod exit_codes;
pub mod manifest;
pub mod results;
pub mod session;
pub mod storage;
pub mod tools;
pub mod validation;

pub const OBSERVABILITY_ROOT: &str = ".sc-hooks/observability";
pub const OBSERVABILITY_LOG_PATH: &str = ".sc-hooks/observability/sc-hooks/logs/sc-hooks.log.jsonl";

#[cfg(test)]
mod tests {
    use super::exit_codes;

    #[test]
    fn exit_code_table_has_expected_range() {
        let codes: Vec<i32> = exit_codes::all().iter().map(|entry| entry.code).collect();
        assert_eq!(codes, (0..=10).collect::<Vec<_>>());
    }
}
