use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display};

use crate::{EntityType, FieldId, FieldType};

mod validation;
mod validation_errors;

pub use validation::ValidationResult;
pub use validation_errors::{ValidationError, ValidationErrorType};

/// Defines the mode of a field, either required or optional
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FieldMode {
    Required,
    Optional,
}

/// Defines the schema for an unnamed field which can be either required or optional.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldSchema {
    pub field_type: FieldType,
    pub field_mode: FieldMode,
    pub order: usize,
    pub allowed_values: Option<Vec<String>>,
}

impl FieldSchema {
    pub fn new(field_type: FieldType, field_mode: FieldMode, order: usize) -> Self {
        FieldSchema {
            field_type,
            field_mode,
            order,
            allowed_values: None,
        }
    }

    /// Creates a new enum field schema with allowed values (normalized to lowercase).
    pub fn new_enum(field_mode: FieldMode, order: usize, allowed_values: Vec<String>) -> Self {
        let normalized_values: Vec<String> = allowed_values
            .iter()
            .map(|v| v.trim().to_lowercase())
            .collect();

        FieldSchema {
            field_type: FieldType::Enum,
            field_mode,
            order,
            allowed_values: Some(normalized_values),
        }
    }

    /// Get the expected field type.
    pub fn expected_type(&self) -> &FieldType {
        &self.field_type
    }

    /// Check if the field is required.
    pub fn is_required(&self) -> bool {
        self.field_mode == FieldMode::Required
    }

    /// Get the allowed values for enum fields.
    pub fn allowed_values(&self) -> Option<&Vec<String>> {
        self.allowed_values.as_ref()
    }
}

/// Defines the schema for an entity type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySchema {
    pub entity_type: EntityType,
    pub fields: HashMap<FieldId, FieldSchema>,
    insertion_order: u16,
}

impl EntitySchema {
    /// Creates a new entity schema with a given name.
    pub fn new(entity_type: EntityType) -> Self {
        Self {
            entity_type,
            fields: HashMap::new(),
            insertion_order: 0,
        }
    }

    /// Builder method to add a field to the schema.
    pub fn add_field_schema(mut self, id: FieldId, field_schema: FieldSchema) -> Self {
        self.fields.insert(id, field_schema);
        self
    }

    /// Builder method to add a raw field.
    /// This does not preserve insertion order.
    pub fn with_raw_field(self, id: FieldId, schema: FieldSchema) -> Self {
        self.add_field_schema(id, schema)
    }

    /// Builder method to add a required field preserving insertion order.
    pub fn with_required_field(self, id: FieldId, field_type: FieldType) -> Self {
        let order = self.next_order();
        self.add_field_schema(id, FieldSchema::new(field_type, FieldMode::Required, order))
    }

    /// Builder method to add an optional field preserving insertion order.
    pub fn with_optional_field(self, id: FieldId, field_type: FieldType) -> Self {
        let order = self.next_order();
        self.add_field_schema(id, FieldSchema::new(field_type, FieldMode::Optional, order))
    }

    /// Builder method to add a required enum field with allowed values.
    pub fn with_required_enum(self, id: FieldId, allowed_values: Vec<String>) -> Self {
        let order = self.next_order();
        self.add_field_schema(
            id,
            FieldSchema::new_enum(FieldMode::Required, order, allowed_values),
        )
    }

    /// Builder method to add an optional enum field with allowed values.
    pub fn with_optional_enum(self, id: FieldId, allowed_values: Vec<String>) -> Self {
        let order = self.next_order();
        self.add_field_schema(
            id,
            FieldSchema::new_enum(FieldMode::Optional, order, allowed_values),
        )
    }

    /// Builder method to add common metadata fields to the schema.
    pub fn with_metadata(self) -> Self {
        self.with_raw_field(
            FieldId::new("notes"),
            FieldSchema::new(FieldType::String, FieldMode::Optional, 100),
        )
        .with_raw_field(
            FieldId::new("created_at"),
            FieldSchema::new(FieldType::DateTime, FieldMode::Optional, 101),
        )
        .with_raw_field(
            FieldId::new("updated_at"),
            FieldSchema::new(FieldType::DateTime, FieldMode::Optional, 102),
        )
    }

    /// Get schema fields sorted by their order.
    pub fn ordered_fields(&self) -> Vec<(&FieldId, &FieldSchema)> {
        let mut ordered: Vec<_> = self.fields.iter().collect();
        ordered.sort_by_key(|&(_, field_schema)| field_schema.order);
        ordered
    }

    /// Gets the next order for a field, preserving insertion order.
    fn next_order(&self) -> usize {
        self.fields.len()
    }
}

impl Display for EntitySchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.entity_type)?;

        for (field_id, field_schema) in &self.ordered_fields() {
            writeln!(f, "\n{}", field_id)?;
            writeln!(f, "- Type: {}", field_schema.expected_type())?;
            writeln!(f, "- Required: {}", field_schema.is_required())?;
            if let Some(allowed_values) = field_schema.allowed_values() {
                writeln!(f, "- Allowed values: {}", allowed_values.join(", "))?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_create_new() {
        let schema = EntitySchema::new(EntityType::new("person"))
            .with_required_field(FieldId::new("name"), FieldType::String)
            .with_optional_field(FieldId::new("email"), FieldType::String);

        assert_eq!(schema.entity_type, EntityType::new("person"));
        let name_field = &schema.fields[&FieldId::new("name")];
        assert_eq!(name_field.field_type, FieldType::String);
        assert_eq!(name_field.field_mode, FieldMode::Required);

        let email_field = &schema.fields[&FieldId::new("email")];
        assert_eq!(email_field.field_type, FieldType::String);
        assert_eq!(email_field.field_mode, FieldMode::Optional);
    }
}
