//! Boolean comparison logic for filters

use super::types::{FilterOperator, FilterValue};

/// Compare a boolean value against a filter
pub fn compare_boolean(value: bool, operator: &FilterOperator, filter_value: &FilterValue) -> bool {
    match filter_value {
        FilterValue::Boolean(filter_bool) => match operator {
            FilterOperator::Equal => value == *filter_bool,
            FilterOperator::NotEqual => value != *filter_bool,
            _ => false,
        },
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_true_equals_true() {
        assert!(compare_boolean(true, &FilterOperator::Equal, &FilterValue::Boolean(true)));
    }

    #[test]
    fn test_false_equals_false() {
        assert!(compare_boolean(false, &FilterOperator::Equal, &FilterValue::Boolean(false)));
    }

    #[test]
    fn test_true_not_equals_false() {
        assert!(!compare_boolean(true, &FilterOperator::Equal, &FilterValue::Boolean(false)));
    }

    #[test]
    fn test_false_not_equals_true() {
        assert!(!compare_boolean(false, &FilterOperator::Equal, &FilterValue::Boolean(true)));
    }

    #[test]
    fn test_true_not_equal_false() {
        assert!(compare_boolean(true, &FilterOperator::NotEqual, &FilterValue::Boolean(false)));
    }

    #[test]
    fn test_false_not_equal_true() {
        assert!(compare_boolean(false, &FilterOperator::NotEqual, &FilterValue::Boolean(true)));
    }

    #[test]
    fn test_true_not_not_equal_true() {
        assert!(!compare_boolean(true, &FilterOperator::NotEqual, &FilterValue::Boolean(true)));
    }

    #[test]
    fn test_false_not_not_equal_false() {
        assert!(!compare_boolean(false, &FilterOperator::NotEqual, &FilterValue::Boolean(false)));
    }

    #[test]
    fn test_unsupported_operator_greater_than() {
        assert!(!compare_boolean(true, &FilterOperator::GreaterThan, &FilterValue::Boolean(false)));
    }

    #[test]
    fn test_unsupported_operator_less_than() {
        assert!(!compare_boolean(false, &FilterOperator::LessThan, &FilterValue::Boolean(true)));
    }

    #[test]
    fn test_unsupported_operator_contains() {
        assert!(!compare_boolean(true, &FilterOperator::Contains, &FilterValue::Boolean(true)));
    }

    #[test]
    fn test_wrong_filter_type_string() {
        assert!(!compare_boolean(true, &FilterOperator::Equal, &FilterValue::String("true".to_string())));
    }

    #[test]
    fn test_wrong_filter_type_integer() {
        assert!(!compare_boolean(true, &FilterOperator::Equal, &FilterValue::Integer(1)));
    }

    #[test]
    fn test_wrong_filter_type_float() {
        assert!(!compare_boolean(false, &FilterOperator::Equal, &FilterValue::Float(0.0)));
    }
}
