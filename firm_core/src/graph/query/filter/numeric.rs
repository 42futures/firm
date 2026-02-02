//! Numeric comparison logic for filters (integer and float)

use super::super::QueryError;
use super::types::{FilterOperator, FilterValue};
use crate::FieldValue;

const SUPPORTED_OPS: [&str; 6] = ["==", "!=", ">", "<", ">=", "<="];

/// Compare an integer field value against a filter
pub fn compare_integer(
    field_value: &FieldValue,
    operator: &FilterOperator,
    filter_value: &FilterValue,
) -> Result<bool, QueryError> {
    let value = match field_value {
        FieldValue::Integer(i) => *i,
        _ => {
            return Err(QueryError::TypeMismatch {
                field_type: field_value.get_type().to_string(),
                filter_type: filter_value.type_name().to_string(),
            })
        }
    };

    match filter_value {
        FilterValue::Integer(filter_int) => match operator {
            FilterOperator::Equal => Ok(value == *filter_int),
            FilterOperator::NotEqual => Ok(value != *filter_int),
            FilterOperator::GreaterThan => Ok(value > *filter_int),
            FilterOperator::LessThan => Ok(value < *filter_int),
            FilterOperator::GreaterOrEqual => Ok(value >= *filter_int),
            FilterOperator::LessOrEqual => Ok(value <= *filter_int),
            _ => Err(unsupported_op_error(field_value, operator)),
        },
        FilterValue::Float(filter_float) => match operator {
            FilterOperator::Equal => Ok(value as f64 == *filter_float),
            FilterOperator::NotEqual => Ok(value as f64 != *filter_float),
            FilterOperator::GreaterThan => Ok((value as f64) > *filter_float),
            FilterOperator::LessThan => Ok((value as f64) < *filter_float),
            FilterOperator::GreaterOrEqual => Ok((value as f64) >= *filter_float),
            FilterOperator::LessOrEqual => Ok((value as f64) <= *filter_float),
            _ => Err(unsupported_op_error(field_value, operator)),
        },
        _ => Err(QueryError::TypeMismatch {
            field_type: field_value.get_type().to_string(),
            filter_type: filter_value.type_name().to_string(),
        }),
    }
}

/// Compare a float field value against a filter
pub fn compare_float(
    field_value: &FieldValue,
    operator: &FilterOperator,
    filter_value: &FilterValue,
) -> Result<bool, QueryError> {
    let value = match field_value {
        FieldValue::Float(f) => *f,
        _ => {
            return Err(QueryError::TypeMismatch {
                field_type: field_value.get_type().to_string(),
                filter_type: filter_value.type_name().to_string(),
            })
        }
    };

    match filter_value {
        FilterValue::Float(filter_float) => match operator {
            FilterOperator::Equal => Ok((value - filter_float).abs() < f64::EPSILON),
            FilterOperator::NotEqual => Ok((value - filter_float).abs() >= f64::EPSILON),
            FilterOperator::GreaterThan => Ok(value > *filter_float),
            FilterOperator::LessThan => Ok(value < *filter_float),
            FilterOperator::GreaterOrEqual => Ok(value >= *filter_float),
            FilterOperator::LessOrEqual => Ok(value <= *filter_float),
            _ => Err(unsupported_op_error(field_value, operator)),
        },
        FilterValue::Integer(filter_int) => match operator {
            FilterOperator::Equal => Ok((value - *filter_int as f64).abs() < f64::EPSILON),
            FilterOperator::NotEqual => Ok((value - *filter_int as f64).abs() >= f64::EPSILON),
            FilterOperator::GreaterThan => Ok(value > *filter_int as f64),
            FilterOperator::LessThan => Ok(value < *filter_int as f64),
            FilterOperator::GreaterOrEqual => Ok(value >= *filter_int as f64),
            FilterOperator::LessOrEqual => Ok(value <= *filter_int as f64),
            _ => Err(unsupported_op_error(field_value, operator)),
        },
        _ => Err(QueryError::TypeMismatch {
            field_type: field_value.get_type().to_string(),
            filter_type: filter_value.type_name().to_string(),
        }),
    }
}

fn unsupported_op_error(field_value: &FieldValue, operator: &FilterOperator) -> QueryError {
    QueryError::UnsupportedOperator {
        field_type: field_value.get_type().to_string(),
        operator: format!("{:?}", operator),
        supported: SUPPORTED_OPS.iter().map(|s| s.to_string()).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn int_field(i: i64) -> FieldValue {
        FieldValue::Integer(i)
    }

    fn float_field(f: f64) -> FieldValue {
        FieldValue::Float(f)
    }

    // ===== Integer Tests =====

    #[test]
    fn test_integer_equal_integer() {
        assert!(compare_integer(&int_field(42), &FilterOperator::Equal, &FilterValue::Integer(42)).unwrap());
    }

    #[test]
    fn test_integer_not_equal_integer() {
        assert!(!compare_integer(&int_field(42), &FilterOperator::Equal, &FilterValue::Integer(100)).unwrap());
    }

    #[test]
    fn test_integer_greater_than() {
        assert!(compare_integer(&int_field(100), &FilterOperator::GreaterThan, &FilterValue::Integer(50)).unwrap());
    }

    #[test]
    fn test_integer_less_than() {
        assert!(compare_integer(&int_field(50), &FilterOperator::LessThan, &FilterValue::Integer(100)).unwrap());
    }

    #[test]
    fn test_integer_greater_or_equal() {
        assert!(compare_integer(&int_field(100), &FilterOperator::GreaterOrEqual, &FilterValue::Integer(100)).unwrap());
        assert!(compare_integer(&int_field(100), &FilterOperator::GreaterOrEqual, &FilterValue::Integer(50)).unwrap());
    }

    #[test]
    fn test_integer_less_or_equal() {
        assert!(compare_integer(&int_field(100), &FilterOperator::LessOrEqual, &FilterValue::Integer(100)).unwrap());
        assert!(compare_integer(&int_field(50), &FilterOperator::LessOrEqual, &FilterValue::Integer(100)).unwrap());
    }

    #[test]
    fn test_integer_not_equal_operator() {
        assert!(compare_integer(&int_field(42), &FilterOperator::NotEqual, &FilterValue::Integer(100)).unwrap());
        assert!(!compare_integer(&int_field(42), &FilterOperator::NotEqual, &FilterValue::Integer(42)).unwrap());
    }

    // ===== Integer vs Float Tests =====

    #[test]
    fn test_integer_equal_float() {
        assert!(compare_integer(&int_field(42), &FilterOperator::Equal, &FilterValue::Float(42.0)).unwrap());
    }

    #[test]
    fn test_integer_not_equal_float_with_decimal() {
        assert!(!compare_integer(&int_field(42), &FilterOperator::Equal, &FilterValue::Float(42.5)).unwrap());
    }

    #[test]
    fn test_integer_greater_than_float() {
        assert!(compare_integer(&int_field(100), &FilterOperator::GreaterThan, &FilterValue::Float(50.5)).unwrap());
    }

    #[test]
    fn test_integer_less_than_float() {
        assert!(compare_integer(&int_field(50), &FilterOperator::LessThan, &FilterValue::Float(100.5)).unwrap());
    }

    #[test]
    fn test_integer_greater_or_equal_float() {
        assert!(compare_integer(&int_field(100), &FilterOperator::GreaterOrEqual, &FilterValue::Float(100.0)).unwrap());
        assert!(compare_integer(&int_field(100), &FilterOperator::GreaterOrEqual, &FilterValue::Float(50.5)).unwrap());
    }

    #[test]
    fn test_integer_less_or_equal_float() {
        assert!(compare_integer(&int_field(100), &FilterOperator::LessOrEqual, &FilterValue::Float(100.0)).unwrap());
        assert!(compare_integer(&int_field(50), &FilterOperator::LessOrEqual, &FilterValue::Float(100.5)).unwrap());
    }

    // ===== Float Tests =====

    #[test]
    fn test_float_equal_float() {
        assert!(compare_float(&float_field(42.5), &FilterOperator::Equal, &FilterValue::Float(42.5)).unwrap());
    }

    #[test]
    fn test_float_not_equal_float() {
        assert!(!compare_float(&float_field(42.5), &FilterOperator::Equal, &FilterValue::Float(100.5)).unwrap());
    }

    #[test]
    fn test_float_greater_than() {
        assert!(compare_float(&float_field(100.5), &FilterOperator::GreaterThan, &FilterValue::Float(50.5)).unwrap());
    }

    #[test]
    fn test_float_less_than() {
        assert!(compare_float(&float_field(50.5), &FilterOperator::LessThan, &FilterValue::Float(100.5)).unwrap());
    }

    #[test]
    fn test_float_greater_or_equal() {
        assert!(compare_float(&float_field(100.5), &FilterOperator::GreaterOrEqual, &FilterValue::Float(100.5)).unwrap());
        assert!(compare_float(&float_field(100.5), &FilterOperator::GreaterOrEqual, &FilterValue::Float(50.5)).unwrap());
    }

    #[test]
    fn test_float_less_or_equal() {
        assert!(compare_float(&float_field(100.5), &FilterOperator::LessOrEqual, &FilterValue::Float(100.5)).unwrap());
        assert!(compare_float(&float_field(50.5), &FilterOperator::LessOrEqual, &FilterValue::Float(100.5)).unwrap());
    }

    #[test]
    fn test_float_not_equal_operator() {
        assert!(compare_float(&float_field(42.5), &FilterOperator::NotEqual, &FilterValue::Float(100.5)).unwrap());
        assert!(!compare_float(&float_field(42.5), &FilterOperator::NotEqual, &FilterValue::Float(42.5)).unwrap());
    }

    // ===== Float vs Integer Tests =====

    #[test]
    fn test_float_equal_integer() {
        assert!(compare_float(&float_field(42.0), &FilterOperator::Equal, &FilterValue::Integer(42)).unwrap());
    }

    #[test]
    fn test_float_not_equal_integer_with_decimal() {
        assert!(!compare_float(&float_field(42.5), &FilterOperator::Equal, &FilterValue::Integer(42)).unwrap());
    }

    #[test]
    fn test_float_greater_than_integer() {
        assert!(compare_float(&float_field(100.5), &FilterOperator::GreaterThan, &FilterValue::Integer(50)).unwrap());
    }

    #[test]
    fn test_float_less_than_integer() {
        assert!(compare_float(&float_field(50.5), &FilterOperator::LessThan, &FilterValue::Integer(100)).unwrap());
    }

    #[test]
    fn test_float_greater_or_equal_integer() {
        assert!(compare_float(&float_field(100.0), &FilterOperator::GreaterOrEqual, &FilterValue::Integer(100)).unwrap());
        assert!(compare_float(&float_field(100.5), &FilterOperator::GreaterOrEqual, &FilterValue::Integer(50)).unwrap());
    }

    #[test]
    fn test_float_less_or_equal_integer() {
        assert!(compare_float(&float_field(100.0), &FilterOperator::LessOrEqual, &FilterValue::Integer(100)).unwrap());
        assert!(compare_float(&float_field(50.5), &FilterOperator::LessOrEqual, &FilterValue::Integer(100)).unwrap());
    }

    // ===== Edge Cases =====

    #[test]
    fn test_float_precision() {
        let value = 1.0 + f64::EPSILON / 2.0;
        assert!(compare_float(&float_field(value), &FilterOperator::Equal, &FilterValue::Float(1.0)).unwrap());
    }

    #[test]
    fn test_integer_unsupported_operator() {
        let result = compare_integer(&int_field(42), &FilterOperator::Contains, &FilterValue::Integer(42));
        assert!(matches!(result, Err(QueryError::UnsupportedOperator { .. })));
    }

    #[test]
    fn test_float_unsupported_operator() {
        let result = compare_float(&float_field(42.5), &FilterOperator::Contains, &FilterValue::Float(42.5));
        assert!(matches!(result, Err(QueryError::UnsupportedOperator { .. })));
    }

    #[test]
    fn test_integer_wrong_filter_type() {
        let result = compare_integer(&int_field(42), &FilterOperator::Equal, &FilterValue::String("42".to_string()));
        assert!(matches!(result, Err(QueryError::TypeMismatch { .. })));
    }

    #[test]
    fn test_float_wrong_filter_type() {
        let result = compare_float(&float_field(42.5), &FilterOperator::Equal, &FilterValue::String("42.5".to_string()));
        assert!(matches!(result, Err(QueryError::TypeMismatch { .. })));
    }

    #[test]
    fn test_large_integers() {
        assert!(compare_integer(&int_field(i64::MAX), &FilterOperator::Equal, &FilterValue::Integer(i64::MAX)).unwrap());
        assert!(compare_integer(&int_field(i64::MIN), &FilterOperator::Equal, &FilterValue::Integer(i64::MIN)).unwrap());
    }
}
