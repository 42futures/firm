//! Sum aggregation: sum numeric field values across entities

use super::super::filter::FieldRef;
use super::super::types::{AggregateValue, AggregationResult};
use super::super::QueryError;
use super::{NumericType, NumericValue, collect_numeric_values, classify_numeric_type, require_regular_field};
use crate::Entity;

pub fn execute(
    field: &FieldRef,
    entities: &[&Entity],
) -> Result<AggregationResult, QueryError> {
    let field_id = require_regular_field(field, "sum")?;
    let values = collect_numeric_values(field_id, entities)?;

    if values.is_empty() {
        return Ok(AggregationResult::Sum(AggregateValue::Integer(0)));
    }

    match classify_numeric_type(&values)? {
        NumericType::Integer => {
            let sum: i64 = values
                .iter()
                .map(|v| match v {
                    NumericValue::Integer(i) => *i,
                    _ => 0,
                })
                .sum();
            Ok(AggregationResult::Sum(AggregateValue::Integer(sum)))
        }
        NumericType::Float => {
            let sum: f64 = values.iter().map(|v| v.as_f64()).sum();
            Ok(AggregationResult::Sum(AggregateValue::Float(sum)))
        }
        NumericType::Currency(expected_currency) => {
            let mut total = rust_decimal::Decimal::ZERO;
            for v in &values {
                match v {
                    NumericValue::Currency { amount, currency } => {
                        if currency.code() != expected_currency.code() {
                            return Err(QueryError::InvalidAggregation {
                                message: format!(
                                    "Cannot sum mixed currencies (found {}, {}). \
                                     Filter first, e.g.: where {} >= 0 {}",
                                    expected_currency.code(),
                                    currency.code(),
                                    field_id.as_str(),
                                    expected_currency.code(),
                                ),
                            });
                        }
                        total += amount;
                    }
                    _ => unreachable!(),
                }
            }
            Ok(AggregationResult::Sum(AggregateValue::Currency {
                amount: total,
                currency: expected_currency,
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Entity, EntityId, EntityType, FieldId, FieldValue};
    use iso_currency::Currency;
    use rust_decimal::Decimal;

    fn make_integer_entities() -> Vec<Entity> {
        vec![
            Entity::new(EntityId::new("a"), EntityType::new("item"))
                .with_field(FieldId::new("val"), FieldValue::Integer(10)),
            Entity::new(EntityId::new("b"), EntityType::new("item"))
                .with_field(FieldId::new("val"), FieldValue::Integer(20)),
            Entity::new(EntityId::new("c"), EntityType::new("item"))
                .with_field(FieldId::new("val"), FieldValue::Integer(30)),
        ]
    }

    #[test]
    fn test_sum_integers() {
        let entities = make_integer_entities();
        let refs: Vec<&Entity> = entities.iter().collect();
        let field = FieldRef::Regular(FieldId::new("val"));
        let result = execute(&field, &refs).unwrap();
        assert_eq!(result, AggregationResult::Sum(AggregateValue::Integer(60)));
    }

    #[test]
    fn test_sum_floats() {
        let entities = vec![
            Entity::new(EntityId::new("a"), EntityType::new("item"))
                .with_field(FieldId::new("val"), FieldValue::Float(1.5)),
            Entity::new(EntityId::new("b"), EntityType::new("item"))
                .with_field(FieldId::new("val"), FieldValue::Float(2.5)),
        ];
        let refs: Vec<&Entity> = entities.iter().collect();
        let field = FieldRef::Regular(FieldId::new("val"));
        let result = execute(&field, &refs).unwrap();
        assert_eq!(result, AggregationResult::Sum(AggregateValue::Float(4.0)));
    }

    #[test]
    fn test_sum_currency_same_code() {
        let entities = vec![
            Entity::new(EntityId::new("a"), EntityType::new("invoice"))
                .with_field(
                    FieldId::new("amount"),
                    FieldValue::Currency {
                        amount: Decimal::new(10000, 2),
                        currency: Currency::USD,
                    },
                ),
            Entity::new(EntityId::new("b"), EntityType::new("invoice"))
                .with_field(
                    FieldId::new("amount"),
                    FieldValue::Currency {
                        amount: Decimal::new(5000, 2),
                        currency: Currency::USD,
                    },
                ),
        ];
        let refs: Vec<&Entity> = entities.iter().collect();
        let field = FieldRef::Regular(FieldId::new("amount"));
        let result = execute(&field, &refs).unwrap();
        assert_eq!(
            result,
            AggregationResult::Sum(AggregateValue::Currency {
                amount: Decimal::new(15000, 2),
                currency: Currency::USD,
            })
        );
    }

    #[test]
    fn test_sum_currency_mixed_codes_error() {
        let entities = vec![
            Entity::new(EntityId::new("a"), EntityType::new("invoice"))
                .with_field(
                    FieldId::new("amount"),
                    FieldValue::Currency {
                        amount: Decimal::new(100, 0),
                        currency: Currency::USD,
                    },
                ),
            Entity::new(EntityId::new("b"), EntityType::new("invoice"))
                .with_field(
                    FieldId::new("amount"),
                    FieldValue::Currency {
                        amount: Decimal::new(200, 0),
                        currency: Currency::EUR,
                    },
                ),
        ];
        let refs: Vec<&Entity> = entities.iter().collect();
        let field = FieldRef::Regular(FieldId::new("amount"));
        let result = execute(&field, &refs);
        assert!(matches!(
            result,
            Err(QueryError::InvalidAggregation { .. })
        ));
    }

    #[test]
    fn test_sum_non_numeric_error() {
        let entities = vec![Entity::new(EntityId::new("a"), EntityType::new("item"))
            .with_field(FieldId::new("name"), "hello")];
        let refs: Vec<&Entity> = entities.iter().collect();
        let field = FieldRef::Regular(FieldId::new("name"));
        let result = execute(&field, &refs);
        assert!(matches!(
            result,
            Err(QueryError::InvalidAggregation { .. })
        ));
    }

    #[test]
    fn test_sum_empty_set() {
        let refs: Vec<&Entity> = vec![];
        let field = FieldRef::Regular(FieldId::new("val"));
        let result = execute(&field, &refs).unwrap();
        assert_eq!(result, AggregationResult::Sum(AggregateValue::Integer(0)));
    }

    #[test]
    fn test_sum_skips_missing_fields() {
        let entities = vec![
            Entity::new(EntityId::new("a"), EntityType::new("item"))
                .with_field(FieldId::new("val"), FieldValue::Integer(10)),
            Entity::new(EntityId::new("b"), EntityType::new("item")),
            // b has no "val" field
        ];
        let refs: Vec<&Entity> = entities.iter().collect();
        let field = FieldRef::Regular(FieldId::new("val"));
        let result = execute(&field, &refs).unwrap();
        assert_eq!(result, AggregationResult::Sum(AggregateValue::Integer(10)));
    }

    #[test]
    fn test_sum_metadata_field_error() {
        let entities = make_integer_entities();
        let refs: Vec<&Entity> = entities.iter().collect();
        let field = FieldRef::Metadata(super::super::super::filter::MetadataField::Id);
        let result = execute(&field, &refs);
        assert!(matches!(
            result,
            Err(QueryError::InvalidAggregation { .. })
        ));
    }
}
