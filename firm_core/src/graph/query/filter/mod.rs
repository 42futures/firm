//! Filter condition types and matching logic

mod types;
mod string;
mod numeric;
mod boolean;
mod currency;
mod datetime;
mod reference;
mod list;

// Re-export types
pub use types::*;

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
    pub fn matches(&self, entity: &Entity) -> bool {
        match &self.field {
            FieldRef::Metadata(metadata) => self.matches_metadata(entity, metadata),
            FieldRef::Regular(field_id) => self.matches_field(entity, field_id),
        }
    }

    fn matches_metadata(&self, entity: &Entity, metadata: &MetadataField) -> bool {
        match metadata {
            MetadataField::Type => {
                let entity_type = entity.entity_type.to_string();
                string::compare_string(&entity_type, &self.operator, &self.value)
            }
            MetadataField::Id => {
                let entity_id = entity.id.to_string();
                string::compare_string(&entity_id, &self.operator, &self.value)
            }
        }
    }

    fn matches_field(&self, entity: &Entity, field_id: &FieldId) -> bool {
        // Get the field value from the entity
        let field_value = match entity.get_field(field_id) {
            Some(value) => value,
            None => return false, // Field doesn't exist, condition fails
        };

        // Compare based on field value type
        match field_value {
            FieldValue::String(s) => string::compare_string(s, &self.operator, &self.value),
            FieldValue::Integer(i) => numeric::compare_integer(*i, &self.operator, &self.value),
            FieldValue::Float(f) => numeric::compare_float(*f, &self.operator, &self.value),
            FieldValue::Boolean(b) => boolean::compare_boolean(*b, &self.operator, &self.value),
            FieldValue::Currency { amount, currency } => {
                currency::compare_currency(amount, currency, &self.operator, &self.value)
            }
            FieldValue::DateTime(dt) => datetime::compare_datetime(dt, &self.operator, &self.value),
            FieldValue::Enum(s) => string::compare_string(s, &self.operator, &self.value),
            FieldValue::Path(p) => {
                if let Some(path_str) = p.to_str() {
                    string::compare_string(path_str, &self.operator, &self.value)
                } else {
                    false
                }
            }
            FieldValue::Reference(ref_value) => {
                reference::compare_reference(ref_value, &self.operator, &self.value)
            }
            FieldValue::List(items) => list::compare_list(items, &self.operator, &self.value),
        }
    }
}
