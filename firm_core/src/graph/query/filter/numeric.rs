//! Numeric comparison logic for filters (integer and float)

use super::types::{FilterOperator, FilterValue};

/// Compare an integer value against a filter
pub fn compare_integer(value: i64, operator: &FilterOperator, filter_value: &FilterValue) -> bool {
    match filter_value {
        FilterValue::Integer(filter_int) => match operator {
            FilterOperator::Equal => value == *filter_int,
            FilterOperator::NotEqual => value != *filter_int,
            FilterOperator::GreaterThan => value > *filter_int,
            FilterOperator::LessThan => value < *filter_int,
            FilterOperator::GreaterOrEqual => value >= *filter_int,
            FilterOperator::LessOrEqual => value <= *filter_int,
            _ => false,
        },
        FilterValue::Float(filter_float) => match operator {
            FilterOperator::Equal => value as f64 == *filter_float,
            FilterOperator::NotEqual => value as f64 != *filter_float,
            FilterOperator::GreaterThan => (value as f64) > *filter_float,
            FilterOperator::LessThan => (value as f64) < *filter_float,
            FilterOperator::GreaterOrEqual => (value as f64) >= *filter_float,
            FilterOperator::LessOrEqual => (value as f64) <= *filter_float,
            _ => false,
        },
        _ => false,
    }
}

/// Compare a float value against a filter
pub fn compare_float(value: f64, operator: &FilterOperator, filter_value: &FilterValue) -> bool {
    match filter_value {
        FilterValue::Float(filter_float) => match operator {
            FilterOperator::Equal => (value - filter_float).abs() < f64::EPSILON,
            FilterOperator::NotEqual => (value - filter_float).abs() >= f64::EPSILON,
            FilterOperator::GreaterThan => value > *filter_float,
            FilterOperator::LessThan => value < *filter_float,
            FilterOperator::GreaterOrEqual => value >= *filter_float,
            FilterOperator::LessOrEqual => value <= *filter_float,
            _ => false,
        },
        FilterValue::Integer(filter_int) => match operator {
            FilterOperator::Equal => (value - *filter_int as f64).abs() < f64::EPSILON,
            FilterOperator::NotEqual => (value - *filter_int as f64).abs() >= f64::EPSILON,
            FilterOperator::GreaterThan => value > *filter_int as f64,
            FilterOperator::LessThan => value < *filter_int as f64,
            FilterOperator::GreaterOrEqual => value >= *filter_int as f64,
            FilterOperator::LessOrEqual => value <= *filter_int as f64,
            _ => false,
        },
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Integer Tests =====

    #[test]
    fn test_integer_equal_integer() {
        assert!(compare_integer(
            42,
            &FilterOperator::Equal,
            &FilterValue::Integer(42)
        ));
    }

    #[test]
    fn test_integer_not_equal_integer() {
        assert!(!compare_integer(
            42,
            &FilterOperator::Equal,
            &FilterValue::Integer(100)
        ));
    }

    #[test]
    fn test_integer_greater_than() {
        assert!(compare_integer(
            100,
            &FilterOperator::GreaterThan,
            &FilterValue::Integer(50)
        ));
    }

    #[test]
    fn test_integer_less_than() {
        assert!(compare_integer(
            50,
            &FilterOperator::LessThan,
            &FilterValue::Integer(100)
        ));
    }

    #[test]
    fn test_integer_greater_or_equal() {
        assert!(compare_integer(
            100,
            &FilterOperator::GreaterOrEqual,
            &FilterValue::Integer(100)
        ));
        assert!(compare_integer(
            100,
            &FilterOperator::GreaterOrEqual,
            &FilterValue::Integer(50)
        ));
    }

    #[test]
    fn test_integer_less_or_equal() {
        assert!(compare_integer(
            100,
            &FilterOperator::LessOrEqual,
            &FilterValue::Integer(100)
        ));
        assert!(compare_integer(
            50,
            &FilterOperator::LessOrEqual,
            &FilterValue::Integer(100)
        ));
    }

    #[test]
    fn test_integer_not_equal_operator() {
        assert!(compare_integer(
            42,
            &FilterOperator::NotEqual,
            &FilterValue::Integer(100)
        ));
        assert!(!compare_integer(
            42,
            &FilterOperator::NotEqual,
            &FilterValue::Integer(42)
        ));
    }

    // ===== Integer vs Float Tests =====

    #[test]
    fn test_integer_equal_float() {
        assert!(compare_integer(
            42,
            &FilterOperator::Equal,
            &FilterValue::Float(42.0)
        ));
    }

    #[test]
    fn test_integer_not_equal_float_with_decimal() {
        assert!(!compare_integer(
            42,
            &FilterOperator::Equal,
            &FilterValue::Float(42.5)
        ));
    }

    #[test]
    fn test_integer_greater_than_float() {
        assert!(compare_integer(
            100,
            &FilterOperator::GreaterThan,
            &FilterValue::Float(50.5)
        ));
    }

    #[test]
    fn test_integer_less_than_float() {
        assert!(compare_integer(
            50,
            &FilterOperator::LessThan,
            &FilterValue::Float(100.5)
        ));
    }

    #[test]
    fn test_integer_greater_or_equal_float() {
        assert!(compare_integer(
            100,
            &FilterOperator::GreaterOrEqual,
            &FilterValue::Float(100.0)
        ));
        assert!(compare_integer(
            100,
            &FilterOperator::GreaterOrEqual,
            &FilterValue::Float(50.5)
        ));
    }

    #[test]
    fn test_integer_less_or_equal_float() {
        assert!(compare_integer(
            100,
            &FilterOperator::LessOrEqual,
            &FilterValue::Float(100.0)
        ));
        assert!(compare_integer(
            50,
            &FilterOperator::LessOrEqual,
            &FilterValue::Float(100.5)
        ));
    }

    // ===== Float Tests =====

    #[test]
    fn test_float_equal_float() {
        assert!(compare_float(
            42.5,
            &FilterOperator::Equal,
            &FilterValue::Float(42.5)
        ));
    }

    #[test]
    fn test_float_not_equal_float() {
        assert!(!compare_float(
            42.5,
            &FilterOperator::Equal,
            &FilterValue::Float(100.5)
        ));
    }

    #[test]
    fn test_float_greater_than() {
        assert!(compare_float(
            100.5,
            &FilterOperator::GreaterThan,
            &FilterValue::Float(50.5)
        ));
    }

    #[test]
    fn test_float_less_than() {
        assert!(compare_float(
            50.5,
            &FilterOperator::LessThan,
            &FilterValue::Float(100.5)
        ));
    }

    #[test]
    fn test_float_greater_or_equal() {
        assert!(compare_float(
            100.5,
            &FilterOperator::GreaterOrEqual,
            &FilterValue::Float(100.5)
        ));
        assert!(compare_float(
            100.5,
            &FilterOperator::GreaterOrEqual,
            &FilterValue::Float(50.5)
        ));
    }

    #[test]
    fn test_float_less_or_equal() {
        assert!(compare_float(
            100.5,
            &FilterOperator::LessOrEqual,
            &FilterValue::Float(100.5)
        ));
        assert!(compare_float(
            50.5,
            &FilterOperator::LessOrEqual,
            &FilterValue::Float(100.5)
        ));
    }

    #[test]
    fn test_float_not_equal_operator() {
        assert!(compare_float(
            42.5,
            &FilterOperator::NotEqual,
            &FilterValue::Float(100.5)
        ));
        assert!(!compare_float(
            42.5,
            &FilterOperator::NotEqual,
            &FilterValue::Float(42.5)
        ));
    }

    // ===== Float vs Integer Tests =====

    #[test]
    fn test_float_equal_integer() {
        assert!(compare_float(
            42.0,
            &FilterOperator::Equal,
            &FilterValue::Integer(42)
        ));
    }

    #[test]
    fn test_float_not_equal_integer_with_decimal() {
        assert!(!compare_float(
            42.5,
            &FilterOperator::Equal,
            &FilterValue::Integer(42)
        ));
    }

    #[test]
    fn test_float_greater_than_integer() {
        assert!(compare_float(
            100.5,
            &FilterOperator::GreaterThan,
            &FilterValue::Integer(50)
        ));
    }

    #[test]
    fn test_float_less_than_integer() {
        assert!(compare_float(
            50.5,
            &FilterOperator::LessThan,
            &FilterValue::Integer(100)
        ));
    }

    #[test]
    fn test_float_greater_or_equal_integer() {
        assert!(compare_float(
            100.0,
            &FilterOperator::GreaterOrEqual,
            &FilterValue::Integer(100)
        ));
        assert!(compare_float(
            100.5,
            &FilterOperator::GreaterOrEqual,
            &FilterValue::Integer(50)
        ));
    }

    #[test]
    fn test_float_less_or_equal_integer() {
        assert!(compare_float(
            100.0,
            &FilterOperator::LessOrEqual,
            &FilterValue::Integer(100)
        ));
        assert!(compare_float(
            50.5,
            &FilterOperator::LessOrEqual,
            &FilterValue::Integer(100)
        ));
    }

    // ===== Edge Cases =====

    #[test]
    fn test_float_precision() {
        // Test that very small differences are considered equal due to EPSILON
        let value = 1.0 + f64::EPSILON / 2.0;
        assert!(compare_float(
            value,
            &FilterOperator::Equal,
            &FilterValue::Float(1.0)
        ));
    }

    #[test]
    fn test_integer_unsupported_operator() {
        assert!(!compare_integer(
            42,
            &FilterOperator::Contains,
            &FilterValue::Integer(42)
        ));
    }

    #[test]
    fn test_float_unsupported_operator() {
        assert!(!compare_float(
            42.5,
            &FilterOperator::Contains,
            &FilterValue::Float(42.5)
        ));
    }

    #[test]
    fn test_integer_wrong_filter_type() {
        assert!(!compare_integer(
            42,
            &FilterOperator::Equal,
            &FilterValue::String("42".to_string())
        ));
    }

    #[test]
    fn test_float_wrong_filter_type() {
        assert!(!compare_float(
            42.5,
            &FilterOperator::Equal,
            &FilterValue::String("42.5".to_string())
        ));
    }

    #[test]
    fn test_large_integers() {
        assert!(compare_integer(
            i64::MAX,
            &FilterOperator::Equal,
            &FilterValue::Integer(i64::MAX)
        ));
        assert!(compare_integer(
            i64::MIN,
            &FilterOperator::Equal,
            &FilterValue::Integer(i64::MIN)
        ));
    }
}
