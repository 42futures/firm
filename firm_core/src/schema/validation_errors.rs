use crate::{EntityId, EntityType, FieldId, FieldType};

/// Defines the types of errors you might encounter when validating a schema.
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationErrorType {
    /// The entity type did not match the schema.
    MismatchedEntityType {
        expected: EntityType,
        actual: EntityType,
    },
    /// The entity is missing a required field.
    MissingRequiredField { required: FieldId },
    /// The entity has a field whose type did not match the schema.
    MismatchedFieldType {
        expected: FieldType,
        actual: FieldType,
    },
    /// The enum field has a value that is not in the allowed values.
    InvalidEnumValue {
        actual: String,
        allowed: Vec<String>,
    },
}

/// Information about an error encountered while validating a schema.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationError {
    pub entity_id: Option<EntityId>,
    pub field: Option<FieldId>,
    pub message: String,
    pub error_type: ValidationErrorType,
}

impl ValidationError {
    /// Shorthand for creating a mismatched entity type error.
    pub fn mismatched_entity_type(
        entity_id: &EntityId,
        expected: &EntityType,
        actual: &EntityType,
    ) -> Self {
        Self {
            entity_id: Some(entity_id.clone()),
            field: None,
            message: format!(
                "Expected entity '{}' to be of type '{}' but it was '{}'",
                entity_id, expected, actual
            ),
            error_type: ValidationErrorType::MismatchedEntityType {
                expected: expected.clone(),
                actual: actual.clone(),
            },
        }
    }

    /// Shorthand for creating a missing required field error.
    pub fn missing_field(entity_id: &EntityId, field_id: &FieldId) -> Self {
        Self {
            entity_id: Some(entity_id.clone()),
            field: Some(field_id.clone()),
            message: format!(
                "Missing required field '{}' for entity '{}'",
                field_id, entity_id
            ),
            error_type: ValidationErrorType::MissingRequiredField {
                required: field_id.clone(),
            },
        }
    }

    /// Shorthand for creating a mismatched field type error.
    pub fn mismatched_field_type(
        entity_id: &EntityId,
        field_id: &FieldId,
        expected: &FieldType,
        actual: &FieldType,
    ) -> Self {
        Self {
            entity_id: Some(entity_id.clone()),
            field: Some(field_id.clone()),
            message: format!(
                "Expected field '{}' for entity '{}' to be of type '{}' but it was '{}'",
                field_id, entity_id, expected, actual
            ),
            error_type: ValidationErrorType::MismatchedFieldType {
                expected: expected.clone(),
                actual: actual.clone(),
            },
        }
    }

    /// Shorthand for creating an invalid enum value error.
    pub fn invalid_enum_value(
        entity_id: &EntityId,
        field_id: &FieldId,
        actual: &str,
        allowed: &[String],
    ) -> Self {
        Self {
            entity_id: Some(entity_id.clone()),
            field: Some(field_id.clone()),
            message: format!(
                "Invalid value '{}' for enum field '{}' in entity '{}'. Expected one of: [{}]",
                actual,
                field_id,
                entity_id,
                allowed.join(", ")
            ),
            error_type: ValidationErrorType::InvalidEnumValue {
                actual: actual.to_string(),
                allowed: allowed.to_vec(),
            },
        }
    }
}
