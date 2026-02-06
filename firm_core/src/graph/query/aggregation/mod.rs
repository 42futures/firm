//! Aggregation execution logic for queries

mod average;
mod count;
mod median;
mod select;
mod sum;

use super::filter::FieldRef;
use super::types::{Aggregation, AggregationResult};
use super::QueryError;
use crate::Entity;

impl Aggregation {
    /// Execute this aggregation over a set of entities
    pub fn execute(&self, entities: &[&Entity]) -> Result<AggregationResult, QueryError> {
        match self {
            Aggregation::Select(fields) => select::execute(fields, entities),
            Aggregation::Count(field) => count::execute(field.as_ref(), entities),
            Aggregation::Sum(field) => sum::execute(field, entities),
            Aggregation::Average(field) => average::execute(field, entities),
            Aggregation::Median(field) => median::execute(field, entities),
        }
    }
}

/// Require that the field is a regular field (not metadata) for numeric aggregations.
fn require_regular_field<'a>(
    field: &'a FieldRef,
    operation: &str,
) -> Result<&'a crate::FieldId, QueryError> {
    match field {
        FieldRef::Regular(id) => Ok(id),
        FieldRef::Metadata(_) => Err(QueryError::InvalidAggregation {
            message: format!(
                "Cannot {} a metadata field. Use a regular numeric field.",
                operation
            ),
        }),
    }
}

/// Internal representation of a numeric value extracted from an entity field.
#[derive(Debug, Clone)]
enum NumericValue {
    Integer(i64),
    Float(f64),
    Currency {
        amount: rust_decimal::Decimal,
        currency: iso_currency::Currency,
    },
}

impl NumericValue {
    fn as_f64(&self) -> f64 {
        match self {
            NumericValue::Integer(i) => *i as f64,
            NumericValue::Float(f) => *f,
            NumericValue::Currency { amount, .. } => {
                use rust_decimal::prelude::ToPrimitive;
                amount.to_f64().unwrap_or(0.0)
            }
        }
    }
}

/// Classifies the dominant numeric type across a set of values.
enum NumericType {
    Integer,
    Float,
    Currency(iso_currency::Currency),
}

/// Classify what numeric type a set of values represents, handling mixed int/float promotion.
fn classify_numeric_type(values: &[NumericValue]) -> Result<NumericType, QueryError> {
    let mut has_integer = false;
    let mut has_float = false;
    let mut currency: Option<iso_currency::Currency> = None;

    for v in values {
        match v {
            NumericValue::Integer(_) => has_integer = true,
            NumericValue::Float(_) => has_float = true,
            NumericValue::Currency { currency: c, .. } => {
                currency = Some(*c);
            }
        }
    }

    let has_currency = currency.is_some();

    if has_currency && (has_integer || has_float) {
        return Err(QueryError::InvalidAggregation {
            message: "Cannot mix currency and numeric values in aggregation".to_string(),
        });
    }

    if has_currency {
        Ok(NumericType::Currency(currency.unwrap()))
    } else if has_float {
        Ok(NumericType::Float)
    } else {
        Ok(NumericType::Integer)
    }
}

/// Collect numeric values from entities for a given field, skipping entities that lack the field.
fn collect_numeric_values(
    field_id: &crate::FieldId,
    entities: &[&Entity],
) -> Result<Vec<NumericValue>, QueryError> {
    let mut values = Vec::new();

    for entity in entities {
        if let Some(field_value) = entity.get_field(field_id) {
            match field_value {
                crate::FieldValue::Integer(i) => {
                    values.push(NumericValue::Integer(*i));
                }
                crate::FieldValue::Float(f) => {
                    values.push(NumericValue::Float(*f));
                }
                crate::FieldValue::Currency { amount, currency } => {
                    values.push(NumericValue::Currency {
                        amount: *amount,
                        currency: *currency,
                    });
                }
                other => {
                    return Err(QueryError::InvalidAggregation {
                        message: format!(
                            "Cannot aggregate non-numeric field '{}'. Found type: {}",
                            field_id.as_str(),
                            other.get_type()
                        ),
                    });
                }
            }
        }
    }

    Ok(values)
}
