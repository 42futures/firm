use convert_case::{Case, Casing};
use serde::{Deserialize, Serialize};
use std::fmt;

use super::{EntityId, EntityType, FieldId, FieldValue};

/// Represents a business entity in the Firm graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entity {
    pub id: EntityId,
    pub entity_type: EntityType,
    pub fields: Vec<(FieldId, FieldValue)>,
}

impl Entity {
    /// Creates a new entity with the desired ID and type.
    pub fn new(id: EntityId, entity_type: EntityType) -> Self {
        Self {
            id,
            entity_type,
            fields: Vec::new(),
        }
    }

    /// Builder method to add a field to a new entity.
    pub fn with_field<V>(mut self, id: FieldId, value: V) -> Self
    where
        V: Into<FieldValue>,
    {
        self.fields.push((id, value.into()));
        self
    }

    /// Try to get a entity field value for a given field ID.
    pub fn get_field(&self, id: &FieldId) -> Option<&FieldValue> {
        self.fields
            .iter()
            .find(|(field_id, _)| field_id == id)
            .map(|(_, field_value)| field_value)
    }
}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}\n", self.id)?;
        for (field_id, field_value) in &self.fields {
            writeln!(
                f,
                "{}: {}",
                field_id.as_str().to_case(Case::Sentence),
                field_value
            )?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_create_new() {
        let person = Entity::new(EntityId::new("john_doe"), EntityType::new("person"));

        assert_eq!(person.id, EntityId::new("john_doe"));
        assert_eq!(person.entity_type, EntityType::new("person"));
        assert!(person.fields.is_empty());
    }

    #[test]
    fn test_entity_with_fields() {
        let person = Entity::new(EntityId::new("john_doe"), EntityType::new("person"))
            .with_field(FieldId::new("name"), "John Doe")
            .with_field(FieldId::new("email"), "john@example.com");

        assert_eq!(
            person.get_field(&FieldId::new("name")),
            Some(&FieldValue::String(String::from("John Doe")))
        );
        assert_eq!(
            person.get_field(&FieldId::new("email")),
            Some(&FieldValue::String(String::from("john@example.com")))
        );
        assert_eq!(person.get_field(&FieldId::new("nonexistant")), None);
    }

    #[test]
    fn test_entity_different_types() {
        let person = Entity::new(EntityId::new("john_doe"), EntityType::new("person"));
        let organization = Entity::new(EntityId::new("megacorp"), EntityType::new("organization"));

        assert_eq!(person.entity_type, EntityType::new("person"));
        assert_eq!(organization.entity_type, EntityType::new("organization"));
    }
}
