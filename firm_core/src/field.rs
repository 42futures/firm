use std::fmt;

use chrono::{DateTime, FixedOffset};
use iso_currency::Currency;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::{EntityId, FieldId};

/// The supported types of an entity field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FieldType {
    Boolean,
    String,
    Integer,
    Float,
    Currency,
    Reference,
    List,
    DateTime,
    Path,
    Enum,
}

impl fmt::Display for FieldType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FieldType::Boolean => write!(f, "Boolean"),
            FieldType::String => write!(f, "String"),
            FieldType::Integer => write!(f, "Integer"),
            FieldType::Float => write!(f, "Float"),
            FieldType::Currency => write!(f, "Currency"),
            FieldType::Reference => write!(f, "Reference"),
            FieldType::List => write!(f, "List"),
            FieldType::DateTime => write!(f, "DateTime"),
            FieldType::Path => write!(f, "Path"),
            FieldType::Enum => write!(f, "Enum"),
        }
    }
}

/// The supported reference types: to an entity or an entity field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ReferenceValue {
    Entity(EntityId),
    Field(EntityId, FieldId),
}

impl fmt::Display for ReferenceValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReferenceValue::Entity(entity_id) => write!(f, "{}", entity_id),
            ReferenceValue::Field(entity_id, field_id) => write!(f, "{}.{}", entity_id, field_id),
        }
    }
}

/// The value of an entity field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FieldValue {
    Boolean(bool),
    String(String),
    Integer(i64),
    Float(f64),
    Currency { amount: Decimal, currency: Currency },
    Reference(ReferenceValue),
    List(Vec<FieldValue>),
    DateTime(DateTime<FixedOffset>),
    Path(PathBuf),
    Enum(String),
}

impl fmt::Display for FieldValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FieldValue::Boolean(val) => write!(f, "{}", val),
            FieldValue::String(val) => write!(f, "{}", val),
            FieldValue::Integer(val) => write!(f, "{}", val),
            FieldValue::Float(val) => write!(f, "{}", val),
            FieldValue::Currency { amount, currency } => write!(f, "{} {}", amount, currency),
            FieldValue::Reference(val) => write!(f, "{}", val),
            FieldValue::List(vals) => {
                write!(
                    f,
                    "[{}]",
                    vals.iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<String>>()
                        .join(",")
                )
            }
            FieldValue::DateTime(val) => write!(f, "{}", val),
            FieldValue::Path(val) => write!(f, "{}", val.display()),
            FieldValue::Enum(val) => write!(f, "{}", val),
        }
    }
}

impl FieldValue {
    /// Gets the type of the given field value.
    pub fn get_type(&self) -> FieldType {
        match self {
            FieldValue::Boolean(_) => FieldType::Boolean,
            FieldValue::String(_) => FieldType::String,
            FieldValue::Integer(_) => FieldType::Integer,
            FieldValue::Float(_) => FieldType::Float,
            FieldValue::Currency {
                amount: _,
                currency: _,
            } => FieldType::Currency,
            FieldValue::Reference(ReferenceValue::Entity(_)) => FieldType::Reference,
            FieldValue::Reference(ReferenceValue::Field(_, _)) => FieldType::Reference,
            FieldValue::List(_) => FieldType::List,
            FieldValue::DateTime(_) => FieldType::DateTime,
            FieldValue::Path(_) => FieldType::Path,
            FieldValue::Enum(_) => FieldType::Enum,
        }
    }

    /// Checks if the field value has the expected type.
    pub fn is_type(&self, expected: &FieldType) -> bool {
        &self.get_type() == expected
    }
}

/// Convert from bool to FieldValue.
impl From<bool> for FieldValue {
    fn from(value: bool) -> Self {
        FieldValue::Boolean(value)
    }
}

/// Convert from &str to FieldValue.
impl From<&str> for FieldValue {
    fn from(value: &str) -> Self {
        FieldValue::String(value.to_string())
    }
}

/// Convert from String to FieldValue.
impl From<String> for FieldValue {
    fn from(value: String) -> Self {
        FieldValue::String(value)
    }
}

/// Convert from i64 to FieldValue.
impl From<i64> for FieldValue {
    fn from(value: i64) -> Self {
        FieldValue::Integer(value)
    }
}

/// Convert from f64 to FieldValue.
impl From<f64> for FieldValue {
    fn from(value: f64) -> Self {
        FieldValue::Float(value)
    }
}

/// Convert from DateTime<FixedOffset> to FieldValue.
impl From<DateTime<FixedOffset>> for FieldValue {
    fn from(value: DateTime<FixedOffset>) -> Self {
        FieldValue::DateTime(value)
    }
}

/// Convert from Vec<FieldValue> to FieldValue.
impl From<Vec<FieldValue>> for FieldValue {
    fn from(value: Vec<FieldValue>) -> Self {
        FieldValue::List(value)
    }
}

/// Convert from PathBuf to FieldValue.
impl From<PathBuf> for FieldValue {
    fn from(value: PathBuf) -> Self {
        FieldValue::Path(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::current_dir;

    #[test]
    fn test_field_value_get_type() {
        let string_value = FieldValue::String("test".to_string());
        assert_eq!(string_value.get_type(), FieldType::String);
    }

    #[test]
    fn test_field_value_is_type() {
        let string_value = FieldValue::String("test".to_string());
        assert!(string_value.is_type(&FieldType::String));
    }

    #[test]
    fn test_field_from_bool() {
        let field: FieldValue = true.into();
        assert_eq!(field, FieldValue::Boolean(true));
    }

    #[test]
    fn test_field_from_str() {
        let field: FieldValue = "Test".into();
        assert_eq!(field, FieldValue::String("Test".to_string()));
    }

    #[test]
    fn test_field_from_string() {
        let field: FieldValue = String::from("Test").into();
        assert_eq!(field, FieldValue::String(String::from("Test")));
    }

    #[test]
    fn test_field_from_i64() {
        let field: FieldValue = 42i64.into();
        assert_eq!(field, FieldValue::Integer(42));
    }

    #[test]
    fn test_field_from_f64() {
        let field: FieldValue = 3.14f64.into();
        assert_eq!(field, FieldValue::Float(3.14));
    }

    #[test]
    fn test_field_from_datetime() {
        use chrono::{FixedOffset, TimeZone};
        let offset = FixedOffset::east_opt(5 * 3600).unwrap();
        let dt = offset.with_ymd_and_hms(2023, 1, 1, 12, 0, 0).unwrap();
        let field: FieldValue = dt.into();
        assert_eq!(field, FieldValue::DateTime(dt));
    }

    #[test]
    fn test_field_from_vec() {
        let values = vec![
            FieldValue::String("test1".to_string()),
            FieldValue::String("test2".to_string()),
        ];
        let field: FieldValue = values.clone().into();
        assert_eq!(field, FieldValue::List(values));
    }

    #[test]
    fn test_field_from_pathbuf() {
        let field: FieldValue = current_dir().unwrap().into();
        assert_eq!(field, FieldValue::Path(current_dir().unwrap()));
    }

    #[test]
    fn test_currency_field_value() {
        use iso_currency::Currency;
        use rust_decimal::Decimal;

        let currency_field = FieldValue::Currency {
            amount: Decimal::new(12345, 2), // $123.45
            currency: Currency::USD,
        };
        assert_eq!(currency_field.get_type(), FieldType::Currency);
        assert!(currency_field.is_type(&FieldType::Currency));
    }

    #[test]
    fn test_entity_reference_field_value() {
        let entity_ref =
            FieldValue::Reference(ReferenceValue::Entity(EntityId::new("test_entity")));
        assert_eq!(entity_ref.get_type(), FieldType::Reference);
        assert!(entity_ref.is_type(&FieldType::Reference));
    }

    #[test]
    fn test_field_reference_field_value() {
        let field_ref = FieldValue::Reference(ReferenceValue::Field(
            EntityId::new("test_entity"),
            FieldId::new("test_field"),
        ));
        assert_eq!(field_ref.get_type(), FieldType::Reference);
        assert!(field_ref.is_type(&FieldType::Reference));
    }

    #[test]
    fn test_list_field_value() {
        let list_field = FieldValue::List(vec![
            FieldValue::String("item1".to_string()),
            FieldValue::String("item2".to_string()),
        ]);
        assert_eq!(list_field.get_type(), FieldType::List);
        assert!(list_field.is_type(&FieldType::List));
    }

    #[test]
    fn test_boolean_serialization() {
        let field = FieldValue::Boolean(true);
        let serialized = serde_json::to_string(&field).unwrap();
        let deserialized: FieldValue = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, field);
    }

    #[test]
    fn test_string_serialization() {
        let field = FieldValue::String("test string".to_string());
        let serialized = serde_json::to_string(&field).unwrap();
        let deserialized: FieldValue = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, field);
    }

    #[test]
    fn test_integer_serialization() {
        let field = FieldValue::Integer(42);
        let serialized = serde_json::to_string(&field).unwrap();
        let deserialized: FieldValue = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, field);
    }

    #[test]
    fn test_float_serialization() {
        let field = FieldValue::Float(3.14);
        let serialized = serde_json::to_string(&field).unwrap();
        let deserialized: FieldValue = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, field);
    }

    #[test]
    fn test_currency_serialization() {
        use iso_currency::Currency;
        use rust_decimal::Decimal;

        let field = FieldValue::Currency {
            amount: Decimal::new(12345, 2),
            currency: Currency::USD,
        };
        let serialized = serde_json::to_string(&field).unwrap();
        let deserialized: FieldValue = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, field);
    }

    #[test]
    fn test_entity_reference_serialization() {
        let field = FieldValue::Reference(ReferenceValue::Entity(EntityId::new("entity1")));
        let serialized = serde_json::to_string(&field).unwrap();
        let deserialized: FieldValue = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, field);
    }

    #[test]
    fn test_field_reference_serialization() {
        let field = FieldValue::Reference(ReferenceValue::Field(
            EntityId::new("entity1"),
            FieldId::new("field1"),
        ));
        let serialized = serde_json::to_string(&field).unwrap();
        let deserialized: FieldValue = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, field);
    }

    #[test]
    fn test_datetime_serialization() {
        use chrono::{FixedOffset, TimeZone};

        let offset = FixedOffset::east_opt(5 * 3600).unwrap();
        let dt = offset.with_ymd_and_hms(2023, 1, 1, 12, 0, 0).unwrap();
        let field = FieldValue::DateTime(dt);
        let serialized = serde_json::to_string(&field).unwrap();
        let deserialized: FieldValue = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, field);
    }

    #[test]
    fn test_string_list_serialization() {
        let field = FieldValue::List(vec![
            FieldValue::String("item1".to_string()),
            FieldValue::String("item2".to_string()),
        ]);
        let serialized = serde_json::to_string(&field).unwrap();
        let deserialized: FieldValue = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, field);
    }

    #[test]
    fn test_integer_list_serialization() {
        let field = FieldValue::List(vec![
            FieldValue::Integer(1),
            FieldValue::Integer(2),
            FieldValue::Integer(3),
        ]);
        let serialized = serde_json::to_string(&field).unwrap();
        let deserialized: FieldValue = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, field);
    }

    #[test]
    fn test_nested_string_list_serialization() {
        let nested_list = FieldValue::List(vec![
            FieldValue::List(vec![
                FieldValue::String("item1".to_string()),
                FieldValue::String("item2".to_string()),
            ]),
            FieldValue::List(vec![
                FieldValue::String("item3".to_string()),
                FieldValue::String("item4".to_string()),
            ]),
        ]);

        let serialized = serde_json::to_string(&nested_list).unwrap();
        let deserialized: FieldValue = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, nested_list);
    }

    #[test]
    fn test_path_serialization() {
        let field = FieldValue::Path(current_dir().unwrap());
        let serialized = serde_json::to_string(&field).unwrap();
        let deserialized: FieldValue = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, field);
    }

    #[test]
    fn test_enum_field_value() {
        let enum_field = FieldValue::Enum("active".to_string());
        assert_eq!(enum_field.get_type(), FieldType::Enum);
        assert!(enum_field.is_type(&FieldType::Enum));
    }

    #[test]
    fn test_enum_serialization() {
        let field = FieldValue::Enum("customer".to_string());
        let serialized = serde_json::to_string(&field).unwrap();
        let deserialized: FieldValue = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, field);
    }
}
