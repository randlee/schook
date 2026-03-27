use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ConditionOperator {
    Exists,
    NotExists,
    Equals,
    NotEquals,
    Contains,
    NotContains,
    StartsWith,
    Matches,
    OneOf,
    Regex,
    Gt,
    Lt,
    Gte,
    Lte,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PayloadCondition {
    pub path: String,
    pub op: ConditionOperator,
    #[serde(default)]
    pub value: Option<Value>,
}
