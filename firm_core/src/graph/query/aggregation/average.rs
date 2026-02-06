//! Average aggregation: compute the mean of a numeric field

use super::super::filter::FieldRef;
use super::super::types::AggregationResult;
use super::super::QueryError;
use super::{collect_numeric_values, require_regular_field};
use crate::Entity;

pub fn execute(
    field: &FieldRef,
    entities: &[&Entity],
) -> Result<AggregationResult, QueryError> {
    let field_id = require_regular_field(field, "average")?;
    let values = collect_numeric_values(field_id, entities)?;

    if values.is_empty() {
        return Err(QueryError::InvalidAggregation {
            message: "Cannot compute average of empty result set".to_string(),
        });
    }

    let sum: f64 = values.iter().map(|v| v.as_f64()).sum();
    let avg = sum / values.len() as f64;

    Ok(AggregationResult::Average(avg))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Entity, EntityId, EntityType, FieldId, FieldValue};

    #[test]
    fn test_average_integers() {
        let entities = vec![
            Entity::new(EntityId::new("a"), EntityType::new("item"))
                .with_field(FieldId::new("val"), FieldValue::Integer(10)),
            Entity::new(EntityId::new("b"), EntityType::new("item"))
                .with_field(FieldId::new("val"), FieldValue::Integer(20)),
            Entity::new(EntityId::new("c"), EntityType::new("item"))
                .with_field(FieldId::new("val"), FieldValue::Integer(30)),
        ];
        let refs: Vec<&Entity> = entities.iter().collect();
        let field = FieldRef::Regular(FieldId::new("val"));
        let result = execute(&field, &refs).unwrap();
        assert_eq!(result, AggregationResult::Average(20.0));
    }

    #[test]
    fn test_average_floats() {
        let entities = vec![
            Entity::new(EntityId::new("a"), EntityType::new("item"))
                .with_field(FieldId::new("val"), FieldValue::Float(1.0)),
            Entity::new(EntityId::new("b"), EntityType::new("item"))
                .with_field(FieldId::new("val"), FieldValue::Float(2.0)),
        ];
        let refs: Vec<&Entity> = entities.iter().collect();
        let field = FieldRef::Regular(FieldId::new("val"));
        let result = execute(&field, &refs).unwrap();
        assert_eq!(result, AggregationResult::Average(1.5));
    }

    #[test]
    fn test_average_skips_missing_fields() {
        let entities = vec![
            Entity::new(EntityId::new("a"), EntityType::new("item"))
                .with_field(FieldId::new("val"), FieldValue::Integer(10)),
            Entity::new(EntityId::new("b"), EntityType::new("item")),
        ];
        let refs: Vec<&Entity> = entities.iter().collect();
        let field = FieldRef::Regular(FieldId::new("val"));
        let result = execute(&field, &refs).unwrap();
        // Only 1 entity has the field, so average = 10/1
        assert_eq!(result, AggregationResult::Average(10.0));
    }

    #[test]
    fn test_average_empty_result_set_error() {
        let refs: Vec<&Entity> = vec![];
        let field = FieldRef::Regular(FieldId::new("val"));
        let result = execute(&field, &refs);
        assert!(matches!(
            result,
            Err(QueryError::InvalidAggregation { .. })
        ));
    }

    #[test]
    fn test_average_no_entities_with_field_error() {
        let entities = vec![Entity::new(EntityId::new("a"), EntityType::new("item"))];
        let refs: Vec<&Entity> = entities.iter().collect();
        let field = FieldRef::Regular(FieldId::new("nonexistent"));
        let result = execute(&field, &refs);
        assert!(matches!(
            result,
            Err(QueryError::InvalidAggregation { .. })
        ));
    }
}
