//! Count aggregation: count entities, optionally filtering by field presence

use super::super::filter::FieldRef;
use super::super::types::AggregationResult;
use super::super::QueryError;
use crate::Entity;

pub fn execute(
    field: Option<&FieldRef>,
    entities: &[&Entity],
) -> Result<AggregationResult, QueryError> {
    let count = match field {
        None => entities.len(),
        Some(FieldRef::Metadata(_)) => entities.len(),
        Some(FieldRef::Regular(field_id)) => entities
            .iter()
            .filter(|e| e.get_field(field_id).is_some())
            .count(),
    };
    Ok(AggregationResult::Count(count))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Entity, EntityId, EntityType, FieldId, FieldValue};
    use super::super::super::filter::MetadataField;

    fn make_entities() -> Vec<Entity> {
        vec![
            Entity::new(EntityId::new("p1"), EntityType::new("person"))
                .with_field(FieldId::new("name"), "Alice")
                .with_field(FieldId::new("age"), FieldValue::Integer(30)),
            Entity::new(EntityId::new("p2"), EntityType::new("person"))
                .with_field(FieldId::new("name"), "Bob"),
            // p2 has no "age" field
        ]
    }

    #[test]
    fn test_count_all() {
        let entities = make_entities();
        let refs: Vec<&Entity> = entities.iter().collect();
        let result = execute(None, &refs).unwrap();
        assert_eq!(result, AggregationResult::Count(2));
    }

    #[test]
    fn test_count_with_present_field() {
        let entities = make_entities();
        let refs: Vec<&Entity> = entities.iter().collect();
        let field = FieldRef::Regular(FieldId::new("name"));
        let result = execute(Some(&field), &refs).unwrap();
        assert_eq!(result, AggregationResult::Count(2));
    }

    #[test]
    fn test_count_with_partial_field() {
        let entities = make_entities();
        let refs: Vec<&Entity> = entities.iter().collect();
        let field = FieldRef::Regular(FieldId::new("age"));
        let result = execute(Some(&field), &refs).unwrap();
        assert_eq!(result, AggregationResult::Count(1));
    }

    #[test]
    fn test_count_with_missing_field() {
        let entities = make_entities();
        let refs: Vec<&Entity> = entities.iter().collect();
        let field = FieldRef::Regular(FieldId::new("nonexistent"));
        let result = execute(Some(&field), &refs).unwrap();
        assert_eq!(result, AggregationResult::Count(0));
    }

    #[test]
    fn test_count_metadata_field_counts_all() {
        let entities = make_entities();
        let refs: Vec<&Entity> = entities.iter().collect();
        let field = FieldRef::Metadata(MetadataField::Id);
        let result = execute(Some(&field), &refs).unwrap();
        assert_eq!(result, AggregationResult::Count(2));
    }

    #[test]
    fn test_count_empty_set() {
        let refs: Vec<&Entity> = vec![];
        let result = execute(None, &refs).unwrap();
        assert_eq!(result, AggregationResult::Count(0));
    }
}
