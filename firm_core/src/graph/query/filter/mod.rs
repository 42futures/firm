//! Filter condition types and matching logic

mod boolean;
mod currency;
mod datetime;
mod list;
mod numeric;
mod reference;
mod string;
mod types;

// Re-export types
pub use types::*;

use super::QueryError;
use crate::{Entity, FieldId, FieldValue};

/// A filter condition for matching entities
#[derive(Debug, Clone, PartialEq)]
pub struct FilterCondition {
    pub field: FieldRef,
    pub operator: FilterOperator,
    pub value: FilterValue,
}

impl FilterCondition {
    /// Create a new filter condition
    pub fn new(field: FieldRef, operator: FilterOperator, value: FilterValue) -> Self {
        Self {
            field,
            operator,
            value,
        }
    }

    /// Check if an entity matches this condition
    pub fn matches(&self, entity: &Entity) -> Result<bool, QueryError> {
        match &self.field {
            FieldRef::Metadata(metadata) => self.matches_metadata(entity, metadata),
            FieldRef::Regular(field_id) => self.matches_field(entity, field_id),
        }
    }

    fn matches_metadata(
        &self,
        entity: &Entity,
        metadata: &MetadataField,
    ) -> Result<bool, QueryError> {
        // Create a synthetic FieldValue for metadata comparisons
        let field_value = match metadata {
            MetadataField::Type => FieldValue::String(entity.entity_type.to_string()),
            MetadataField::Id => FieldValue::String(entity.id.to_string()),
        };
        string::compare_string(&field_value, &self.operator, &self.value)
    }

    fn matches_field(&self, entity: &Entity, field_id: &FieldId) -> Result<bool, QueryError> {
        // Get the field value from the entity
        let field_value = match entity.get_field(field_id) {
            Some(value) => value,
            None => return Ok(false), // Field doesn't exist, condition fails
        };

        // Compare based on field value type - now we pass the FieldValue directly
        match field_value {
            FieldValue::String(_) | FieldValue::Enum(_) | FieldValue::Path(_) => {
                string::compare_string(field_value, &self.operator, &self.value)
            }
            FieldValue::Integer(_) => {
                numeric::compare_integer(field_value, &self.operator, &self.value)
            }
            FieldValue::Float(_) => {
                numeric::compare_float(field_value, &self.operator, &self.value)
            }
            FieldValue::Boolean(_) => {
                boolean::compare_boolean(field_value, &self.operator, &self.value)
            }
            FieldValue::Currency { .. } => {
                currency::compare_currency(field_value, &self.operator, &self.value)
            }
            FieldValue::DateTime(_) => {
                datetime::compare_datetime(field_value, &self.operator, &self.value)
            }
            FieldValue::Reference(_) => {
                reference::compare_reference(field_value, &self.operator, &self.value)
            }
            FieldValue::List(_) => list::compare_list(field_value, &self.operator, &self.value),
        }
    }
}
