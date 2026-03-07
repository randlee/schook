pub mod conditions;
pub mod dispatch;
pub mod events;
pub mod exit_codes;
pub mod manifest;
pub mod results;
pub mod validation;

#[cfg(test)]
mod tests {
    use super::exit_codes;

    #[test]
    fn exit_code_table_has_expected_range() {
        let codes: Vec<i32> = exit_codes::all().iter().map(|entry| entry.code).collect();
        assert_eq!(codes, (0..=10).collect::<Vec<_>>());
    }
}
