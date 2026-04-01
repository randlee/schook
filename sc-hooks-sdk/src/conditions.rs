use glob::Pattern;
use regex::Regex;
use serde_json::Value;
use thiserror::Error;

use sc_hooks_core::conditions::{ConditionOperator, PayloadCondition};

#[derive(Debug, Error)]
/// Errors produced while validating or evaluating payload conditions.
pub enum ConditionError {
    /// The dot-separated payload path was invalid.
    #[error("condition path `{path}` is invalid")]
    InvalidPath {
        /// Offending dot-separated payload path.
        path: String,
    },

    /// The operator required a comparison value but none was provided.
    #[error("condition `{path}` with operator `{op:?}` requires a value")]
    MissingValue {
        /// Offending dot-separated payload path.
        path: String,
        /// Operator that required a value.
        op: ConditionOperator,
    },

    /// The provided comparison value was not compatible with the operator.
    #[error("condition `{path}` has invalid value for `{op:?}`")]
    InvalidValue {
        /// Offending dot-separated payload path.
        path: String,
        /// Operator that rejected the value.
        op: ConditionOperator,
    },

    /// A glob pattern could not be compiled.
    #[error("invalid glob pattern `{pattern}`: {source}")]
    InvalidGlob {
        /// Original glob pattern.
        pattern: String,
        #[source]
        /// Underlying glob parser error.
        source: glob::PatternError,
    },

    /// A regex pattern could not be compiled.
    #[error("invalid regex `{pattern}`: {source}")]
    InvalidRegex {
        /// Original regex pattern.
        pattern: String,
        #[source]
        /// Underlying regex parser error.
        source: regex::Error,
    },
}

/// Validates a list of manifest payload conditions.
pub fn validate_payload_conditions(conditions: &[PayloadCondition]) -> Result<(), ConditionError> {
    for condition in conditions {
        validate_path(&condition.path)?;
        validate_condition_value(condition)?;
    }

    Ok(())
}

/// Evaluates payload conditions against an optional payload object.
pub fn evaluate_payload_conditions(
    conditions: &[PayloadCondition],
    payload: Option<&Value>,
) -> Result<bool, ConditionError> {
    validate_payload_conditions(conditions)?;

    for condition in conditions {
        if !evaluate_single(condition, payload)? {
            return Ok(false);
        }
    }

    Ok(true)
}

fn validate_path(path: &str) -> Result<(), ConditionError> {
    if path.is_empty() || path.split('.').any(|segment| segment.trim().is_empty()) {
        return Err(ConditionError::InvalidPath {
            path: path.to_string(),
        });
    }

    Ok(())
}

fn validate_condition_value(condition: &PayloadCondition) -> Result<(), ConditionError> {
    match condition.op {
        ConditionOperator::Exists | ConditionOperator::NotExists => Ok(()),
        ConditionOperator::Equals
        | ConditionOperator::NotEquals
        | ConditionOperator::Contains
        | ConditionOperator::NotContains
        | ConditionOperator::StartsWith
        | ConditionOperator::Matches
        | ConditionOperator::Regex
        | ConditionOperator::Gt
        | ConditionOperator::Lt
        | ConditionOperator::Gte
        | ConditionOperator::Lte => {
            let value = condition
                .value
                .as_ref()
                .ok_or_else(|| ConditionError::MissingValue {
                    path: condition.path.clone(),
                    op: condition.op.clone(),
                })?;

            if matches!(
                condition.op,
                ConditionOperator::Gt
                    | ConditionOperator::Lt
                    | ConditionOperator::Gte
                    | ConditionOperator::Lte
            ) && value.as_f64().is_none()
            {
                return Err(ConditionError::InvalidValue {
                    path: condition.path.clone(),
                    op: condition.op.clone(),
                });
            }

            if matches!(
                condition.op,
                ConditionOperator::Matches | ConditionOperator::Regex
            ) && value.as_str().is_none()
            {
                return Err(ConditionError::InvalidValue {
                    path: condition.path.clone(),
                    op: condition.op.clone(),
                });
            }

            Ok(())
        }
        ConditionOperator::OneOf => {
            let value = condition
                .value
                .as_ref()
                .ok_or_else(|| ConditionError::MissingValue {
                    path: condition.path.clone(),
                    op: condition.op.clone(),
                })?;
            let Some(values) = value.as_array() else {
                return Err(ConditionError::InvalidValue {
                    path: condition.path.clone(),
                    op: condition.op.clone(),
                });
            };
            if values.iter().any(|entry| entry.as_str().is_none()) {
                return Err(ConditionError::InvalidValue {
                    path: condition.path.clone(),
                    op: condition.op.clone(),
                });
            }
            Ok(())
        }
        _ => Err(ConditionError::InvalidValue {
            path: condition.path.clone(),
            op: condition.op.clone(),
        }),
    }
}

fn evaluate_single(
    condition: &PayloadCondition,
    payload: Option<&Value>,
) -> Result<bool, ConditionError> {
    let resolved = payload.and_then(|value| resolve_path(value, &condition.path));

    match condition.op {
        ConditionOperator::Exists => Ok(resolved.is_some_and(|value| !value.is_null())),
        ConditionOperator::NotExists => {
            Ok(resolved.is_none() || resolved.is_some_and(Value::is_null))
        }
        ConditionOperator::Equals => Ok(match (resolved, condition.value.as_ref()) {
            (Some(left), Some(right)) => left == right,
            _ => false,
        }),
        ConditionOperator::NotEquals => Ok(match (resolved, condition.value.as_ref()) {
            (Some(left), Some(right)) => left != right,
            _ => false,
        }),
        ConditionOperator::Contains => Ok(
            match (
                resolved.and_then(Value::as_str),
                condition.value.as_ref().and_then(Value::as_str),
            ) {
                (Some(left), Some(needle)) => left.contains(needle),
                _ => false,
            },
        ),
        ConditionOperator::NotContains => Ok(
            match (
                resolved.and_then(Value::as_str),
                condition.value.as_ref().and_then(Value::as_str),
            ) {
                (Some(left), Some(needle)) => !left.contains(needle),
                _ => false,
            },
        ),
        ConditionOperator::StartsWith => Ok(
            match (
                resolved.and_then(Value::as_str),
                condition.value.as_ref().and_then(Value::as_str),
            ) {
                (Some(left), Some(prefix)) => left.starts_with(prefix),
                _ => false,
            },
        ),
        ConditionOperator::Matches => {
            let Some(pattern) = condition.value.as_ref().and_then(Value::as_str) else {
                return Ok(false);
            };
            let compiled = Pattern::new(pattern).map_err(|source| ConditionError::InvalidGlob {
                pattern: pattern.to_string(),
                source,
            })?;
            Ok(resolved
                .and_then(Value::as_str)
                .is_some_and(|target| compiled.matches(target)))
        }
        ConditionOperator::OneOf => {
            let Some(candidates) = condition.value.as_ref().and_then(Value::as_array) else {
                return Ok(false);
            };
            let Some(target) = resolved.and_then(Value::as_str) else {
                return Ok(false);
            };

            Ok(candidates
                .iter()
                .filter_map(Value::as_str)
                .any(|candidate| candidate == target))
        }
        ConditionOperator::Regex => {
            let Some(pattern) = condition.value.as_ref().and_then(Value::as_str) else {
                return Ok(false);
            };
            let compiled = Regex::new(pattern).map_err(|source| ConditionError::InvalidRegex {
                pattern: pattern.to_string(),
                source,
            })?;
            Ok(resolved
                .and_then(Value::as_str)
                .is_some_and(|target| compiled.is_match(target)))
        }
        ConditionOperator::Gt
        | ConditionOperator::Lt
        | ConditionOperator::Gte
        | ConditionOperator::Lte => {
            let Some(left) = resolved.and_then(Value::as_f64) else {
                return Ok(false);
            };
            let Some(right) = condition.value.as_ref().and_then(Value::as_f64) else {
                return Ok(false);
            };

            Ok(match condition.op {
                ConditionOperator::Gt => left > right,
                ConditionOperator::Lt => left < right,
                ConditionOperator::Gte => left >= right,
                ConditionOperator::Lte => left <= right,
                _ => {
                    return Err(ConditionError::InvalidValue {
                        path: condition.path.clone(),
                        op: condition.op.clone(),
                    });
                }
            })
        }
        _ => Err(ConditionError::InvalidValue {
            path: condition.path.clone(),
            op: condition.op.clone(),
        }),
    }
}

fn resolve_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = value;
    for segment in path.split('.') {
        current = current.as_object()?.get(segment)?;
    }
    Some(current)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sc_hooks_core::conditions::{ConditionOperator, PayloadCondition};

    fn payload() -> Value {
        serde_json::json!({
            "tool_input": {
                "command": "atm send team-lead hi",
                "subagent_type": "rust-developer",
                "file_path": "src/main.rs",
                "attempt": 3,
                "score": 7.5
            }
        })
    }

    fn cond(path: &str, op: ConditionOperator, value: Option<Value>) -> PayloadCondition {
        PayloadCondition {
            path: path.to_string(),
            op,
            value,
        }
    }

    #[test]
    fn supports_all_required_and_extended_operators() {
        let payload = payload();
        let checks = vec![
            cond("tool_input.command", ConditionOperator::Exists, None),
            cond("tool_input.missing", ConditionOperator::NotExists, None),
            cond(
                "tool_input.subagent_type",
                ConditionOperator::Equals,
                Some(Value::String("rust-developer".to_string())),
            ),
            cond(
                "tool_input.subagent_type",
                ConditionOperator::NotEquals,
                Some(Value::String("scrum-master".to_string())),
            ),
            cond(
                "tool_input.command",
                ConditionOperator::Contains,
                Some(Value::String("atm".to_string())),
            ),
            cond(
                "tool_input.command",
                ConditionOperator::NotContains,
                Some(Value::String("python".to_string())),
            ),
            cond(
                "tool_input.file_path",
                ConditionOperator::StartsWith,
                Some(Value::String("src/".to_string())),
            ),
            cond(
                "tool_input.file_path",
                ConditionOperator::Matches,
                Some(Value::String("src/**/*.rs".to_string())),
            ),
            cond(
                "tool_input.subagent_type",
                ConditionOperator::OneOf,
                Some(Value::Array(vec![
                    Value::String("scrum-master".to_string()),
                    Value::String("rust-developer".to_string()),
                ])),
            ),
            cond(
                "tool_input.command",
                ConditionOperator::Regex,
                Some(Value::String("^atm\\s".to_string())),
            ),
            cond(
                "tool_input.attempt",
                ConditionOperator::Gt,
                Some(Value::from(2)),
            ),
            cond(
                "tool_input.attempt",
                ConditionOperator::Lt,
                Some(Value::from(10)),
            ),
            cond(
                "tool_input.score",
                ConditionOperator::Gte,
                Some(Value::from(7.5)),
            ),
            cond(
                "tool_input.score",
                ConditionOperator::Lte,
                Some(Value::from(8)),
            ),
        ];

        let result = evaluate_payload_conditions(&checks, Some(&payload))
            .expect("all supported operators should evaluate cleanly");
        assert!(result);
    }

    #[test]
    fn fails_fast_when_condition_does_not_match() {
        let payload = payload();
        let checks = vec![cond(
            "tool_input.command",
            ConditionOperator::Contains,
            Some(Value::String("non-existent".to_string())),
        )];
        assert!(!evaluate_payload_conditions(&checks, Some(&payload)).unwrap());
    }
}
