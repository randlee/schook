use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FieldType {
    String,
    Number,
    Integer,
    Boolean,
    Object,
    Array,
    Any,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationRule {
    NonEmpty,
    DirExists,
    FileExists,
    PathResolves,
    PositiveInt,
    OneOf,
}

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
