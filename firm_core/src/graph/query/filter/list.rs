//! List comparison logic for filters

use crate::FieldValue;
use super::types::{FilterOperator, FilterValue};
use super::{boolean, numeric, string, currency, datetime, reference};

/// Compare a list value against a filter
pub fn compare_list(
    items: &[FieldValue],
    operator: &FilterOperator,
    filter_value: &FilterValue,
) -> bool {
    match operator {
        FilterOperator::Contains => {
            // For "contains" operator, check if any list item matches the filter value
            // For strings, we check if the item contains the substring (not just equality)
            // For other types, we use equality comparison
            items.iter().any(|item| {
                match (item, filter_value) {
                    // String contains: check if any string in the list contains the filter substring
                    (FieldValue::String(s), FilterValue::String(_)) => {
                        string::compare_string(s, &FilterOperator::Contains, filter_value)
                    }
                    // For all other types, use equality
                    _ => compare_list_item(item, &FilterOperator::Equal, filter_value)
                }
            })
        }
        FilterOperator::Equal => {
            // For equality, compare the entire list (exact match)
            match filter_value {
                FilterValue::List(filter_items) => {
                    if items.len() != filter_items.len() {
                        return false;
                    }
                    // Compare each item using equality
                    items.iter().zip(filter_items.iter()).all(|(item, filter_item)| {
                        compare_list_item(item, &FilterOperator::Equal, filter_item)
                    })
                }
                _ => false,
            }
        }
        _ => false, // Other operators not supported for lists yet
    }
}

/// Compare individual list items by delegating to type-specific comparators
/// This leverages all the existing comparison logic we've already tested
fn compare_list_item(item: &FieldValue, operator: &FilterOperator, filter_value: &FilterValue) -> bool {
    match item {
        FieldValue::String(s) => string::compare_string(s, operator, filter_value),
        FieldValue::Integer(i) => numeric::compare_integer(*i, operator, filter_value),
        FieldValue::Float(f) => numeric::compare_float(*f, operator, filter_value),
        FieldValue::Boolean(b) => boolean::compare_boolean(*b, operator, filter_value),
        FieldValue::Currency { amount, currency } => currency::compare_currency(amount, currency, operator, filter_value),
        FieldValue::DateTime(dt) => datetime::compare_datetime(dt, operator, filter_value),
        FieldValue::Reference(r) => reference::compare_reference(r, operator, filter_value),
        FieldValue::Enum(e) => string::compare_string(e, operator, filter_value), // Enums are strings
        FieldValue::Path(p) => {
            // Path is a PathBuf, convert to string for comparison
            if let Some(path_str) = p.to_str() {
                string::compare_string(path_str, operator, filter_value)
            } else {
                false // Invalid UTF-8 in path
            }
        }
        FieldValue::List(_) => false, // Nested lists not supported yet
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ReferenceValue, EntityId};
    use chrono::{FixedOffset, TimeZone};
    use iso_currency::Currency;
    use rust_decimal::Decimal;
    use std::path::PathBuf;
    use std::str::FromStr;

    #[test]
    fn test_list_contains_string() {
        let items = vec![
            FieldValue::String("apple".to_string()),
            FieldValue::String("banana".to_string()),
            FieldValue::String("cherry".to_string()),
        ];

        assert!(compare_list(&items, &FilterOperator::Contains, &FilterValue::String("banana".to_string())));
        assert!(compare_list(&items, &FilterOperator::Contains, &FilterValue::String("APPLE".to_string()))); // Case insensitive
        assert!(!compare_list(&items, &FilterOperator::Contains, &FilterValue::String("grape".to_string())));
    }

    #[test]
    fn test_list_contains_string_substring() {
        let items = vec![
            FieldValue::String("https://www.linkedin.com/in/johndoe/".to_string()),
            FieldValue::String("https://twitter.com/johndoe".to_string()),
        ];

        // Should match substring
        assert!(compare_list(&items, &FilterOperator::Contains, &FilterValue::String("linkedin".to_string())));
        assert!(compare_list(&items, &FilterOperator::Contains, &FilterValue::String("TWITTER".to_string()))); // Case insensitive
        assert!(!compare_list(&items, &FilterOperator::Contains, &FilterValue::String("github".to_string())));
    }

    #[test]
    fn test_list_contains_integer() {
        let items = vec![
            FieldValue::Integer(1),
            FieldValue::Integer(2),
            FieldValue::Integer(3),
        ];

        assert!(compare_list(&items, &FilterOperator::Contains, &FilterValue::Integer(2)));
        assert!(!compare_list(&items, &FilterOperator::Contains, &FilterValue::Integer(5)));
    }

    #[test]
    fn test_list_contains_float() {
        let items = vec![
            FieldValue::Float(1.5),
            FieldValue::Float(2.5),
            FieldValue::Float(3.5),
        ];

        assert!(compare_list(&items, &FilterOperator::Contains, &FilterValue::Float(2.5)));
        assert!(!compare_list(&items, &FilterOperator::Contains, &FilterValue::Float(4.5)));
    }

    #[test]
    fn test_list_contains_boolean() {
        let items = vec![
            FieldValue::Boolean(true),
            FieldValue::Boolean(false),
        ];

        assert!(compare_list(&items, &FilterOperator::Contains, &FilterValue::Boolean(true)));
        assert!(compare_list(&items, &FilterOperator::Contains, &FilterValue::Boolean(false)));
    }

    #[test]
    fn test_list_contains_currency() {
        let items = vec![
            FieldValue::Currency {
                amount: Decimal::from_str("100.50").unwrap(),
                currency: Currency::from_code("USD").unwrap(),
            },
            FieldValue::Currency {
                amount: Decimal::from_str("200.75").unwrap(),
                currency: Currency::from_code("USD").unwrap(),
            },
        ];

        assert!(compare_list(&items, &FilterOperator::Contains, &FilterValue::Currency {
            amount: 100.50,
            code: "USD".to_string(),
        }));

        // Different currency code should not match
        assert!(!compare_list(&items, &FilterOperator::Contains, &FilterValue::Currency {
            amount: 100.50,
            code: "EUR".to_string(),
        }));
    }

    #[test]
    fn test_list_contains_datetime() {
        let dt1 = FixedOffset::east_opt(0).unwrap().with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
        let dt2 = FixedOffset::east_opt(0).unwrap().with_ymd_and_hms(2024, 2, 20, 14, 45, 0).unwrap();

        let items = vec![
            FieldValue::DateTime(dt1),
            FieldValue::DateTime(dt2),
        ];

        assert!(compare_list(&items, &FilterOperator::Contains, &FilterValue::DateTime("2024-01-15T10:30:00+00:00".to_string())));
        assert!(!compare_list(&items, &FilterOperator::Contains, &FilterValue::DateTime("2024-03-01T00:00:00+00:00".to_string())));
    }

    #[test]
    fn test_list_contains_reference() {
        let items = vec![
            FieldValue::Reference(ReferenceValue::Entity(EntityId::new("person.john"))),
            FieldValue::Reference(ReferenceValue::Entity(EntityId::new("person.jane"))),
        ];

        assert!(compare_list(&items, &FilterOperator::Contains, &FilterValue::Reference("person.john".to_string())));
        assert!(compare_list(&items, &FilterOperator::Contains, &FilterValue::Reference("PERSON.JOHN".to_string()))); // Case insensitive
        assert!(!compare_list(&items, &FilterOperator::Contains, &FilterValue::Reference("person.bob".to_string())));
    }

    #[test]
    fn test_list_contains_enum() {
        let items = vec![
            FieldValue::Enum("active".to_string()),
            FieldValue::Enum("pending".to_string()),
            FieldValue::Enum("completed".to_string()),
        ];

        assert!(compare_list(&items, &FilterOperator::Contains, &FilterValue::Enum("pending".to_string())));
        assert!(compare_list(&items, &FilterOperator::Contains, &FilterValue::Enum("ACTIVE".to_string()))); // Case insensitive
        assert!(!compare_list(&items, &FilterOperator::Contains, &FilterValue::Enum("cancelled".to_string())));
    }

    #[test]
    fn test_list_contains_path() {
        let items = vec![
            FieldValue::Path(PathBuf::from("/path/to/file1.txt")),
            FieldValue::Path(PathBuf::from("/path/to/file2.txt")),
        ];

        assert!(compare_list(&items, &FilterOperator::Contains, &FilterValue::Path("/path/to/file1.txt".to_string())));
        assert!(!compare_list(&items, &FilterOperator::Contains, &FilterValue::Path("/path/to/file3.txt".to_string())));
    }

    #[test]
    fn test_list_equal_strings() {
        let items = vec![
            FieldValue::String("apple".to_string()),
            FieldValue::String("banana".to_string()),
        ];

        let filter_items = vec![
            FilterValue::String("apple".to_string()),
            FilterValue::String("banana".to_string()),
        ];

        assert!(compare_list(&items, &FilterOperator::Equal, &FilterValue::List(filter_items)));
    }

    #[test]
    fn test_list_equal_case_insensitive() {
        let items = vec![
            FieldValue::String("apple".to_string()),
            FieldValue::String("banana".to_string()),
        ];

        let filter_items = vec![
            FilterValue::String("APPLE".to_string()),
            FilterValue::String("BANANA".to_string()),
        ];

        assert!(compare_list(&items, &FilterOperator::Equal, &FilterValue::List(filter_items)));
    }

    #[test]
    fn test_list_equal_different_order() {
        let items = vec![
            FieldValue::String("apple".to_string()),
            FieldValue::String("banana".to_string()),
        ];

        let filter_items = vec![
            FilterValue::String("banana".to_string()),
            FilterValue::String("apple".to_string()),
        ];

        // Order matters for equality
        assert!(!compare_list(&items, &FilterOperator::Equal, &FilterValue::List(filter_items)));
    }

    #[test]
    fn test_list_equal_different_length() {
        let items = vec![
            FieldValue::String("apple".to_string()),
            FieldValue::String("banana".to_string()),
        ];

        let filter_items = vec![
            FilterValue::String("apple".to_string()),
        ];

        assert!(!compare_list(&items, &FilterOperator::Equal, &FilterValue::List(filter_items)));
    }

    #[test]
    fn test_list_equal_integers() {
        let items = vec![
            FieldValue::Integer(1),
            FieldValue::Integer(2),
            FieldValue::Integer(3),
        ];

        let filter_items = vec![
            FilterValue::Integer(1),
            FilterValue::Integer(2),
            FilterValue::Integer(3),
        ];

        assert!(compare_list(&items, &FilterOperator::Equal, &FilterValue::List(filter_items)));
    }

    #[test]
    fn test_list_equal_floats() {
        let items = vec![
            FieldValue::Float(1.5),
            FieldValue::Float(2.5),
        ];

        let filter_items = vec![
            FilterValue::Float(1.5),
            FilterValue::Float(2.5),
        ];

        assert!(compare_list(&items, &FilterOperator::Equal, &FilterValue::List(filter_items)));
    }

    #[test]
    fn test_list_equal_booleans() {
        let items = vec![
            FieldValue::Boolean(true),
            FieldValue::Boolean(false),
        ];

        let filter_items = vec![
            FilterValue::Boolean(true),
            FilterValue::Boolean(false),
        ];

        assert!(compare_list(&items, &FilterOperator::Equal, &FilterValue::List(filter_items)));
    }

    #[test]
    fn test_list_equal_currency() {
        let items = vec![
            FieldValue::Currency {
                amount: Decimal::from_str("100.50").unwrap(),
                currency: Currency::from_code("USD").unwrap(),
            },
            FieldValue::Currency {
                amount: Decimal::from_str("200.75").unwrap(),
                currency: Currency::from_code("USD").unwrap(),
            },
        ];

        let filter_items = vec![
            FilterValue::Currency {
                amount: 100.50,
                code: "USD".to_string(),
            },
            FilterValue::Currency {
                amount: 200.75,
                code: "USD".to_string(),
            },
        ];

        assert!(compare_list(&items, &FilterOperator::Equal, &FilterValue::List(filter_items)));
    }

    #[test]
    fn test_list_equal_datetime() {
        let dt1 = FixedOffset::east_opt(0).unwrap().with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
        let dt2 = FixedOffset::east_opt(0).unwrap().with_ymd_and_hms(2024, 2, 20, 14, 45, 0).unwrap();

        let items = vec![
            FieldValue::DateTime(dt1),
            FieldValue::DateTime(dt2),
        ];

        let filter_items = vec![
            FilterValue::DateTime("2024-01-15T10:30:00+00:00".to_string()),
            FilterValue::DateTime("2024-02-20T14:45:00+00:00".to_string()),
        ];

        assert!(compare_list(&items, &FilterOperator::Equal, &FilterValue::List(filter_items)));
    }

    #[test]
    fn test_list_equal_references() {
        let items = vec![
            FieldValue::Reference(ReferenceValue::Entity(EntityId::new("person.john"))),
            FieldValue::Reference(ReferenceValue::Entity(EntityId::new("person.jane"))),
        ];

        let filter_items = vec![
            FilterValue::Reference("person.john".to_string()),
            FilterValue::Reference("person.jane".to_string()),
        ];

        assert!(compare_list(&items, &FilterOperator::Equal, &FilterValue::List(filter_items)));
    }

    #[test]
    fn test_list_equal_mixed_types() {
        let items = vec![
            FieldValue::String("test".to_string()),
            FieldValue::Integer(42),
            FieldValue::Boolean(true),
        ];

        let filter_items = vec![
            FilterValue::String("test".to_string()),
            FilterValue::Integer(42),
            FilterValue::Boolean(true),
        ];

        assert!(compare_list(&items, &FilterOperator::Equal, &FilterValue::List(filter_items)));
    }

    #[test]
    fn test_list_equal_type_mismatch() {
        let items = vec![
            FieldValue::String("42".to_string()),
            FieldValue::Integer(100),
        ];

        let filter_items = vec![
            FilterValue::Integer(42), // Type mismatch: string vs integer
            FilterValue::Integer(100),
        ];

        assert!(!compare_list(&items, &FilterOperator::Equal, &FilterValue::List(filter_items)));
    }

    #[test]
    fn test_list_contains_empty_list() {
        let items: Vec<FieldValue> = vec![];

        assert!(!compare_list(&items, &FilterOperator::Contains, &FilterValue::String("test".to_string())));
    }

    #[test]
    fn test_list_equal_empty_lists() {
        let items: Vec<FieldValue> = vec![];
        let filter_items: Vec<FilterValue> = vec![];

        assert!(compare_list(&items, &FilterOperator::Equal, &FilterValue::List(filter_items)));
    }

    #[test]
    fn test_list_unsupported_operator() {
        let items = vec![
            FieldValue::String("apple".to_string()),
        ];

        // GreaterThan not supported for lists
        assert!(!compare_list(&items, &FilterOperator::GreaterThan, &FilterValue::String("apple".to_string())));
        assert!(!compare_list(&items, &FilterOperator::LessThan, &FilterValue::String("apple".to_string())));
    }

    #[test]
    fn test_list_contains_cross_type_numeric() {
        let items = vec![
            FieldValue::Integer(42),
            FieldValue::Integer(100),
        ];

        // Should work: comparing integer list item against float filter
        assert!(compare_list(&items, &FilterOperator::Contains, &FilterValue::Float(42.0)));
        assert!(!compare_list(&items, &FilterOperator::Contains, &FilterValue::Float(43.0)));
    }

    #[test]
    fn test_list_equal_cross_type_numeric() {
        let items = vec![
            FieldValue::Integer(42),
            FieldValue::Float(3.14),
        ];

        let filter_items = vec![
            FilterValue::Float(42.0), // Cross-type: int vs float
            FilterValue::Float(3.14),
        ];

        assert!(compare_list(&items, &FilterOperator::Equal, &FilterValue::List(filter_items)));
    }

    #[test]
    fn test_list_contains_nested_list_not_supported() {
        let nested = vec![FieldValue::String("inner".to_string())];
        let items = vec![
            FieldValue::List(nested),
        ];

        // Nested lists are not supported
        assert!(!compare_list(&items, &FilterOperator::Contains, &FilterValue::String("inner".to_string())));
    }
}
