//! Currency comparison logic for filters

use super::types::{FilterOperator, FilterValue};
use iso_currency::Currency;
use rust_decimal::Decimal;

/// Compare a currency value against a filter
pub fn compare_currency(
    amount: &Decimal,
    currency: &Currency,
    operator: &FilterOperator,
    filter_value: &FilterValue,
) -> bool {
    match filter_value {
        FilterValue::Currency {
            amount: filter_amount,
            code: filter_code,
        } => {
            // Currency code must match (use code() to get ISO code like "EUR", not full name)
            let currency_code = currency.code();
            if currency_code != filter_code.as_str() {
                return false;
            }

            // Convert filter amount to Decimal for comparison
            let filter_decimal = Decimal::from_f64_retain(*filter_amount);

            if let Some(filter_dec) = filter_decimal {
                // Then compare amounts
                match operator {
                    FilterOperator::Equal => amount == &filter_dec,
                    FilterOperator::NotEqual => amount != &filter_dec,
                    FilterOperator::GreaterThan => amount > &filter_dec,
                    FilterOperator::LessThan => amount < &filter_dec,
                    FilterOperator::GreaterOrEqual => amount >= &filter_dec,
                    FilterOperator::LessOrEqual => amount <= &filter_dec,
                    _ => false,
                }
            } else {
                false
            }
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equal_same_currency() {
        let amount = Decimal::new(10050, 2); // 100.50
        let currency = Currency::EUR;
        assert!(compare_currency(
            &amount,
            &currency,
            &FilterOperator::Equal,
            &FilterValue::Currency {
                amount: 100.50,
                code: "EUR".to_string(),
            }
        ));
    }

    #[test]
    fn test_not_equal_different_amounts() {
        let amount = Decimal::new(10050, 2); // 100.50
        let currency = Currency::EUR;
        assert!(!compare_currency(
            &amount,
            &currency,
            &FilterOperator::Equal,
            &FilterValue::Currency {
                amount: 200.00,
                code: "EUR".to_string(),
            }
        ));
    }

    #[test]
    fn test_equal_different_currency_codes() {
        let amount = Decimal::new(10050, 2); // 100.50
        let currency = Currency::EUR;
        // Same amount but different currency should not match
        assert!(!compare_currency(
            &amount,
            &currency,
            &FilterOperator::Equal,
            &FilterValue::Currency {
                amount: 100.50,
                code: "USD".to_string(),
            }
        ));
    }

    #[test]
    fn test_greater_than() {
        let amount = Decimal::new(15000, 2); // 150.00
        let currency = Currency::USD;
        assert!(compare_currency(
            &amount,
            &currency,
            &FilterOperator::GreaterThan,
            &FilterValue::Currency {
                amount: 100.00,
                code: "USD".to_string(),
            }
        ));
    }

    #[test]
    fn test_less_than() {
        let amount = Decimal::new(5000, 2); // 50.00
        let currency = Currency::GBP;
        assert!(compare_currency(
            &amount,
            &currency,
            &FilterOperator::LessThan,
            &FilterValue::Currency {
                amount: 100.00,
                code: "GBP".to_string(),
            }
        ));
    }

    #[test]
    fn test_greater_or_equal_greater() {
        let amount = Decimal::new(15000, 2); // 150.00
        let currency = Currency::EUR;
        assert!(compare_currency(
            &amount,
            &currency,
            &FilterOperator::GreaterOrEqual,
            &FilterValue::Currency {
                amount: 100.00,
                code: "EUR".to_string(),
            }
        ));
    }

    #[test]
    fn test_greater_or_equal_equal() {
        let amount = Decimal::new(10000, 2); // 100.00
        let currency = Currency::EUR;
        assert!(compare_currency(
            &amount,
            &currency,
            &FilterOperator::GreaterOrEqual,
            &FilterValue::Currency {
                amount: 100.00,
                code: "EUR".to_string(),
            }
        ));
    }

    #[test]
    fn test_less_or_equal_less() {
        let amount = Decimal::new(5000, 2); // 50.00
        let currency = Currency::USD;
        assert!(compare_currency(
            &amount,
            &currency,
            &FilterOperator::LessOrEqual,
            &FilterValue::Currency {
                amount: 100.00,
                code: "USD".to_string(),
            }
        ));
    }

    #[test]
    fn test_less_or_equal_equal() {
        let amount = Decimal::new(10000, 2); // 100.00
        let currency = Currency::USD;
        assert!(compare_currency(
            &amount,
            &currency,
            &FilterOperator::LessOrEqual,
            &FilterValue::Currency {
                amount: 100.00,
                code: "USD".to_string(),
            }
        ));
    }

    #[test]
    fn test_not_equal_operator() {
        let amount = Decimal::new(10050, 2); // 100.50
        let currency = Currency::EUR;
        assert!(compare_currency(
            &amount,
            &currency,
            &FilterOperator::NotEqual,
            &FilterValue::Currency {
                amount: 200.00,
                code: "EUR".to_string(),
            }
        ));
    }

    #[test]
    fn test_not_equal_same_amount_returns_false() {
        let amount = Decimal::new(10050, 2); // 100.50
        let currency = Currency::EUR;
        assert!(!compare_currency(
            &amount,
            &currency,
            &FilterOperator::NotEqual,
            &FilterValue::Currency {
                amount: 100.50,
                code: "EUR".to_string(),
            }
        ));
    }

    #[test]
    fn test_mismatched_currency_with_greater_than() {
        let amount = Decimal::new(15000, 2); // 150.00 EUR
        let currency = Currency::EUR;
        // Even though 150 > 100, different currencies should not compare
        assert!(!compare_currency(
            &amount,
            &currency,
            &FilterOperator::GreaterThan,
            &FilterValue::Currency {
                amount: 100.00,
                code: "USD".to_string(),
            }
        ));
    }

    #[test]
    fn test_unsupported_operator_contains() {
        let amount = Decimal::new(10050, 2);
        let currency = Currency::EUR;
        assert!(!compare_currency(
            &amount,
            &currency,
            &FilterOperator::Contains,
            &FilterValue::Currency {
                amount: 100.50,
                code: "EUR".to_string(),
            }
        ));
    }

    #[test]
    fn test_wrong_filter_type() {
        let amount = Decimal::new(10050, 2);
        let currency = Currency::EUR;
        assert!(!compare_currency(
            &amount,
            &currency,
            &FilterOperator::Equal,
            &FilterValue::Float(100.50),
        ));
    }

    #[test]
    fn test_zero_amount() {
        let amount = Decimal::new(0, 2); // 0.00
        let currency = Currency::EUR;
        assert!(compare_currency(
            &amount,
            &currency,
            &FilterOperator::Equal,
            &FilterValue::Currency {
                amount: 0.00,
                code: "EUR".to_string(),
            }
        ));
    }

    #[test]
    fn test_negative_amount() {
        let amount = Decimal::new(-10050, 2); // -100.50
        let currency = Currency::EUR;
        assert!(compare_currency(
            &amount,
            &currency,
            &FilterOperator::LessThan,
            &FilterValue::Currency {
                amount: 0.00,
                code: "EUR".to_string(),
            }
        ));
    }
}
