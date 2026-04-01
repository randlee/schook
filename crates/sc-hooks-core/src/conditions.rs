use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
/// Supported payload-condition operators.
pub enum ConditionOperator {
    /// Passes when the path resolves to a non-null value.
    Exists,
    /// Passes when the path is absent or null.
    NotExists,
    /// Passes when the resolved value equals the configured value.
    Equals,
    /// Passes when the resolved value does not equal the configured value.
    NotEquals,
    /// Passes when the resolved string contains the configured value.
    Contains,
    /// Passes when the resolved string does not contain the configured value.
    NotContains,
    /// Passes when the resolved string starts with the configured value.
    StartsWith,
    /// Passes when the resolved string matches a glob pattern.
    Matches,
    /// Passes when the resolved string equals one of the configured values.
    OneOf,
    /// Passes when the resolved string matches a regex pattern.
    Regex,
    /// Passes when the resolved numeric value is greater than the configured value.
    Gt,
    /// Passes when the resolved numeric value is less than the configured value.
    Lt,
    /// Passes when the resolved numeric value is greater than or equal to the configured value.
    Gte,
    /// Passes when the resolved numeric value is less than or equal to the configured value.
    Lte,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// One manifest-declared payload-condition entry.
pub struct PayloadCondition {
    /// Dot-separated path within the payload object.
    pub path: String,
    /// Operator used for the comparison.
    pub op: ConditionOperator,
    #[serde(default)]
    /// Comparison value when the operator requires one.
    pub value: Option<Value>,
}
