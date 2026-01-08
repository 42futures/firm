//! Reference comparison logic for filters

use crate::ReferenceValue;
use super::types::{FilterOperator, FilterValue};

/// Compare a reference value against a filter
pub fn compare_reference(
    reference: &ReferenceValue,
    operator: &FilterOperator,
    filter_value: &FilterValue,
) -> bool {
    // Only equality operators make sense for references
    if !matches!(operator, FilterOperator::Equal | FilterOperator::NotEqual) {
        return false;
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
        _ => false,
    };

    match operator {
        FilterOperator::Equal => matches,
        FilterOperator::NotEqual => !matches,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use crate::{EntityId, FieldId};
    use super::*;

    #[test]
    fn test_entity_reference_equal_with_reference_value() {
        let reference = ReferenceValue::Entity(EntityId::new("person.john_doe"));
        assert!(compare_reference(
            &reference,
            &FilterOperator::Equal,
            &FilterValue::Reference("person.john_doe".to_string()),
        ));
    }

    #[test]
    fn test_entity_reference_equal_with_string_value() {
        let reference = ReferenceValue::Entity(EntityId::new("person.john_doe"));
        assert!(compare_reference(
            &reference,
            &FilterOperator::Equal,
            &FilterValue::String("person.john_doe".to_string()),
        ));
    }

    #[test]
    fn test_entity_reference_not_equal() {
        let reference = ReferenceValue::Entity(EntityId::new("person.john_doe"));
        assert!(!compare_reference(
            &reference,
            &FilterOperator::Equal,
            &FilterValue::Reference("person.jane_smith".to_string()),
        ));
    }

    #[test]
    fn test_entity_reference_case_insensitive() {
        let reference = ReferenceValue::Entity(EntityId::new("person.john_doe"));
        assert!(compare_reference(
            &reference,
            &FilterOperator::Equal,
            &FilterValue::Reference("PERSON.JOHN_DOE".to_string()),
        ));
    }

    #[test]
    fn test_field_reference_equal() {
        let reference = ReferenceValue::Field(
            EntityId::new("person.john_doe"),
            FieldId::new("name"),
        );
        assert!(compare_reference(
            &reference,
            &FilterOperator::Equal,
            &FilterValue::Reference("person.john_doe.name".to_string()),
        ));
    }

    #[test]
    fn test_field_reference_equal_with_string() {
        let reference = ReferenceValue::Field(
            EntityId::new("person.john_doe"),
            FieldId::new("email"),
        );
        assert!(compare_reference(
            &reference,
            &FilterOperator::Equal,
            &FilterValue::String("person.john_doe.email".to_string()),
        ));
    }

    #[test]
    fn test_field_reference_not_equal() {
        let reference = ReferenceValue::Field(
            EntityId::new("person.john_doe"),
            FieldId::new("name"),
        );
        assert!(!compare_reference(
            &reference,
            &FilterOperator::Equal,
            &FilterValue::Reference("person.john_doe.email".to_string()),
        ));
    }

    #[test]
    fn test_field_reference_case_insensitive() {
        let reference = ReferenceValue::Field(
            EntityId::new("person.john_doe"),
            FieldId::new("name"),
        );
        assert!(compare_reference(
            &reference,
            &FilterOperator::Equal,
            &FilterValue::Reference("PERSON.JOHN_DOE.NAME".to_string()),
        ));
    }

    #[test]
    fn test_entity_vs_field_reference_not_equal() {
        let reference = ReferenceValue::Entity(EntityId::new("person.john_doe"));
        // Entity reference should not match field reference
        assert!(!compare_reference(
            &reference,
            &FilterOperator::Equal,
            &FilterValue::Reference("person.john_doe.name".to_string()),
        ));
    }

    #[test]
    fn test_not_equal_operator() {
        let reference = ReferenceValue::Entity(EntityId::new("person.john_doe"));
        assert!(compare_reference(
            &reference,
            &FilterOperator::NotEqual,
            &FilterValue::Reference("person.jane_smith".to_string()),
        ));
    }

    #[test]
    fn test_not_equal_operator_with_same_reference() {
        let reference = ReferenceValue::Entity(EntityId::new("person.john_doe"));
        assert!(!compare_reference(
            &reference,
            &FilterOperator::NotEqual,
            &FilterValue::Reference("person.john_doe".to_string()),
        ));
    }

    #[test]
    fn test_unsupported_operator_greater_than() {
        let reference = ReferenceValue::Entity(EntityId::new("person.john_doe"));
        assert!(!compare_reference(
            &reference,
            &FilterOperator::GreaterThan,
            &FilterValue::Reference("person.john_doe".to_string()),
        ));
    }

    #[test]
    fn test_unsupported_operator_less_than() {
        let reference = ReferenceValue::Entity(EntityId::new("person.john_doe"));
        assert!(!compare_reference(
            &reference,
            &FilterOperator::LessThan,
            &FilterValue::Reference("person.john_doe".to_string()),
        ));
    }

    #[test]
    fn test_unsupported_operator_contains() {
        let reference = ReferenceValue::Entity(EntityId::new("person.john_doe"));
        assert!(!compare_reference(
            &reference,
            &FilterOperator::Contains,
            &FilterValue::Reference("john".to_string()),
        ));
    }

    #[test]
    fn test_wrong_filter_type_integer() {
        let reference = ReferenceValue::Entity(EntityId::new("person.john_doe"));
        assert!(!compare_reference(
            &reference,
            &FilterOperator::Equal,
            &FilterValue::Integer(42),
        ));
    }

    #[test]
    fn test_wrong_filter_type_boolean() {
        let reference = ReferenceValue::Entity(EntityId::new("person.john_doe"));
        assert!(!compare_reference(
            &reference,
            &FilterOperator::Equal,
            &FilterValue::Boolean(true),
        ));
    }
}
