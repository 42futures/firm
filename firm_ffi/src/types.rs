//! UniFFI-compatible types that mirror firm_core's domain types.
//!
//! These are thin wrappers that avoid exposing Rust-specific types
//! (Decimal, Currency, DateTime, PathBuf) across the FFI boundary.

use std::path::PathBuf;

use firm_core::{Entity, EntityId, EntityType, FieldId, FieldValue, ReferenceValue};

/// A Firm entity with its type, ID, and fields.
#[derive(Debug, Clone, uniffi::Record)]
pub struct FirmEntity {
    /// Composite ID (e.g. "person.alice").
    pub id: String,
    /// Entity type (e.g. "person").
    pub entity_type: String,
    /// Ordered list of fields.
    pub fields: Vec<FirmField>,
}

/// A named field on an entity.
#[derive(Debug, Clone, uniffi::Record)]
pub struct FirmField {
    pub name: String,
    pub value: FirmFieldValue,
}

/// The value of an entity field.
///
/// Mirrors firm_core's `FieldValue` with FFI-safe types.
/// `ReferenceValue` is flattened into two variants.
/// `Currency` and `DateTime` use string representations.
#[derive(Debug, Clone, uniffi::Enum)]
pub enum FirmFieldValue {
    Boolean { value: bool },
    String { value: String },
    Integer { value: i64 },
    Float { value: f64 },
    /// Currency as decimal string + ISO 4217 code (e.g. "123.45", "USD").
    Currency { amount: String, currency_code: String },
    /// Reference to another entity (e.g. "person.alice").
    EntityReference { entity_id: String },
    /// Reference to a field on another entity.
    FieldReference { entity_id: String, field_id: String },
    /// Ordered list of values.
    List { items: Vec<FirmFieldValue> },
    /// ISO 8601 / RFC 3339 datetime string.
    DateTime { value: String },
    /// File path as string.
    Path { value: String },
    /// Enum variant name.
    Enum { value: String },
}

/// Direction for traversing entity relationships.
#[derive(Debug, Clone, uniffi::Enum)]
pub enum FirmDirection {
    /// Entities this entity references.
    Outgoing,
    /// Entities that reference this entity.
    Incoming,
    /// Both directions.
    Both,
}

/// The result of executing a query.
#[derive(Debug, Clone, uniffi::Enum)]
pub enum FirmQueryResult {
    /// Matching entities.
    Entities { entities: Vec<FirmEntity> },
    /// Aggregation result as a display string (count, sum, average, etc.).
    Aggregation { value: String },
}

// --- Conversions from firm_core types ---

impl From<&Entity> for FirmEntity {
    fn from(entity: &Entity) -> Self {
        FirmEntity {
            id: entity.id.to_string(),
            entity_type: entity.entity_type.to_string(),
            fields: entity
                .fields
                .iter()
                .map(|(id, value)| FirmField {
                    name: id.to_string(),
                    value: FirmFieldValue::from(value),
                })
                .collect(),
        }
    }
}

// --- Conversions to firm_core types ---

impl TryFrom<&FirmEntity> for Entity {
    type Error = String;

    fn try_from(firm: &FirmEntity) -> Result<Self, String> {
        let entity_id = EntityId::new(&firm.id);
        let entity_type = EntityType::new(&firm.entity_type);
        let mut entity = Entity::new(entity_id, entity_type);

        for field in &firm.fields {
            let value = FieldValue::try_from(&field.value)?;
            entity = entity.with_field(FieldId::new(&field.name), value);
        }

        Ok(entity)
    }
}

impl TryFrom<&FirmFieldValue> for FieldValue {
    type Error = String;

    fn try_from(value: &FirmFieldValue) -> Result<Self, String> {
        match value {
            FirmFieldValue::Boolean { value } => Ok(FieldValue::Boolean(*value)),
            FirmFieldValue::String { value } => Ok(FieldValue::String(value.clone())),
            FirmFieldValue::Integer { value } => Ok(FieldValue::Integer(*value)),
            FirmFieldValue::Float { value } => Ok(FieldValue::Float(*value)),
            FirmFieldValue::Currency {
                amount,
                currency_code,
            } => {
                let decimal = rust_decimal::Decimal::from_str_exact(amount)
                    .map_err(|e| format!("Invalid currency amount '{}': {}", amount, e))?;
                let currency = iso_currency::Currency::from_code(currency_code)
                    .ok_or_else(|| format!("Invalid currency code '{}'", currency_code))?;
                Ok(FieldValue::Currency {
                    amount: decimal,
                    currency,
                })
            }
            FirmFieldValue::EntityReference { entity_id } => Ok(FieldValue::Reference(
                ReferenceValue::Entity(EntityId::new(entity_id)),
            )),
            FirmFieldValue::FieldReference {
                entity_id,
                field_id,
            } => Ok(FieldValue::Reference(ReferenceValue::Field(
                EntityId::new(entity_id),
                FieldId::new(field_id),
            ))),
            FirmFieldValue::List { items } => {
                let values: Result<Vec<FieldValue>, String> =
                    items.iter().map(FieldValue::try_from).collect();
                Ok(FieldValue::List(values?))
            }
            FirmFieldValue::DateTime { value } => {
                let dt = chrono::DateTime::parse_from_rfc3339(value)
                    .map_err(|e| format!("Invalid datetime '{}': {}", value, e))?;
                Ok(FieldValue::DateTime(dt))
            }
            FirmFieldValue::Path { value } => Ok(FieldValue::Path(PathBuf::from(value))),
            FirmFieldValue::Enum { value } => Ok(FieldValue::Enum(value.clone())),
        }
    }
}

// --- Conversions from firm_core types ---

impl From<&FieldValue> for FirmFieldValue {
    fn from(value: &FieldValue) -> Self {
        match value {
            FieldValue::Boolean(v) => FirmFieldValue::Boolean { value: *v },
            FieldValue::String(v) => FirmFieldValue::String {
                value: v.clone(),
            },
            FieldValue::Integer(v) => FirmFieldValue::Integer { value: *v },
            FieldValue::Float(v) => FirmFieldValue::Float { value: *v },
            FieldValue::Currency { amount, currency } => FirmFieldValue::Currency {
                amount: amount.to_string(),
                currency_code: currency.code().to_string(),
            },
            FieldValue::Reference(ReferenceValue::Entity(entity_id)) => {
                FirmFieldValue::EntityReference {
                    entity_id: entity_id.to_string(),
                }
            }
            FieldValue::Reference(ReferenceValue::Field(entity_id, field_id)) => {
                FirmFieldValue::FieldReference {
                    entity_id: entity_id.to_string(),
                    field_id: field_id.to_string(),
                }
            }
            FieldValue::List(items) => FirmFieldValue::List {
                items: items.iter().map(FirmFieldValue::from).collect(),
            },
            FieldValue::DateTime(dt) => FirmFieldValue::DateTime {
                value: dt.to_rfc3339(),
            },
            FieldValue::Path(p) => FirmFieldValue::Path {
                value: p.to_string_lossy().into_owned(),
            },
            FieldValue::Enum(v) => FirmFieldValue::Enum {
                value: v.clone(),
            },
        }
    }
}
