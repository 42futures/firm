//! Reference comparison logic for filters

use super::super::QueryError;
use super::types::{FilterOperator, FilterValue};
use crate::FieldValue;

/// Compare a reference field value against a filter
pub fn compare_reference(
    field_value: &FieldValue,
    operator: &FilterOperator,
    filter_value: &FilterValue,
) -> Result<bool, QueryError> {
    let reference = match field_value {
        FieldValue::Reference(r) => r,
        _ => {
            return Err(QueryError::TypeMismatch {
                field_type: field_value.get_type().to_string(),
                filter_type: filter_value.type_name().to_string(),
            })
        }
    };

    // Only equality operators make sense for references
    if !matches!(operator, FilterOperator::Equal | FilterOperator::NotEqual) {
        return Err(QueryError::UnsupportedOperator {
            field_type: field_value.get_type().to_string(),
            operator: format!("{:?}", operator),
            supported: vec!["==".to_string(), "!=".to_string()],
        });
    }

    // Compare reference by converting to string representation
    // Entity references: "person.john_doe"
    // Field references: "person.john_doe.name"
    let ref_str = reference.to_string();

    // The filter value should be a Reference or String variant
    let matches = match filter_value {
        FilterValue::Reference(filter_ref_str) => {
            // Case-insensitive comparison of reference strings
            ref_str.eq_ignore_ascii_case(filter_ref_str)
        }
        FilterValue::String(filter_str) => {
            // Also allow comparing against plain strings for convenience
            ref_str.eq_ignore_ascii_case(filter_str)
        }
        _ => {
            return Err(QueryError::TypeMismatch {
                field_type: field_value.get_type().to_string(),
                filter_type: filter_value.type_name().to_string(),
            })
        }
    };

    match operator {
        FilterOperator::Equal => Ok(matches),
        FilterOperator::NotEqual => Ok(!matches),
        _ => unreachable!(), // Already checked above
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EntityId, FieldId, ReferenceValue};

    fn make_entity_ref(id: &str) -> FieldValue {
        FieldValue::Reference(ReferenceValue::Entity(EntityId::new(id)))
    }

    fn make_field_ref(entity_id: &str, field_id: &str) -> FieldValue {
        FieldValue::Reference(ReferenceValue::Field(
            EntityId::new(entity_id),
            FieldId::new(field_id),
        ))
    }

    #[test]
    fn test_entity_reference_equal_with_reference_value() {
        let field = make_entity_ref("person.john_doe");
        assert!(compare_reference(
            &field,
            &FilterOperator::Equal,
            &FilterValue::Reference("person.john_doe".to_string()),
        )
        .unwrap());
    }

    #[test]
    fn test_entity_reference_equal_with_string_value() {
        let field = make_entity_ref("person.john_doe");
        assert!(compare_reference(
            &field,
            &FilterOperator::Equal,
            &FilterValue::String("person.john_doe".to_string()),
        )
        .unwrap());
    }

    #[test]
    fn test_entity_reference_not_equal() {
        let field = make_entity_ref("person.john_doe");
        assert!(!compare_reference(
            &field,
            &FilterOperator::Equal,
            &FilterValue::Reference("person.jane_smith".to_string()),
        )
        .unwrap());
    }

    #[test]
    fn test_entity_reference_case_insensitive() {
        let field = make_entity_ref("person.john_doe");
        assert!(compare_reference(
            &field,
            &FilterOperator::Equal,
            &FilterValue::Reference("PERSON.JOHN_DOE".to_string()),
        )
        .unwrap());
    }

    #[test]
    fn test_field_reference_equal() {
        let field = make_field_ref("person.john_doe", "name");
        assert!(compare_reference(
            &field,
            &FilterOperator::Equal,
            &FilterValue::Reference("person.john_doe.name".to_string()),
        )
        .unwrap());
    }

    #[test]
    fn test_field_reference_equal_with_string() {
        let field = make_field_ref("person.john_doe", "email");
        assert!(compare_reference(
            &field,
            &FilterOperator::Equal,
            &FilterValue::String("person.john_doe.email".to_string()),
        )
        .unwrap());
    }

    #[test]
    fn test_field_reference_not_equal() {
        let field = make_field_ref("person.john_doe", "name");
        assert!(!compare_reference(
            &field,
            &FilterOperator::Equal,
            &FilterValue::Reference("person.john_doe.email".to_string()),
        )
        .unwrap());
    }

    #[test]
    fn test_field_reference_case_insensitive() {
        let field = make_field_ref("person.john_doe", "name");
        assert!(compare_reference(
            &field,
            &FilterOperator::Equal,
            &FilterValue::Reference("PERSON.JOHN_DOE.NAME".to_string()),
        )
        .unwrap());
    }

    #[test]
    fn test_entity_vs_field_reference_not_equal() {
        let field = make_entity_ref("person.john_doe");
        // Entity reference should not match field reference
        assert!(!compare_reference(
            &field,
            &FilterOperator::Equal,
            &FilterValue::Reference("person.john_doe.name".to_string()),
        )
        .unwrap());
    }

    #[test]
    fn test_not_equal_operator() {
        let field = make_entity_ref("person.john_doe");
        assert!(compare_reference(
            &field,
            &FilterOperator::NotEqual,
            &FilterValue::Reference("person.jane_smith".to_string()),
        )
        .unwrap());
    }

    #[test]
    fn test_not_equal_operator_with_same_reference() {
        let field = make_entity_ref("person.john_doe");
        assert!(!compare_reference(
            &field,
            &FilterOperator::NotEqual,
            &FilterValue::Reference("person.john_doe".to_string()),
        )
        .unwrap());
    }

    #[test]
    fn test_unsupported_operator_greater_than() {
        let field = make_entity_ref("person.john_doe");
        let result = compare_reference(
            &field,
            &FilterOperator::GreaterThan,
            &FilterValue::Reference("person.john_doe".to_string()),
        );
        assert!(matches!(result, Err(QueryError::UnsupportedOperator { .. })));
    }

    #[test]
    fn test_unsupported_operator_less_than() {
        let field = make_entity_ref("person.john_doe");
        let result = compare_reference(
            &field,
            &FilterOperator::LessThan,
            &FilterValue::Reference("person.john_doe".to_string()),
        );
        assert!(matches!(result, Err(QueryError::UnsupportedOperator { .. })));
    }

    #[test]
    fn test_unsupported_operator_contains() {
        let field = make_entity_ref("person.john_doe");
        let result = compare_reference(
            &field,
            &FilterOperator::Contains,
            &FilterValue::Reference("john".to_string()),
        );
        assert!(matches!(result, Err(QueryError::UnsupportedOperator { .. })));
    }

    #[test]
    fn test_wrong_filter_type_integer() {
        let field = make_entity_ref("person.john_doe");
        let result = compare_reference(&field, &FilterOperator::Equal, &FilterValue::Integer(42));
        assert!(matches!(result, Err(QueryError::TypeMismatch { .. })));
    }

    #[test]
    fn test_wrong_filter_type_boolean() {
        let field = make_entity_ref("person.john_doe");
        let result =
            compare_reference(&field, &FilterOperator::Equal, &FilterValue::Boolean(true));
        assert!(matches!(result, Err(QueryError::TypeMismatch { .. })));
    }
}
