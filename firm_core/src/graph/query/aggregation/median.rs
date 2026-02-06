//! Median aggregation: compute the median of a numeric field

use super::super::filter::FieldRef;
use super::super::types::AggregationResult;
use super::super::QueryError;
use super::{collect_numeric_values, require_regular_field};
use crate::Entity;

pub fn execute(
    field: &FieldRef,
    entities: &[&Entity],
) -> Result<AggregationResult, QueryError> {
    let field_id = require_regular_field(field, "median")?;
    let values = collect_numeric_values(field_id, entities)?;

    if values.is_empty() {
        return Err(QueryError::InvalidAggregation {
            message: "Cannot compute median of empty result set".to_string(),
        });
    }

    let mut float_values: Vec<f64> = values.iter().map(|v| v.as_f64()).collect();
    float_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let len = float_values.len();
    let median = if len % 2 == 0 {
        (float_values[len / 2 - 1] + float_values[len / 2]) / 2.0
    } else {
        float_values[len / 2]
    };

    Ok(AggregationResult::Median(median))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Entity, EntityId, EntityType, FieldId, FieldValue};

    #[test]
    fn test_median_odd_count() {
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
        assert_eq!(result, AggregationResult::Median(20.0));
    }

    #[test]
    fn test_median_mixed_integer_and_float() {
        let entities = vec![
            Entity::new(EntityId::new("a"), EntityType::new("item"))
                .with_field(FieldId::new("val"), FieldValue::Integer(10)),
            Entity::new(EntityId::new("b"), EntityType::new("item"))
                .with_field(FieldId::new("val"), FieldValue::Float(20.0)),
            Entity::new(EntityId::new("c"), EntityType::new("item"))
                .with_field(FieldId::new("val"), FieldValue::Integer(30)),
        ];
        let refs: Vec<&Entity> = entities.iter().collect();
        let field = FieldRef::Regular(FieldId::new("val"));
        let result = execute(&field, &refs).unwrap();
        assert_eq!(result, AggregationResult::Median(20.0));
    }

    #[test]
    fn test_median_even_count() {
        let entities = vec![
            Entity::new(EntityId::new("a"), EntityType::new("item"))
                .with_field(FieldId::new("val"), FieldValue::Integer(10)),
            Entity::new(EntityId::new("b"), EntityType::new("item"))
                .with_field(FieldId::new("val"), FieldValue::Integer(20)),
        ];
        let refs: Vec<&Entity> = entities.iter().collect();
        let field = FieldRef::Regular(FieldId::new("val"));
        let result = execute(&field, &refs).unwrap();
        assert_eq!(result, AggregationResult::Median(15.0));
    }

    #[test]
    fn test_median_single_value() {
        let entities = vec![Entity::new(EntityId::new("a"), EntityType::new("item"))
            .with_field(FieldId::new("val"), FieldValue::Integer(42))];
        let refs: Vec<&Entity> = entities.iter().collect();
        let field = FieldRef::Regular(FieldId::new("val"));
        let result = execute(&field, &refs).unwrap();
        assert_eq!(result, AggregationResult::Median(42.0));
    }

    #[test]
    fn test_median_unsorted_input() {
        let entities = vec![
            Entity::new(EntityId::new("a"), EntityType::new("item"))
                .with_field(FieldId::new("val"), FieldValue::Integer(30)),
            Entity::new(EntityId::new("b"), EntityType::new("item"))
                .with_field(FieldId::new("val"), FieldValue::Integer(10)),
            Entity::new(EntityId::new("c"), EntityType::new("item"))
                .with_field(FieldId::new("val"), FieldValue::Integer(20)),
        ];
        let refs: Vec<&Entity> = entities.iter().collect();
        let field = FieldRef::Regular(FieldId::new("val"));
        let result = execute(&field, &refs).unwrap();
        assert_eq!(result, AggregationResult::Median(20.0));
    }

    #[test]
    fn test_median_empty_error() {
        let refs: Vec<&Entity> = vec![];
        let field = FieldRef::Regular(FieldId::new("val"));
        let result = execute(&field, &refs);
        assert!(matches!(
            result,
            Err(QueryError::InvalidAggregation { .. })
        ));
    }
}
