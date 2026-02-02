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
}

/// A compound filter condition combining multiple conditions with a logical operator
#[derive(Debug, Clone, PartialEq)]
pub struct CompoundFilterCondition {
    pub conditions: Vec<FilterCondition>,
    pub combinator: Combinator,
}

impl CompoundFilterCondition {
    /// Create a new compound filter condition
    pub fn new(conditions: Vec<FilterCondition>, combinator: Combinator) -> Self {
        Self {
            conditions,
            combinator,
        }
    }

    /// Create a compound condition with a single filter (AND by default)
    pub fn single(condition: FilterCondition) -> Self {
        Self {
            conditions: vec![condition],
            combinator: Combinator::default(),
        }
    }

    /// Check if an entity matches this compound condition
    pub fn matches(&self, entity: &Entity) -> Result<bool, QueryError> {
        let results: Result<Vec<bool>, QueryError> = self
            .conditions
            .iter()
            .map(|c| c.matches(entity))
            .collect();

        Ok(match self.combinator {
            Combinator::And => results?.iter().all(|&r| r),
            Combinator::Or => results?.iter().any(|&r| r),
        })
    }
}

impl FilterCondition {

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Entity, EntityId, EntityType, FieldId, FieldValue};

    fn make_test_entity(name: &str, age: i64, active: bool) -> Entity {
        Entity::new(EntityId::new("test"), EntityType::new("person"))
            .with_field(FieldId::new("name"), name)
            .with_field(FieldId::new("age"), FieldValue::Integer(age))
            .with_field(FieldId::new("active"), active)
    }

    #[test]
    fn test_compound_condition_single() {
        let entity = make_test_entity("Alice", 30, true);
        let condition = CompoundFilterCondition::single(FilterCondition::new(
            FieldRef::Regular(FieldId::new("name")),
            FilterOperator::Equal,
            FilterValue::String("Alice".to_string()),
        ));

        assert!(condition.matches(&entity).unwrap());
    }

    #[test]
    fn test_compound_condition_and_all_match() {
        let entity = make_test_entity("Alice", 30, true);
        let condition = CompoundFilterCondition::new(
            vec![
                FilterCondition::new(
                    FieldRef::Regular(FieldId::new("name")),
                    FilterOperator::Equal,
                    FilterValue::String("Alice".to_string()),
                ),
                FilterCondition::new(
                    FieldRef::Regular(FieldId::new("age")),
                    FilterOperator::GreaterThan,
                    FilterValue::Integer(25),
                ),
            ],
            Combinator::And,
        );

        assert!(condition.matches(&entity).unwrap());
    }

    #[test]
    fn test_compound_condition_and_one_fails() {
        let entity = make_test_entity("Alice", 30, true);
        let condition = CompoundFilterCondition::new(
            vec![
                FilterCondition::new(
                    FieldRef::Regular(FieldId::new("name")),
                    FilterOperator::Equal,
                    FilterValue::String("Alice".to_string()),
                ),
                FilterCondition::new(
                    FieldRef::Regular(FieldId::new("age")),
                    FilterOperator::GreaterThan,
                    FilterValue::Integer(35), // Alice is 30, so this fails
                ),
            ],
            Combinator::And,
        );

        assert!(!condition.matches(&entity).unwrap());
    }

    #[test]
    fn test_compound_condition_or_one_matches() {
        let entity = make_test_entity("Alice", 30, true);
        let condition = CompoundFilterCondition::new(
            vec![
                FilterCondition::new(
                    FieldRef::Regular(FieldId::new("name")),
                    FilterOperator::Equal,
                    FilterValue::String("Bob".to_string()), // Doesn't match
                ),
                FilterCondition::new(
                    FieldRef::Regular(FieldId::new("age")),
                    FilterOperator::Equal,
                    FilterValue::Integer(30), // Matches
                ),
            ],
            Combinator::Or,
        );

        assert!(condition.matches(&entity).unwrap());
    }

    #[test]
    fn test_compound_condition_or_none_match() {
        let entity = make_test_entity("Alice", 30, true);
        let condition = CompoundFilterCondition::new(
            vec![
                FilterCondition::new(
                    FieldRef::Regular(FieldId::new("name")),
                    FilterOperator::Equal,
                    FilterValue::String("Bob".to_string()),
                ),
                FilterCondition::new(
                    FieldRef::Regular(FieldId::new("age")),
                    FilterOperator::Equal,
                    FilterValue::Integer(25),
                ),
            ],
            Combinator::Or,
        );

        assert!(!condition.matches(&entity).unwrap());
    }

    #[test]
    fn test_compound_condition_or_multiple_values_same_field() {
        // This is the primary use case: where status = "draft" or status = "sent"
        let entity = make_test_entity("Alice", 30, true);
        let condition = CompoundFilterCondition::new(
            vec![
                FilterCondition::new(
                    FieldRef::Regular(FieldId::new("name")),
                    FilterOperator::Equal,
                    FilterValue::String("Alice".to_string()),
                ),
                FilterCondition::new(
                    FieldRef::Regular(FieldId::new("name")),
                    FilterOperator::Equal,
                    FilterValue::String("Bob".to_string()),
                ),
            ],
            Combinator::Or,
        );

        assert!(condition.matches(&entity).unwrap());
    }
}
