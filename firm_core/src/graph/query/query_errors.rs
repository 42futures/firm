//! Error types for query execution

use std::fmt;

/// Errors that can occur during query execution
#[derive(Debug, Clone, PartialEq)]
pub enum QueryError {
    /// Operator is not supported for the given field type
    UnsupportedOperator {
        field_type: String,
        operator: String,
        supported: Vec<String>,
    },
    /// Filter value type doesn't match the field type
    TypeMismatch {
        field_type: String,
        filter_type: String,
    },
}

impl fmt::Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueryError::UnsupportedOperator {
                field_type,
                operator,
                supported,
            } => {
                write!(
                    f,
                    "Cannot use '{}' operator on {} fields. Supported: {}",
                    operator,
                    field_type,
                    supported.join(", ")
                )
            }
            QueryError::TypeMismatch {
                field_type,
                filter_type,
            } => {
                write!(
                    f,
                    "Type mismatch: {} field cannot be compared with {} value",
                    field_type, filter_type
                )
            }
        }
    }
}

impl std::error::Error for QueryError {}
