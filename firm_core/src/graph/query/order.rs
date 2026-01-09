//! Entity ordering/sorting logic for queries

use crate::{Entity, FieldValue};
use super::types::SortDirection;
use super::filter::{FieldRef, MetadataField};

/// Compare two entities by a specific field (or metadata) for sorting
pub fn compare_entities_by_field(
    a: &Entity,
    b: &Entity,
    field_ref: &FieldRef,
    direction: &SortDirection,
) -> std::cmp::Ordering {
    use std::cmp::Ordering;

    let ordering = match field_ref {
        FieldRef::Regular(field_id) => {
            let a_value = a.get_field(field_id);
            let b_value = b.get_field(field_id);

            // Handle missing values: sort to end
            match (a_value, b_value) {
                (None, None) => Ordering::Equal,
                (None, Some(_)) => Ordering::Greater, // Missing values sort to end
                (Some(_), None) => Ordering::Less,
                (Some(a_val), Some(b_val)) => compare_field_values(a_val, b_val),
            }
        }
        FieldRef::Metadata(metadata) => {
            match metadata {
                MetadataField::Type => {
                    // Compare entity types (case-insensitive string comparison)
                    a.entity_type.as_str().to_lowercase().cmp(&b.entity_type.as_str().to_lowercase())
                }
                MetadataField::Id => {
                    // Compare entity IDs (case-insensitive string comparison)
                    a.id.as_str().to_lowercase().cmp(&b.id.as_str().to_lowercase())
                }
            }
        }
    };

    // Apply direction
    match direction {
        SortDirection::Ascending => ordering,
        SortDirection::Descending => ordering.reverse(),
    }
}

/// Compare two field values for sorting
fn compare_field_values(a: &FieldValue, b: &FieldValue) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    use FieldValue::*;

    match (a, b) {
        // Same types - natural comparison
        (Boolean(a), Boolean(b)) => a.cmp(b),
        (Integer(a), Integer(b)) => a.cmp(b),
        (Float(a), Float(b)) => {
            // Handle NaN: NaN sorts to end
            if a.is_nan() && b.is_nan() {
                Ordering::Equal
            } else if a.is_nan() {
                Ordering::Greater
            } else if b.is_nan() {
                Ordering::Less
            } else {
                a.partial_cmp(b).unwrap_or(Ordering::Equal)
            }
        }
        (String(a), String(b)) => a.to_lowercase().cmp(&b.to_lowercase()), // Case-insensitive
        (Enum(a), Enum(b)) => a.to_lowercase().cmp(&b.to_lowercase()), // Case-insensitive
        (DateTime(a), DateTime(b)) => a.cmp(b),
        (Currency { amount: a_amt, currency: a_cur }, Currency { amount: b_amt, currency: b_cur }) => {
            // Only compare if same currency
            if a_cur.code() == b_cur.code() {
                a_amt.cmp(b_amt)
            } else {
                // Different currencies: compare by currency code for consistent ordering
                a_cur.code().cmp(b_cur.code())
            }
        }
        (Reference(a), Reference(b)) => {
            a.to_string().to_lowercase().cmp(&b.to_string().to_lowercase())
        }
        (Path(a), Path(b)) => a.cmp(b),

        // Cross-type: Integer vs Float
        (Integer(a), Float(b)) => {
            let a_float = *a as f64;
            if b.is_nan() {
                Ordering::Less
            } else {
                a_float.partial_cmp(b).unwrap_or(Ordering::Equal)
            }
        }
        (Float(a), Integer(b)) => {
            let b_float = *b as f64;
            if a.is_nan() {
                Ordering::Greater
            } else {
                a.partial_cmp(&b_float).unwrap_or(Ordering::Equal)
            }
        }

        // Lists: compare element by element
        (List(a), List(b)) => {
            for (a_item, b_item) in a.iter().zip(b.iter()) {
                match compare_field_values(a_item, b_item) {
                    Ordering::Equal => continue,
                    other => return other,
                }
            }
            // If all elements equal, shorter list comes first
            a.len().cmp(&b.len())
        }

        // Different types: use type precedence for consistent ordering
        // Order: Boolean < Integer/Float < String/Enum/Path < DateTime < Currency < Reference < List
        _ => compare_type_precedence(a, b),
    }
}

/// Define a consistent ordering for different field value types
fn compare_type_precedence(a: &FieldValue, b: &FieldValue) -> std::cmp::Ordering {
    fn type_order(v: &FieldValue) -> u8 {
        match v {
            FieldValue::Boolean(_) => 0,
            FieldValue::Integer(_) | FieldValue::Float(_) => 1,
            FieldValue::String(_) | FieldValue::Enum(_) | FieldValue::Path(_) => 2,
            FieldValue::DateTime(_) => 3,
            FieldValue::Currency { .. } => 4,
            FieldValue::Reference(_) => 5,
            FieldValue::List(_) => 6,
        }
    }
    type_order(a).cmp(&type_order(b))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Entity, EntityId, EntityType, FieldId, FieldValue, ReferenceValue};
    use chrono::{FixedOffset, TimeZone};
    use iso_currency::Currency;
    use rust_decimal::Decimal;
    use std::path::PathBuf;
    use std::str::FromStr;

    fn create_entity(id: &str, field: &str, value: FieldValue) -> Entity {
        Entity::new(EntityId::new(id), EntityType::new("test"))
            .with_field(FieldId::new(field), value)
    }

    fn create_entity_with_type(id: &str, entity_type: &str, field: &str, value: FieldValue) -> Entity {
        Entity::new(EntityId::new(id), EntityType::new(entity_type))
            .with_field(FieldId::new(field), value)
    }

    // Boolean tests
    #[test]
    fn test_order_boolean_ascending() {
        let e1 = create_entity("e1", "flag", FieldValue::Boolean(true));
        let e2 = create_entity("e2", "flag", FieldValue::Boolean(false));

        let result = compare_entities_by_field(&e1, &e2, &FieldRef::Regular(FieldId::new("flag")), &SortDirection::Ascending);
        assert_eq!(result, std::cmp::Ordering::Greater); // true > false
    }

    #[test]
    fn test_order_boolean_descending() {
        let e1 = create_entity("e1", "flag", FieldValue::Boolean(true));
        let e2 = create_entity("e2", "flag", FieldValue::Boolean(false));

        let result = compare_entities_by_field(&e1, &e2, &FieldRef::Regular(FieldId::new("flag")), &SortDirection::Descending);
        assert_eq!(result, std::cmp::Ordering::Less); // reversed
    }

    // Integer tests
    #[test]
    fn test_order_integer_ascending() {
        let e1 = create_entity("e1", "count", FieldValue::Integer(10));
        let e2 = create_entity("e2", "count", FieldValue::Integer(5));

        let result = compare_entities_by_field(&e1, &e2, &FieldRef::Regular(FieldId::new("count")), &SortDirection::Ascending);
        assert_eq!(result, std::cmp::Ordering::Greater);
    }

    #[test]
    fn test_order_integer_descending() {
        let e1 = create_entity("e1", "count", FieldValue::Integer(10));
        let e2 = create_entity("e2", "count", FieldValue::Integer(5));

        let result = compare_entities_by_field(&e1, &e2, &FieldRef::Regular(FieldId::new("count")), &SortDirection::Descending);
        assert_eq!(result, std::cmp::Ordering::Less);
    }

    // Float tests
    #[test]
    fn test_order_float_ascending() {
        let e1 = create_entity("e1", "score", FieldValue::Float(3.14));
        let e2 = create_entity("e2", "score", FieldValue::Float(2.71));

        let result = compare_entities_by_field(&e1, &e2, &FieldRef::Regular(FieldId::new("score")), &SortDirection::Ascending);
        assert_eq!(result, std::cmp::Ordering::Greater);
    }

    #[test]
    fn test_order_float_descending() {
        let e1 = create_entity("e1", "score", FieldValue::Float(3.14));
        let e2 = create_entity("e2", "score", FieldValue::Float(2.71));

        let result = compare_entities_by_field(&e1, &e2, &FieldRef::Regular(FieldId::new("score")), &SortDirection::Descending);
        assert_eq!(result, std::cmp::Ordering::Less);
    }

    #[test]
    fn test_order_float_nan_sorts_to_end() {
        let e1 = create_entity("e1", "score", FieldValue::Float(f64::NAN));
        let e2 = create_entity("e2", "score", FieldValue::Float(3.14));

        let result = compare_entities_by_field(&e1, &e2, &FieldRef::Regular(FieldId::new("score")), &SortDirection::Ascending);
        assert_eq!(result, std::cmp::Ordering::Greater); // NaN sorts after normal values
    }

    // String tests
    #[test]
    fn test_order_string_ascending() {
        let e1 = create_entity("e1", "name", FieldValue::String("Zebra".to_string()));
        let e2 = create_entity("e2", "name", FieldValue::String("Apple".to_string()));

        let result = compare_entities_by_field(&e1, &e2, &FieldRef::Regular(FieldId::new("name")), &SortDirection::Ascending);
        assert_eq!(result, std::cmp::Ordering::Greater);
    }

    #[test]
    fn test_order_string_descending() {
        let e1 = create_entity("e1", "name", FieldValue::String("Zebra".to_string()));
        let e2 = create_entity("e2", "name", FieldValue::String("Apple".to_string()));

        let result = compare_entities_by_field(&e1, &e2, &FieldRef::Regular(FieldId::new("name")), &SortDirection::Descending);
        assert_eq!(result, std::cmp::Ordering::Less);
    }

    #[test]
    fn test_order_string_case_insensitive() {
        let e1 = create_entity("e1", "name", FieldValue::String("zebra".to_string()));
        let e2 = create_entity("e2", "name", FieldValue::String("APPLE".to_string()));

        let result = compare_entities_by_field(&e1, &e2, &FieldRef::Regular(FieldId::new("name")), &SortDirection::Ascending);
        assert_eq!(result, std::cmp::Ordering::Greater); // Case-insensitive comparison
    }

    // Enum tests
    #[test]
    fn test_order_enum_ascending() {
        let e1 = create_entity("e1", "status", FieldValue::Enum("pending".to_string()));
        let e2 = create_entity("e2", "status", FieldValue::Enum("active".to_string()));

        let result = compare_entities_by_field(&e1, &e2, &FieldRef::Regular(FieldId::new("status")), &SortDirection::Ascending);
        assert_eq!(result, std::cmp::Ordering::Greater);
    }

    #[test]
    fn test_order_enum_descending() {
        let e1 = create_entity("e1", "status", FieldValue::Enum("pending".to_string()));
        let e2 = create_entity("e2", "status", FieldValue::Enum("active".to_string()));

        let result = compare_entities_by_field(&e1, &e2, &FieldRef::Regular(FieldId::new("status")), &SortDirection::Descending);
        assert_eq!(result, std::cmp::Ordering::Less);
    }

    // DateTime tests
    #[test]
    fn test_order_datetime_ascending() {
        let dt1 = FixedOffset::east_opt(0).unwrap().with_ymd_and_hms(2024, 6, 1, 10, 0, 0).unwrap();
        let dt2 = FixedOffset::east_opt(0).unwrap().with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap();

        let e1 = create_entity("e1", "date", FieldValue::DateTime(dt1));
        let e2 = create_entity("e2", "date", FieldValue::DateTime(dt2));

        let result = compare_entities_by_field(&e1, &e2, &FieldRef::Regular(FieldId::new("date")), &SortDirection::Ascending);
        assert_eq!(result, std::cmp::Ordering::Greater);
    }

    #[test]
    fn test_order_datetime_descending() {
        let dt1 = FixedOffset::east_opt(0).unwrap().with_ymd_and_hms(2024, 6, 1, 10, 0, 0).unwrap();
        let dt2 = FixedOffset::east_opt(0).unwrap().with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap();

        let e1 = create_entity("e1", "date", FieldValue::DateTime(dt1));
        let e2 = create_entity("e2", "date", FieldValue::DateTime(dt2));

        let result = compare_entities_by_field(&e1, &e2, &FieldRef::Regular(FieldId::new("date")), &SortDirection::Descending);
        assert_eq!(result, std::cmp::Ordering::Less);
    }

    // Currency tests
    #[test]
    fn test_order_currency_same_code_ascending() {
        let e1 = create_entity("e1", "price", FieldValue::Currency {
            amount: Decimal::from_str("100.50").unwrap(),
            currency: Currency::from_code("USD").unwrap(),
        });
        let e2 = create_entity("e2", "price", FieldValue::Currency {
            amount: Decimal::from_str("50.25").unwrap(),
            currency: Currency::from_code("USD").unwrap(),
        });

        let result = compare_entities_by_field(&e1, &e2, &FieldRef::Regular(FieldId::new("price")), &SortDirection::Ascending);
        assert_eq!(result, std::cmp::Ordering::Greater);
    }

    #[test]
    fn test_order_currency_same_code_descending() {
        let e1 = create_entity("e1", "price", FieldValue::Currency {
            amount: Decimal::from_str("100.50").unwrap(),
            currency: Currency::from_code("USD").unwrap(),
        });
        let e2 = create_entity("e2", "price", FieldValue::Currency {
            amount: Decimal::from_str("50.25").unwrap(),
            currency: Currency::from_code("USD").unwrap(),
        });

        let result = compare_entities_by_field(&e1, &e2, &FieldRef::Regular(FieldId::new("price")), &SortDirection::Descending);
        assert_eq!(result, std::cmp::Ordering::Less);
    }

    #[test]
    fn test_order_currency_different_codes() {
        let e1 = create_entity("e1", "price", FieldValue::Currency {
            amount: Decimal::from_str("100.00").unwrap(),
            currency: Currency::from_code("USD").unwrap(),
        });
        let e2 = create_entity("e2", "price", FieldValue::Currency {
            amount: Decimal::from_str("100.00").unwrap(),
            currency: Currency::from_code("EUR").unwrap(),
        });

        let result = compare_entities_by_field(&e1, &e2, &FieldRef::Regular(FieldId::new("price")), &SortDirection::Ascending);
        assert_eq!(result, std::cmp::Ordering::Greater); // USD > EUR alphabetically
    }

    // Reference tests
    #[test]
    fn test_order_reference_ascending() {
        let e1 = create_entity("e1", "ref", FieldValue::Reference(
            ReferenceValue::Entity(EntityId::new("person.zebra"))
        ));
        let e2 = create_entity("e2", "ref", FieldValue::Reference(
            ReferenceValue::Entity(EntityId::new("person.apple"))
        ));

        let result = compare_entities_by_field(&e1, &e2, &FieldRef::Regular(FieldId::new("ref")), &SortDirection::Ascending);
        assert_eq!(result, std::cmp::Ordering::Greater);
    }

    #[test]
    fn test_order_reference_descending() {
        let e1 = create_entity("e1", "ref", FieldValue::Reference(
            ReferenceValue::Entity(EntityId::new("person.zebra"))
        ));
        let e2 = create_entity("e2", "ref", FieldValue::Reference(
            ReferenceValue::Entity(EntityId::new("person.apple"))
        ));

        let result = compare_entities_by_field(&e1, &e2, &FieldRef::Regular(FieldId::new("ref")), &SortDirection::Descending);
        assert_eq!(result, std::cmp::Ordering::Less);
    }

    // Path tests
    #[test]
    fn test_order_path_ascending() {
        let e1 = create_entity("e1", "file", FieldValue::Path(PathBuf::from("/z/file.txt")));
        let e2 = create_entity("e2", "file", FieldValue::Path(PathBuf::from("/a/file.txt")));

        let result = compare_entities_by_field(&e1, &e2, &FieldRef::Regular(FieldId::new("file")), &SortDirection::Ascending);
        assert_eq!(result, std::cmp::Ordering::Greater);
    }

    #[test]
    fn test_order_path_descending() {
        let e1 = create_entity("e1", "file", FieldValue::Path(PathBuf::from("/z/file.txt")));
        let e2 = create_entity("e2", "file", FieldValue::Path(PathBuf::from("/a/file.txt")));

        let result = compare_entities_by_field(&e1, &e2, &FieldRef::Regular(FieldId::new("file")), &SortDirection::Descending);
        assert_eq!(result, std::cmp::Ordering::Less);
    }

    // List tests
    #[test]
    fn test_order_list_ascending() {
        let e1 = create_entity("e1", "tags", FieldValue::List(vec![
            FieldValue::String("b".to_string()),
            FieldValue::String("c".to_string()),
        ]));
        let e2 = create_entity("e2", "tags", FieldValue::List(vec![
            FieldValue::String("a".to_string()),
            FieldValue::String("c".to_string()),
        ]));

        let result = compare_entities_by_field(&e1, &e2, &FieldRef::Regular(FieldId::new("tags")), &SortDirection::Ascending);
        assert_eq!(result, std::cmp::Ordering::Greater); // First element differs
    }

    #[test]
    fn test_order_list_descending() {
        let e1 = create_entity("e1", "tags", FieldValue::List(vec![
            FieldValue::String("b".to_string()),
        ]));
        let e2 = create_entity("e2", "tags", FieldValue::List(vec![
            FieldValue::String("a".to_string()),
        ]));

        let result = compare_entities_by_field(&e1, &e2, &FieldRef::Regular(FieldId::new("tags")), &SortDirection::Descending);
        assert_eq!(result, std::cmp::Ordering::Less);
    }

    // Cross-type: Integer vs Float
    #[test]
    fn test_order_integer_vs_float_ascending() {
        let e1 = create_entity("e1", "value", FieldValue::Integer(42));
        let e2 = create_entity("e2", "value", FieldValue::Float(3.14));

        let result = compare_entities_by_field(&e1, &e2, &FieldRef::Regular(FieldId::new("value")), &SortDirection::Ascending);
        assert_eq!(result, std::cmp::Ordering::Greater); // 42 > 3.14
    }

    #[test]
    fn test_order_float_vs_integer_ascending() {
        let e1 = create_entity("e1", "value", FieldValue::Float(3.14));
        let e2 = create_entity("e2", "value", FieldValue::Integer(42));

        let result = compare_entities_by_field(&e1, &e2, &FieldRef::Regular(FieldId::new("value")), &SortDirection::Ascending);
        assert_eq!(result, std::cmp::Ordering::Less); // 3.14 < 42
    }

    // Type precedence test
    #[test]
    fn test_order_type_precedence() {
        // Boolean < Integer/Float < String < DateTime < Currency < Reference < List
        let e_bool = create_entity("e1", "field", FieldValue::Boolean(true));
        let e_int = create_entity("e2", "field", FieldValue::Integer(42));
        let e_str = create_entity("e3", "field", FieldValue::String("test".to_string()));
        let e_dt = create_entity("e4", "field", FieldValue::DateTime(
            FixedOffset::east_opt(0).unwrap().with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap()
        ));

        // Boolean < Integer
        assert_eq!(
            compare_entities_by_field(&e_bool, &e_int, &FieldRef::Regular(FieldId::new("field")), &SortDirection::Ascending),
            std::cmp::Ordering::Less
        );

        // Integer < String
        assert_eq!(
            compare_entities_by_field(&e_int, &e_str, &FieldRef::Regular(FieldId::new("field")), &SortDirection::Ascending),
            std::cmp::Ordering::Less
        );

        // String < DateTime
        assert_eq!(
            compare_entities_by_field(&e_str, &e_dt, &FieldRef::Regular(FieldId::new("field")), &SortDirection::Ascending),
            std::cmp::Ordering::Less
        );
    }

    // Missing value tests
    #[test]
    fn test_order_missing_value_sorts_to_end() {
        let e1 = Entity::new(EntityId::new("e1"), EntityType::new("test")); // No field
        let e2 = create_entity("e2", "value", FieldValue::Integer(42));

        let result = compare_entities_by_field(&e1, &e2, &FieldRef::Regular(FieldId::new("value")), &SortDirection::Ascending);
        assert_eq!(result, std::cmp::Ordering::Greater); // Missing sorts after present
    }

    #[test]
    fn test_order_both_missing_equal() {
        let e1 = Entity::new(EntityId::new("e1"), EntityType::new("test"));
        let e2 = Entity::new(EntityId::new("e2"), EntityType::new("test"));

        let result = compare_entities_by_field(&e1, &e2, &FieldRef::Regular(FieldId::new("value")), &SortDirection::Ascending);
        assert_eq!(result, std::cmp::Ordering::Equal);
    }

    // Metadata: @type tests
    #[test]
    fn test_order_metadata_type_ascending() {
        let e1 = create_entity_with_type("e1", "task", "name", FieldValue::String("test".to_string()));
        let e2 = create_entity_with_type("e2", "person", "name", FieldValue::String("test".to_string()));

        let result = compare_entities_by_field(&e1, &e2, &FieldRef::Metadata(MetadataField::Type), &SortDirection::Ascending);
        assert_eq!(result, std::cmp::Ordering::Greater); // task > person alphabetically
    }

    #[test]
    fn test_order_metadata_type_descending() {
        let e1 = create_entity_with_type("e1", "task", "name", FieldValue::String("test".to_string()));
        let e2 = create_entity_with_type("e2", "person", "name", FieldValue::String("test".to_string()));

        let result = compare_entities_by_field(&e1, &e2, &FieldRef::Metadata(MetadataField::Type), &SortDirection::Descending);
        assert_eq!(result, std::cmp::Ordering::Less); // reversed
    }

    #[test]
    fn test_order_metadata_type_case_insensitive() {
        let e1 = create_entity_with_type("e1", "TASK", "name", FieldValue::String("test".to_string()));
        let e2 = create_entity_with_type("e2", "person", "name", FieldValue::String("test".to_string()));

        let result = compare_entities_by_field(&e1, &e2, &FieldRef::Metadata(MetadataField::Type), &SortDirection::Ascending);
        assert_eq!(result, std::cmp::Ordering::Greater); // Case-insensitive: task > person
    }

    // Metadata: @id tests
    #[test]
    fn test_order_metadata_id_ascending() {
        let e1 = Entity::new(EntityId::new("zebra"), EntityType::new("test"));
        let e2 = Entity::new(EntityId::new("apple"), EntityType::new("test"));

        let result = compare_entities_by_field(&e1, &e2, &FieldRef::Metadata(MetadataField::Id), &SortDirection::Ascending);
        assert_eq!(result, std::cmp::Ordering::Greater); // zebra > apple
    }

    #[test]
    fn test_order_metadata_id_descending() {
        let e1 = Entity::new(EntityId::new("zebra"), EntityType::new("test"));
        let e2 = Entity::new(EntityId::new("apple"), EntityType::new("test"));

        let result = compare_entities_by_field(&e1, &e2, &FieldRef::Metadata(MetadataField::Id), &SortDirection::Descending);
        assert_eq!(result, std::cmp::Ordering::Less); // reversed
    }

    #[test]
    fn test_order_metadata_id_case_insensitive() {
        let e1 = Entity::new(EntityId::new("ZEBRA"), EntityType::new("test"));
        let e2 = Entity::new(EntityId::new("apple"), EntityType::new("test"));

        let result = compare_entities_by_field(&e1, &e2, &FieldRef::Metadata(MetadataField::Id), &SortDirection::Ascending);
        assert_eq!(result, std::cmp::Ordering::Greater); // Case-insensitive: zebra > apple
    }
}
