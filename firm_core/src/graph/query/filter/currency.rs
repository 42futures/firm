//! Currency comparison logic for filters

use super::super::QueryError;
use super::types::{FilterOperator, FilterValue};
use crate::FieldValue;
use rust_decimal::Decimal;

const SUPPORTED_OPS: [&str; 6] = ["==", "!=", ">", "<", ">=", "<="];

/// Compare a currency field value against a filter
pub fn compare_currency(
    field_value: &FieldValue,
    operator: &FilterOperator,
    filter_value: &FilterValue,
) -> Result<bool, QueryError> {
    let (amount, currency) = match field_value {
        FieldValue::Currency { amount, currency } => (amount, currency),
        _ => {
            return Err(QueryError::TypeMismatch {
                field_type: field_value.get_type().to_string(),
                filter_type: filter_value.type_name().to_string(),
            })
        }
    };

    match filter_value {
        FilterValue::Currency {
            amount: filter_amount,
            code: filter_code,
        } => {
            // Currency code must match (use code() to get ISO code like "EUR", not full name)
            let currency_code = currency.code();
            if currency_code != filter_code.as_str() {
                return Ok(false);
            }

            // Convert filter amount to Decimal for comparison
            let filter_decimal = Decimal::from_f64_retain(*filter_amount);

            if let Some(filter_dec) = filter_decimal {
                // Then compare amounts
                match operator {
                    FilterOperator::Equal => Ok(amount == &filter_dec),
                    FilterOperator::NotEqual => Ok(amount != &filter_dec),
                    FilterOperator::GreaterThan => Ok(amount > &filter_dec),
                    FilterOperator::LessThan => Ok(amount < &filter_dec),
                    FilterOperator::GreaterOrEqual => Ok(amount >= &filter_dec),
                    FilterOperator::LessOrEqual => Ok(amount <= &filter_dec),
                    _ => Err(QueryError::UnsupportedOperator {
                        field_type: field_value.get_type().to_string(),
                        operator: format!("{:?}", operator),
                        supported: SUPPORTED_OPS.iter().map(|s| s.to_string()).collect(),
                    }),
                }
            } else {
                Ok(false)
            }
        }
        _ => Err(QueryError::TypeMismatch {
            field_type: field_value.get_type().to_string(),
            filter_type: filter_value.type_name().to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iso_currency::Currency;

    fn make_currency_field(amount_cents: i64, currency: Currency) -> FieldValue {
        FieldValue::Currency {
            amount: Decimal::new(amount_cents, 2),
            currency,
        }
    }

    #[test]
    fn test_equal_same_currency() {
        let field = make_currency_field(10050, Currency::EUR); // 100.50
        assert!(compare_currency(
            &field,
            &FilterOperator::Equal,
            &FilterValue::Currency {
                amount: 100.50,
                code: "EUR".to_string(),
            }
        )
        .unwrap());
    }

    #[test]
    fn test_not_equal_different_amounts() {
        let field = make_currency_field(10050, Currency::EUR); // 100.50
        assert!(!compare_currency(
            &field,
            &FilterOperator::Equal,
            &FilterValue::Currency {
                amount: 200.00,
                code: "EUR".to_string(),
            }
        )
        .unwrap());
    }

    #[test]
    fn test_equal_different_currency_codes() {
        let field = make_currency_field(10050, Currency::EUR); // 100.50
        // Same amount but different currency should not match
        assert!(!compare_currency(
            &field,
            &FilterOperator::Equal,
            &FilterValue::Currency {
                amount: 100.50,
                code: "USD".to_string(),
            }
        )
        .unwrap());
    }

    #[test]
    fn test_greater_than() {
        let field = make_currency_field(15000, Currency::USD); // 150.00
        assert!(compare_currency(
            &field,
            &FilterOperator::GreaterThan,
            &FilterValue::Currency {
                amount: 100.00,
                code: "USD".to_string(),
            }
        )
        .unwrap());
    }

    #[test]
    fn test_less_than() {
        let field = make_currency_field(5000, Currency::GBP); // 50.00
        assert!(compare_currency(
            &field,
            &FilterOperator::LessThan,
            &FilterValue::Currency {
                amount: 100.00,
                code: "GBP".to_string(),
            }
        )
        .unwrap());
    }

    #[test]
    fn test_greater_or_equal_greater() {
        let field = make_currency_field(15000, Currency::EUR); // 150.00
        assert!(compare_currency(
            &field,
            &FilterOperator::GreaterOrEqual,
            &FilterValue::Currency {
                amount: 100.00,
                code: "EUR".to_string(),
            }
        )
        .unwrap());
    }

    #[test]
    fn test_greater_or_equal_equal() {
        let field = make_currency_field(10000, Currency::EUR); // 100.00
        assert!(compare_currency(
            &field,
            &FilterOperator::GreaterOrEqual,
            &FilterValue::Currency {
                amount: 100.00,
                code: "EUR".to_string(),
            }
        )
        .unwrap());
    }

    #[test]
    fn test_less_or_equal_less() {
        let field = make_currency_field(5000, Currency::USD); // 50.00
        assert!(compare_currency(
            &field,
            &FilterOperator::LessOrEqual,
            &FilterValue::Currency {
                amount: 100.00,
                code: "USD".to_string(),
            }
        )
        .unwrap());
    }

    #[test]
    fn test_less_or_equal_equal() {
        let field = make_currency_field(10000, Currency::USD); // 100.00
        assert!(compare_currency(
            &field,
            &FilterOperator::LessOrEqual,
            &FilterValue::Currency {
                amount: 100.00,
                code: "USD".to_string(),
            }
        )
        .unwrap());
    }

    #[test]
    fn test_not_equal_operator() {
        let field = make_currency_field(10050, Currency::EUR); // 100.50
        assert!(compare_currency(
            &field,
            &FilterOperator::NotEqual,
            &FilterValue::Currency {
                amount: 200.00,
                code: "EUR".to_string(),
            }
        )
        .unwrap());
    }

    #[test]
    fn test_not_equal_same_amount_returns_false() {
        let field = make_currency_field(10050, Currency::EUR); // 100.50
        assert!(!compare_currency(
            &field,
            &FilterOperator::NotEqual,
            &FilterValue::Currency {
                amount: 100.50,
                code: "EUR".to_string(),
            }
        )
        .unwrap());
    }

    #[test]
    fn test_mismatched_currency_with_greater_than() {
        let field = make_currency_field(15000, Currency::EUR); // 150.00 EUR
        // Even though 150 > 100, different currencies should not compare
        assert!(!compare_currency(
            &field,
            &FilterOperator::GreaterThan,
            &FilterValue::Currency {
                amount: 100.00,
                code: "USD".to_string(),
            }
        )
        .unwrap());
    }

    #[test]
    fn test_unsupported_operator_contains() {
        let field = make_currency_field(10050, Currency::EUR);
        let result = compare_currency(
            &field,
            &FilterOperator::Contains,
            &FilterValue::Currency {
                amount: 100.50,
                code: "EUR".to_string(),
            },
        );
        assert!(matches!(result, Err(QueryError::UnsupportedOperator { .. })));
    }

    #[test]
    fn test_wrong_filter_type() {
        let field = make_currency_field(10050, Currency::EUR);
        let result = compare_currency(&field, &FilterOperator::Equal, &FilterValue::Float(100.50));
        assert!(matches!(result, Err(QueryError::TypeMismatch { .. })));
    }

    #[test]
    fn test_zero_amount() {
        let field = make_currency_field(0, Currency::EUR); // 0.00
        assert!(compare_currency(
            &field,
            &FilterOperator::Equal,
            &FilterValue::Currency {
                amount: 0.00,
                code: "EUR".to_string(),
            }
        )
        .unwrap());
    }

    #[test]
    fn test_negative_amount() {
        let field = make_currency_field(-10050, Currency::EUR); // -100.50
        assert!(compare_currency(
            &field,
            &FilterOperator::LessThan,
            &FilterValue::Currency {
                amount: 0.00,
                code: "EUR".to_string(),
            }
        )
        .unwrap());
    }
}
