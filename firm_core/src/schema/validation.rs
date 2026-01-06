use log::debug;

use super::{EntitySchema, ValidationError};
use crate::Entity;

pub type ValidationResult = Result<(), Vec<ValidationError>>;

impl EntitySchema {
    /// Validates an entity against the schema.
    pub fn validate(&self, entity: &Entity) -> ValidationResult {
        debug!(
            "Validating entity: '{}' for schema: '{}'",
            entity.id, self.entity_type
        );

        let mut errors = Vec::new();

        // Check the entity type against the schema
        if entity.entity_type != self.entity_type {
            errors.push(ValidationError::mismatched_entity_type(
                &entity.id,
                &self.entity_type,
                &entity.entity_type,
            ))
        }

        // Check each field in the schema
        for (field_name, field_schema) in &self.fields {
            match entity.get_field(field_name) {
                // Entity has the field: Check that it has desired type
                Some(field_value) => {
                    let expected_type = field_schema.expected_type();
                    if !field_value.is_type(expected_type) {
                        errors.push(ValidationError::mismatched_field_type(
                            &entity.id,
                            field_name,
                            expected_type,
                            &field_value.get_type(),
                        ));
                    } else if let crate::field::FieldValue::Enum(value) = field_value {
                        // For enum fields, validate against allowed values
                        if let Some(allowed_values) = field_schema.allowed_values() {
                            let normalized_value = value.trim().to_lowercase();
                            if !allowed_values.contains(&normalized_value) {
                                errors.push(ValidationError::invalid_enum_value(
                                    &entity.id,
                                    field_name,
                                    value,
                                    allowed_values,
                                ));
                            }
                        } else {
                            errors.push(ValidationError::invalid_enum_value(
                                &entity.id,
                                field_name,
                                value,
                                &[],
                            ));
                        }
                    }
                }
                // Entity does not have the field: Check if it's required
                None => {
                    if field_schema.is_required() {
                        errors.push(ValidationError::missing_field(&entity.id, field_name));
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            debug!(
                "Entity '{}' failed validation with {} errors",
                entity.id,
                errors.len()
            );
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::ValidationErrorType;
    use crate::{
        EntityId, EntityType, FieldId,
        field::{FieldType, FieldValue},
    };
    use assert_matches::assert_matches;

    #[test]
    fn test_validate_ok() {
        let schema = EntitySchema::new(EntityType::new("person"))
            .with_required_field(FieldId::new("name"), FieldType::String)
            .with_optional_field(FieldId::new("email"), FieldType::String);

        let entity = Entity::new(EntityId::new("test_person"), EntityType::new("person"))
            .with_field(
                FieldId::new("name"),
                FieldValue::String(String::from("John Doe")),
            );

        let result = schema.validate(&entity);

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_error_mismatched_entity_types() {
        let schema = EntitySchema::new(EntityType::new("test_a"));
        let entity = Entity::new(EntityId::new("test"), EntityType::new("test_b"));

        let result = schema.validate(&entity);

        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);

        assert_matches!(
            &errors[0].error_type,
            ValidationErrorType::MismatchedEntityType { expected, actual } if expected == &EntityType::new("test_a") && actual == &EntityType::new("test_b")
        );
    }

    #[test]
    fn test_validate_error_missing_field() {
        let schema = EntitySchema::new(EntityType::new("person"))
            .with_required_field(FieldId::new("name"), FieldType::String)
            .with_required_field(FieldId::new("email"), FieldType::String);

        let entity = Entity::new(EntityId::new("test_person"), EntityType::new("person"))
            .with_field(
                FieldId::new("name"),
                FieldValue::String(String::from("John Doe")),
            );

        let result = schema.validate(&entity);

        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);

        assert_matches!(
            &errors[0].error_type,
            ValidationErrorType::MissingRequiredField { required } if required == &FieldId::new("email")
        );
    }

    #[test]
    fn test_validate_error_mismatched_field_types() {
        let schema = EntitySchema::new(EntityType::new("person"))
            .with_required_field(FieldId::new("is_nice"), FieldType::Boolean);

        let entity = Entity::new(EntityId::new("test_person"), EntityType::new("person"))
            .with_field(
                FieldId::new("is_nice"),
                FieldValue::String("Sure".to_string()),
            );

        let result = schema.validate(&entity);

        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);

        assert_matches!(
            &errors[0].error_type,
            ValidationErrorType::MismatchedFieldType { expected, actual } if expected == &FieldType::Boolean && actual == &FieldType::String
        );
    }

    #[test]
    fn test_validate_enum_with_valid_value() {
        let schema = EntitySchema::new(EntityType::new("account"))
            .with_required_enum(
                FieldId::new("status"),
                vec!["prospect".to_string(), "customer".to_string(), "partner".to_string()],
            );

        let entity = Entity::new(EntityId::new("test_account"), EntityType::new("account"))
            .with_field(
                FieldId::new("status"),
                FieldValue::Enum("customer".to_string()),
            );

        let result = schema.validate(&entity);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_enum_with_case_insensitive_match() {
        let schema = EntitySchema::new(EntityType::new("account"))
            .with_required_enum(
                FieldId::new("status"),
                vec!["prospect".to_string(), "customer".to_string()],
            );

        let entity = Entity::new(EntityId::new("test_account"), EntityType::new("account"))
            .with_field(
                FieldId::new("status"),
                FieldValue::Enum("CUSTOMER".to_string()),
            );

        let result = schema.validate(&entity);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_enum_with_whitespace_trimmed() {
        let schema = EntitySchema::new(EntityType::new("account"))
            .with_required_enum(
                FieldId::new("status"),
                vec!["prospect".to_string(), "customer".to_string()],
            );

        let entity = Entity::new(EntityId::new("test_account"), EntityType::new("account"))
            .with_field(
                FieldId::new("status"),
                FieldValue::Enum("  customer  ".to_string()),
            );

        let result = schema.validate(&entity);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_enum_with_invalid_value() {
        let schema = EntitySchema::new(EntityType::new("account"))
            .with_required_enum(
                FieldId::new("status"),
                vec!["prospect".to_string(), "customer".to_string(), "partner".to_string()],
            );

        let entity = Entity::new(EntityId::new("test_account"), EntityType::new("account"))
            .with_field(
                FieldId::new("status"),
                FieldValue::Enum("client".to_string()),
            );

        let result = schema.validate(&entity);

        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);

        assert_matches!(
            &errors[0].error_type,
            ValidationErrorType::InvalidEnumValue { actual, allowed }
            if actual == "client" && allowed == &vec!["prospect".to_string(), "customer".to_string(), "partner".to_string()]
        );
    }

    #[test]
    fn test_validate_optional_enum_can_be_missing() {
        let schema = EntitySchema::new(EntityType::new("account"))
            .with_optional_enum(
                FieldId::new("status"),
                vec!["prospect".to_string(), "customer".to_string()],
            );

        let entity = Entity::new(EntityId::new("test_account"), EntityType::new("account"));

        let result = schema.validate(&entity);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_optional_enum_validates_when_present() {
        let schema = EntitySchema::new(EntityType::new("account"))
            .with_optional_enum(
                FieldId::new("status"),
                vec!["prospect".to_string(), "customer".to_string()],
            );

        let entity = Entity::new(EntityId::new("test_account"), EntityType::new("account"))
            .with_field(
                FieldId::new("status"),
                FieldValue::Enum("invalid".to_string()),
            );

        let result = schema.validate(&entity);

        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);

        assert_matches!(
            &errors[0].error_type,
            ValidationErrorType::InvalidEnumValue { actual, .. } if actual == "invalid"
        );
    }
}
