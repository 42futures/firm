//! Boolean comparison logic for filters

use super::super::QueryError;
use super::types::{FilterOperator, FilterValue};
use crate::FieldValue;

/// Compare a boolean field value against a filter
pub fn compare_boolean(
    field_value: &FieldValue,
    operator: &FilterOperator,
    filter_value: &FilterValue,
) -> Result<bool, QueryError> {
    let value = match field_value {
        FieldValue::Boolean(b) => *b,
        _ => {
            return Err(QueryError::TypeMismatch {
                field_type: field_value.get_type().to_string(),
                filter_type: filter_value.type_name().to_string(),
            });
        }
    };

    match filter_value {
        FilterValue::Boolean(filter_bool) => match operator {
            FilterOperator::Equal => Ok(value == *filter_bool),
            FilterOperator::NotEqual => Ok(value != *filter_bool),
            _ => Err(QueryError::UnsupportedOperator {
                field_type: field_value.get_type().to_string(),
                operator: format!("{:?}", operator),
                supported: vec!["==".to_string(), "!=".to_string()],
            }),
        },
        _ => Err(QueryError::TypeMismatch {
            field_type: field_value.get_type().to_string(),
            filter_type: filter_value.type_name().to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_true_equals_true() {
        let field = FieldValue::Boolean(true);
        assert!(
            compare_boolean(&field, &FilterOperator::Equal, &FilterValue::Boolean(true)).unwrap()
        );
    }

    #[test]
    fn test_false_equals_false() {
        let field = FieldValue::Boolean(false);
        assert!(
            compare_boolean(&field, &FilterOperator::Equal, &FilterValue::Boolean(false)).unwrap()
        );
    }

    #[test]
    fn test_true_not_equals_false() {
        let field = FieldValue::Boolean(true);
        assert!(
            !compare_boolean(&field, &FilterOperator::Equal, &FilterValue::Boolean(false)).unwrap()
        );
    }

    #[test]
    fn test_false_not_equals_true() {
        let field = FieldValue::Boolean(false);
        assert!(
            !compare_boolean(&field, &FilterOperator::Equal, &FilterValue::Boolean(true)).unwrap()
        );
    }

    #[test]
    fn test_true_not_equal_false() {
        let field = FieldValue::Boolean(true);
        assert!(
            compare_boolean(
                &field,
                &FilterOperator::NotEqual,
                &FilterValue::Boolean(false)
            )
            .unwrap()
        );
    }

    #[test]
    fn test_false_not_equal_true() {
        let field = FieldValue::Boolean(false);
        assert!(
            compare_boolean(
                &field,
                &FilterOperator::NotEqual,
                &FilterValue::Boolean(true)
            )
            .unwrap()
        );
    }

    #[test]
    fn test_true_not_not_equal_true() {
        let field = FieldValue::Boolean(true);
        assert!(
            !compare_boolean(
                &field,
                &FilterOperator::NotEqual,
                &FilterValue::Boolean(true)
            )
            .unwrap()
        );
    }

    #[test]
    fn test_false_not_not_equal_false() {
        let field = FieldValue::Boolean(false);
        assert!(
            !compare_boolean(
                &field,
                &FilterOperator::NotEqual,
                &FilterValue::Boolean(false)
            )
            .unwrap()
        );
    }

    #[test]
    fn test_unsupported_operator_greater_than() {
        let field = FieldValue::Boolean(true);
        let result = compare_boolean(
            &field,
            &FilterOperator::GreaterThan,
            &FilterValue::Boolean(false),
        );
        assert!(matches!(
            result,
            Err(QueryError::UnsupportedOperator { .. })
        ));
    }

    #[test]
    fn test_unsupported_operator_less_than() {
        let field = FieldValue::Boolean(false);
        let result = compare_boolean(
            &field,
            &FilterOperator::LessThan,
            &FilterValue::Boolean(true),
        );
        assert!(matches!(
            result,
            Err(QueryError::UnsupportedOperator { .. })
        ));
    }

    #[test]
    fn test_unsupported_operator_contains() {
        let field = FieldValue::Boolean(true);
        let result = compare_boolean(
            &field,
            &FilterOperator::Contains,
            &FilterValue::Boolean(true),
        );
        assert!(matches!(
            result,
            Err(QueryError::UnsupportedOperator { .. })
        ));
    }

    #[test]
    fn test_wrong_filter_type_string() {
        let field = FieldValue::Boolean(true);
        let result = compare_boolean(
            &field,
            &FilterOperator::Equal,
            &FilterValue::String("true".to_string()),
        );
        assert!(matches!(result, Err(QueryError::TypeMismatch { .. })));
    }

    #[test]
    fn test_wrong_filter_type_integer() {
        let field = FieldValue::Boolean(true);
        let result = compare_boolean(&field, &FilterOperator::Equal, &FilterValue::Integer(1));
        assert!(matches!(result, Err(QueryError::TypeMismatch { .. })));
    }

    #[test]
    fn test_wrong_filter_type_float() {
        let field = FieldValue::Boolean(false);
        let result = compare_boolean(&field, &FilterOperator::Equal, &FilterValue::Float(0.0));
        assert!(matches!(result, Err(QueryError::TypeMismatch { .. })));
    }
}
