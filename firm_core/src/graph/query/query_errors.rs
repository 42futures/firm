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
    /// Entity type does not exist in the graph
    UnknownEntityType {
        requested: String,
        available: Vec<String>,
    },
    /// Invalid date/datetime format in filter value
    InvalidDateFormat {
        value: String,
    },
    /// Invalid aggregation operation
    InvalidAggregation {
        message: String,
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
            QueryError::UnknownEntityType {
                requested,
                available,
            } => {
                if available.is_empty() {
                    write!(f, "Entity type '{}' not found. No entity types exist.", requested)
                } else {
                    write!(
                        f,
                        "Entity type '{}' not found. Available types: {}",
                        requested,
                        available.join(", ")
                    )
                }
            }
            QueryError::InvalidDateFormat { value } => {
                write!(
                    f,
                    "Invalid date '{}'. Expected format: YYYY-MM-DD or full ISO 8601 datetime.",
                    value
                )
            }
            QueryError::InvalidAggregation { message } => {
                write!(f, "Invalid aggregation: {}", message)
            }
        }
    }
}

impl std::error::Error for QueryError {}
