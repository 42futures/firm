//! List comparison logic for filters

use super::super::QueryError;
use super::types::{FilterOperator, FilterValue};
use super::{boolean, currency, datetime, numeric, reference, string};
use crate::FieldValue;

/// Compare a list field value against a filter
pub fn compare_list(
    field_value: &FieldValue,
    operator: &FilterOperator,
    filter_value: &FilterValue,
) -> Result<bool, QueryError> {
    let items = match field_value {
        FieldValue::List(items) => items,
        _ => {
            return Err(QueryError::TypeMismatch {
                field_type: field_value.get_type().to_string(),
                filter_type: filter_value.type_name().to_string(),
            })
        }
    };

    match operator {
        FilterOperator::Contains => {
            // For "contains" operator, check if any list item matches the filter value
            // For strings, we check if the item contains the substring (not just equality)
            // For other types, we use equality comparison
            for item in items {
                let matches = match (item, filter_value) {
                    // String contains: check if any string in the list contains the filter substring
                    (FieldValue::String(_), FilterValue::String(_)) => {
                        string::compare_string(item, &FilterOperator::Contains, filter_value)?
                    }
                    // For all other types, use equality
                    _ => compare_list_item(item, &FilterOperator::Equal, filter_value)?,
                };
                if matches {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        FilterOperator::Equal => {
            // For equality, compare the entire list (exact match)
            match filter_value {
                FilterValue::List(filter_items) => {
                    if items.len() != filter_items.len() {
                        return Ok(false);
                    }
                    // Compare each item using equality
                    for (item, filter_item) in items.iter().zip(filter_items.iter()) {
                        if !compare_list_item(item, &FilterOperator::Equal, filter_item)? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                }
                _ => Err(QueryError::TypeMismatch {
                    field_type: field_value.get_type().to_string(),
                    filter_type: filter_value.type_name().to_string(),
                }),
            }
        }
        _ => Err(QueryError::UnsupportedOperator {
            field_type: field_value.get_type().to_string(),
            operator: format!("{:?}", operator),
            supported: vec!["contains".to_string(), "==".to_string()],
        }),
    }
}

/// Compare individual list items by delegating to type-specific comparators
/// This leverages all the existing comparison logic we've already tested
fn compare_list_item(
    item: &FieldValue,
    operator: &FilterOperator,
    filter_value: &FilterValue,
) -> Result<bool, QueryError> {
    match item {
        FieldValue::String(_) | FieldValue::Enum(_) | FieldValue::Path(_) => {
            string::compare_string(item, operator, filter_value)
        }
        FieldValue::Integer(_) => numeric::compare_integer(item, operator, filter_value),
        FieldValue::Float(_) => numeric::compare_float(item, operator, filter_value),
        FieldValue::Boolean(_) => boolean::compare_boolean(item, operator, filter_value),
        FieldValue::Currency { .. } => currency::compare_currency(item, operator, filter_value),
        FieldValue::DateTime(_) => datetime::compare_datetime(item, operator, filter_value),
        FieldValue::Reference(_) => reference::compare_reference(item, operator, filter_value),
        FieldValue::List(_) => Ok(false), // Nested lists not supported yet
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EntityId, ReferenceValue};
    use chrono::{FixedOffset, TimeZone};
    use iso_currency::Currency;
    use rust_decimal::Decimal;
    use std::path::PathBuf;
    use std::str::FromStr;

    fn list_field(items: Vec<FieldValue>) -> FieldValue {
        FieldValue::List(items)
    }

    #[test]
    fn test_list_contains_string() {
        let field = list_field(vec![
            FieldValue::String("apple".to_string()),
            FieldValue::String("banana".to_string()),
            FieldValue::String("cherry".to_string()),
        ]);

        assert!(compare_list(
            &field,
            &FilterOperator::Contains,
            &FilterValue::String("banana".to_string())
        ).unwrap());
        assert!(compare_list(
            &field,
            &FilterOperator::Contains,
            &FilterValue::String("APPLE".to_string())
        ).unwrap()); // Case insensitive
        assert!(!compare_list(
            &field,
            &FilterOperator::Contains,
            &FilterValue::String("grape".to_string())
        ).unwrap());
    }

    #[test]
    fn test_list_contains_string_substring() {
        let field = list_field(vec![
            FieldValue::String("https://www.linkedin.com/in/johndoe/".to_string()),
            FieldValue::String("https://twitter.com/johndoe".to_string()),
        ]);

        // Should match substring
        assert!(compare_list(
            &field,
            &FilterOperator::Contains,
            &FilterValue::String("linkedin".to_string())
        ).unwrap());
        assert!(compare_list(
            &field,
            &FilterOperator::Contains,
            &FilterValue::String("TWITTER".to_string())
        ).unwrap()); // Case insensitive
        assert!(!compare_list(
            &field,
            &FilterOperator::Contains,
            &FilterValue::String("github".to_string())
        ).unwrap());
    }

    #[test]
    fn test_list_contains_integer() {
        let field = list_field(vec![
            FieldValue::Integer(1),
            FieldValue::Integer(2),
            FieldValue::Integer(3),
        ]);

        assert!(compare_list(
            &field,
            &FilterOperator::Contains,
            &FilterValue::Integer(2)
        ).unwrap());
        assert!(!compare_list(
            &field,
            &FilterOperator::Contains,
            &FilterValue::Integer(5)
        ).unwrap());
    }

    #[test]
    fn test_list_contains_float() {
        let field = list_field(vec![
            FieldValue::Float(1.5),
            FieldValue::Float(2.5),
            FieldValue::Float(3.5),
        ]);

        assert!(compare_list(
            &field,
            &FilterOperator::Contains,
            &FilterValue::Float(2.5)
        ).unwrap());
        assert!(!compare_list(
            &field,
            &FilterOperator::Contains,
            &FilterValue::Float(4.5)
        ).unwrap());
    }

    #[test]
    fn test_list_contains_boolean() {
        let field = list_field(vec![FieldValue::Boolean(true), FieldValue::Boolean(false)]);

        assert!(compare_list(
            &field,
            &FilterOperator::Contains,
            &FilterValue::Boolean(true)
        ).unwrap());
        assert!(compare_list(
            &field,
            &FilterOperator::Contains,
            &FilterValue::Boolean(false)
        ).unwrap());
    }

    #[test]
    fn test_list_contains_currency() {
        let field = list_field(vec![
            FieldValue::Currency {
                amount: Decimal::from_str("100.50").unwrap(),
                currency: Currency::from_code("USD").unwrap(),
            },
            FieldValue::Currency {
                amount: Decimal::from_str("200.75").unwrap(),
                currency: Currency::from_code("USD").unwrap(),
            },
        ]);

        assert!(compare_list(
            &field,
            &FilterOperator::Contains,
            &FilterValue::Currency {
                amount: 100.50,
                code: "USD".to_string(),
            }
        ).unwrap());

        // Different currency code should not match
        assert!(!compare_list(
            &field,
            &FilterOperator::Contains,
            &FilterValue::Currency {
                amount: 100.50,
                code: "EUR".to_string(),
            }
        ).unwrap());
    }

    #[test]
    fn test_list_contains_datetime() {
        let dt1 = FixedOffset::east_opt(0)
            .unwrap()
            .with_ymd_and_hms(2024, 1, 15, 10, 30, 0)
            .unwrap();
        let dt2 = FixedOffset::east_opt(0)
            .unwrap()
            .with_ymd_and_hms(2024, 2, 20, 14, 45, 0)
            .unwrap();

        let field = list_field(vec![FieldValue::DateTime(dt1), FieldValue::DateTime(dt2)]);

        assert!(compare_list(
            &field,
            &FilterOperator::Contains,
            &FilterValue::DateTime("2024-01-15T10:30:00+00:00".to_string())
        ).unwrap());
        assert!(!compare_list(
            &field,
            &FilterOperator::Contains,
            &FilterValue::DateTime("2024-03-01T00:00:00+00:00".to_string())
        ).unwrap());
    }

    #[test]
    fn test_list_contains_reference() {
        let field = list_field(vec![
            FieldValue::Reference(ReferenceValue::Entity(EntityId::new("person.john"))),
            FieldValue::Reference(ReferenceValue::Entity(EntityId::new("person.jane"))),
        ]);

        assert!(compare_list(
            &field,
            &FilterOperator::Contains,
            &FilterValue::Reference("person.john".to_string())
        ).unwrap());
        assert!(compare_list(
            &field,
            &FilterOperator::Contains,
            &FilterValue::Reference("PERSON.JOHN".to_string())
        ).unwrap()); // Case insensitive
        assert!(!compare_list(
            &field,
            &FilterOperator::Contains,
            &FilterValue::Reference("person.bob".to_string())
        ).unwrap());
    }

    #[test]
    fn test_list_contains_enum() {
        let field = list_field(vec![
            FieldValue::Enum("active".to_string()),
            FieldValue::Enum("pending".to_string()),
            FieldValue::Enum("completed".to_string()),
        ]);

        assert!(compare_list(
            &field,
            &FilterOperator::Contains,
            &FilterValue::Enum("pending".to_string())
        ).unwrap());
        assert!(compare_list(
            &field,
            &FilterOperator::Contains,
            &FilterValue::Enum("ACTIVE".to_string())
        ).unwrap()); // Case insensitive
        assert!(!compare_list(
            &field,
            &FilterOperator::Contains,
            &FilterValue::Enum("cancelled".to_string())
        ).unwrap());
    }

    #[test]
    fn test_list_contains_path() {
        let field = list_field(vec![
            FieldValue::Path(PathBuf::from("/path/to/file1.txt")),
            FieldValue::Path(PathBuf::from("/path/to/file2.txt")),
        ]);

        assert!(compare_list(
            &field,
            &FilterOperator::Contains,
            &FilterValue::Path("/path/to/file1.txt".to_string())
        ).unwrap());
        assert!(!compare_list(
            &field,
            &FilterOperator::Contains,
            &FilterValue::Path("/path/to/file3.txt".to_string())
        ).unwrap());
    }

    #[test]
    fn test_list_equal_strings() {
        let field = list_field(vec![
            FieldValue::String("apple".to_string()),
            FieldValue::String("banana".to_string()),
        ]);

        let filter_items = vec![
            FilterValue::String("apple".to_string()),
            FilterValue::String("banana".to_string()),
        ];

        assert!(compare_list(
            &field,
            &FilterOperator::Equal,
            &FilterValue::List(filter_items)
        ).unwrap());
    }

    #[test]
    fn test_list_equal_case_insensitive() {
        let field = list_field(vec![
            FieldValue::String("apple".to_string()),
            FieldValue::String("banana".to_string()),
        ]);

        let filter_items = vec![
            FilterValue::String("APPLE".to_string()),
            FilterValue::String("BANANA".to_string()),
        ];

        assert!(compare_list(
            &field,
            &FilterOperator::Equal,
            &FilterValue::List(filter_items)
        ).unwrap());
    }

    #[test]
    fn test_list_equal_different_order() {
        let field = list_field(vec![
            FieldValue::String("apple".to_string()),
            FieldValue::String("banana".to_string()),
        ]);

        let filter_items = vec![
            FilterValue::String("banana".to_string()),
            FilterValue::String("apple".to_string()),
        ];

        // Order matters for equality
        assert!(!compare_list(
            &field,
            &FilterOperator::Equal,
            &FilterValue::List(filter_items)
        ).unwrap());
    }

    #[test]
    fn test_list_equal_different_length() {
        let field = list_field(vec![
            FieldValue::String("apple".to_string()),
            FieldValue::String("banana".to_string()),
        ]);

        let filter_items = vec![FilterValue::String("apple".to_string())];

        assert!(!compare_list(
            &field,
            &FilterOperator::Equal,
            &FilterValue::List(filter_items)
        ).unwrap());
    }

    #[test]
    fn test_list_equal_integers() {
        let field = list_field(vec![
            FieldValue::Integer(1),
            FieldValue::Integer(2),
            FieldValue::Integer(3),
        ]);

        let filter_items = vec![
            FilterValue::Integer(1),
            FilterValue::Integer(2),
            FilterValue::Integer(3),
        ];

        assert!(compare_list(
            &field,
            &FilterOperator::Equal,
            &FilterValue::List(filter_items)
        ).unwrap());
    }

    #[test]
    fn test_list_equal_floats() {
        let field = list_field(vec![FieldValue::Float(1.5), FieldValue::Float(2.5)]);

        let filter_items = vec![FilterValue::Float(1.5), FilterValue::Float(2.5)];

        assert!(compare_list(
            &field,
            &FilterOperator::Equal,
            &FilterValue::List(filter_items)
        ).unwrap());
    }

    #[test]
    fn test_list_equal_booleans() {
        let field = list_field(vec![FieldValue::Boolean(true), FieldValue::Boolean(false)]);

        let filter_items = vec![FilterValue::Boolean(true), FilterValue::Boolean(false)];

        assert!(compare_list(
            &field,
            &FilterOperator::Equal,
            &FilterValue::List(filter_items)
        ).unwrap());
    }

    #[test]
    fn test_list_equal_currency() {
        let field = list_field(vec![
            FieldValue::Currency {
                amount: Decimal::from_str("100.50").unwrap(),
                currency: Currency::from_code("USD").unwrap(),
            },
            FieldValue::Currency {
                amount: Decimal::from_str("200.75").unwrap(),
                currency: Currency::from_code("USD").unwrap(),
            },
        ]);

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

        assert!(compare_list(
            &field,
            &FilterOperator::Equal,
            &FilterValue::List(filter_items)
        ).unwrap());
    }

    #[test]
    fn test_list_equal_datetime() {
        let dt1 = FixedOffset::east_opt(0)
            .unwrap()
            .with_ymd_and_hms(2024, 1, 15, 10, 30, 0)
            .unwrap();
        let dt2 = FixedOffset::east_opt(0)
            .unwrap()
            .with_ymd_and_hms(2024, 2, 20, 14, 45, 0)
            .unwrap();

        let field = list_field(vec![FieldValue::DateTime(dt1), FieldValue::DateTime(dt2)]);

        let filter_items = vec![
            FilterValue::DateTime("2024-01-15T10:30:00+00:00".to_string()),
            FilterValue::DateTime("2024-02-20T14:45:00+00:00".to_string()),
        ];

        assert!(compare_list(
            &field,
            &FilterOperator::Equal,
            &FilterValue::List(filter_items)
        ).unwrap());
    }

    #[test]
    fn test_list_equal_references() {
        let field = list_field(vec![
            FieldValue::Reference(ReferenceValue::Entity(EntityId::new("person.john"))),
            FieldValue::Reference(ReferenceValue::Entity(EntityId::new("person.jane"))),
        ]);

        let filter_items = vec![
            FilterValue::Reference("person.john".to_string()),
            FilterValue::Reference("person.jane".to_string()),
        ];

        assert!(compare_list(
            &field,
            &FilterOperator::Equal,
            &FilterValue::List(filter_items)
        ).unwrap());
    }

    #[test]
    fn test_list_equal_mixed_types() {
        let field = list_field(vec![
            FieldValue::String("test".to_string()),
            FieldValue::Integer(42),
            FieldValue::Boolean(true),
        ]);

        let filter_items = vec![
            FilterValue::String("test".to_string()),
            FilterValue::Integer(42),
            FilterValue::Boolean(true),
        ];

        assert!(compare_list(
            &field,
            &FilterOperator::Equal,
            &FilterValue::List(filter_items)
        ).unwrap());
    }

    #[test]
    fn test_list_equal_type_mismatch() {
        let field = list_field(vec![
            FieldValue::String("42".to_string()),
            FieldValue::Integer(100),
        ]);

        let filter_items = vec![
            FilterValue::Integer(42), // Type mismatch: string vs integer
            FilterValue::Integer(100),
        ];

        // Type mismatch in list item comparison returns an error
        let result = compare_list(
            &field,
            &FilterOperator::Equal,
            &FilterValue::List(filter_items)
        );
        assert!(matches!(result, Err(QueryError::TypeMismatch { .. })));
    }

    #[test]
    fn test_list_contains_empty_list() {
        let field = list_field(vec![]);

        assert!(!compare_list(
            &field,
            &FilterOperator::Contains,
            &FilterValue::String("test".to_string())
        ).unwrap());
    }

    #[test]
    fn test_list_equal_empty_lists() {
        let field = list_field(vec![]);
        let filter_items: Vec<FilterValue> = vec![];

        assert!(compare_list(
            &field,
            &FilterOperator::Equal,
            &FilterValue::List(filter_items)
        ).unwrap());
    }

    #[test]
    fn test_list_unsupported_operator() {
        let field = list_field(vec![FieldValue::String("apple".to_string())]);

        // GreaterThan not supported for lists
        let result = compare_list(
            &field,
            &FilterOperator::GreaterThan,
            &FilterValue::String("apple".to_string())
        );
        assert!(matches!(result, Err(QueryError::UnsupportedOperator { .. })));

        let result = compare_list(
            &field,
            &FilterOperator::LessThan,
            &FilterValue::String("apple".to_string())
        );
        assert!(matches!(result, Err(QueryError::UnsupportedOperator { .. })));
    }

    #[test]
    fn test_list_contains_cross_type_numeric() {
        let field = list_field(vec![FieldValue::Integer(42), FieldValue::Integer(100)]);

        // Should work: comparing integer list item against float filter
        assert!(compare_list(
            &field,
            &FilterOperator::Contains,
            &FilterValue::Float(42.0)
        ).unwrap());
        assert!(!compare_list(
            &field,
            &FilterOperator::Contains,
            &FilterValue::Float(43.0)
        ).unwrap());
    }

    #[test]
    fn test_list_equal_cross_type_numeric() {
        let field = list_field(vec![FieldValue::Integer(42), FieldValue::Float(3.14)]);

        let filter_items = vec![
            FilterValue::Float(42.0), // Cross-type: int vs float
            FilterValue::Float(3.14),
        ];

        assert!(compare_list(
            &field,
            &FilterOperator::Equal,
            &FilterValue::List(filter_items)
        ).unwrap());
    }

    #[test]
    fn test_list_contains_nested_list_not_supported() {
        let nested = vec![FieldValue::String("inner".to_string())];
        let field = list_field(vec![FieldValue::List(nested)]);

        // Nested lists are not supported
        assert!(!compare_list(
            &field,
            &FilterOperator::Contains,
            &FilterValue::String("inner".to_string())
        ).unwrap());
    }
}
