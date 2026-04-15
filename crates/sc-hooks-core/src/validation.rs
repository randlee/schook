use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
/// Supported metadata field types in plugin manifests.
pub enum FieldType {
    /// JSON string.
    String,
    /// Any JSON number.
    Number,
    /// Integral JSON number.
    Integer,
    /// JSON boolean.
    Boolean,
    /// JSON object.
    Object,
    /// JSON array.
    Array,
    /// Any JSON value.
    Any,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Supported validation rules in plugin manifests.
pub enum ValidationRule {
    /// String must be non-empty after trimming.
    NonEmpty,
    /// Path must exist and be a directory.
    DirExists,
    /// Path must exist and be a file.
    FileExists,
    /// Path must resolve via canonicalization.
    PathResolves,
    /// Integer must be positive.
    PositiveInt,
    /// String must equal one of a configured set of values.
    OneOf,
}

/// Parses a serialized validation rule into the typed representation used by the host.
pub fn parse_validation_rule(raw: &str) -> Option<(ValidationRule, Option<Vec<String>>)> {
    if raw == "non_empty" {
        return Some((ValidationRule::NonEmpty, None));
    }
    if raw == "dir_exists" {
        return Some((ValidationRule::DirExists, None));
    }
    if raw == "file_exists" {
        return Some((ValidationRule::FileExists, None));
    }
    if raw == "path_resolves" {
        return Some((ValidationRule::PathResolves, None));
    }
    if raw == "positive_int" {
        return Some((ValidationRule::PositiveInt, None));
    }

    raw.strip_prefix("one_of:").map(|rest| {
        let values = rest
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        (ValidationRule::OneOf, Some(values))
    })
}

#[cfg(test)]
mod tests {
    use super::{ValidationRule, parse_validation_rule};

    #[test]
    fn parses_non_empty_rule() {
        assert_eq!(
            parse_validation_rule("non_empty"),
            Some((ValidationRule::NonEmpty, None))
        );
    }

    #[test]
    fn parses_dir_exists_rule() {
        assert_eq!(
            parse_validation_rule("dir_exists"),
            Some((ValidationRule::DirExists, None))
        );
    }

    #[test]
    fn parses_file_exists_rule() {
        assert_eq!(
            parse_validation_rule("file_exists"),
            Some((ValidationRule::FileExists, None))
        );
    }

    #[test]
    fn parses_path_resolves_rule() {
        assert_eq!(
            parse_validation_rule("path_resolves"),
            Some((ValidationRule::PathResolves, None))
        );
    }

    #[test]
    fn parses_positive_int_rule() {
        assert_eq!(
            parse_validation_rule("positive_int"),
            Some((ValidationRule::PositiveInt, None))
        );
    }

    #[test]
    fn parses_one_of_rule() {
        assert_eq!(
            parse_validation_rule("one_of: alpha, beta , ,gamma"),
            Some((
                ValidationRule::OneOf,
                Some(vec![
                    "alpha".to_string(),
                    "beta".to_string(),
                    "gamma".to_string()
                ])
            ))
        );
    }

    #[test]
    fn unknown_rule_returns_none() {
        assert_eq!(parse_validation_rule("not_a_rule"), None);
    }
}
