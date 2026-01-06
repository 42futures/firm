use firm_core::{Entity, FieldId, FieldValue, ReferenceValue, compose_entity_id};

use super::EntityConversionError;
use crate::parser::{ParsedEntity, ParsedValue};

/// Converts a ParsedEntity to an Entity.
impl TryFrom<&ParsedEntity<'_>> for Entity {
    type Error = EntityConversionError;

    fn try_from(parsed: &ParsedEntity) -> Result<Self, EntityConversionError> {
        let entity_type_str = parsed
            .entity_type()
            .ok_or(EntityConversionError::MissingEntityType)?;

        let entity_id = parsed.id().ok_or(EntityConversionError::MissingEntityId)?;
        let composite_id = compose_entity_id(entity_type_str, entity_id);
        let mut entity = Entity::new(composite_id, entity_type_str.into());

        for field in parsed.fields() {
            let field_id = field.id().ok_or(EntityConversionError::MissingFieldId)?;
            let parsed_value = field
                .value()
                .map_err(|_| EntityConversionError::InvalidFieldValue)?;

            let field_value: FieldValue = parsed_value
                .try_into()
                .map_err(|_| EntityConversionError::InvalidFieldValue)?;

            entity
                .fields
                .push((FieldId(field_id.to_string()), field_value));
        }

        Ok(entity)
    }
}

/// Converts a ParsedValue to a FieldValue.
impl TryFrom<ParsedValue> for FieldValue {
    type Error = EntityConversionError;

    fn try_from(parsed: ParsedValue) -> Result<Self, EntityConversionError> {
        match parsed {
            ParsedValue::Boolean(value) => Ok(FieldValue::Boolean(value)),
            ParsedValue::String(value) => Ok(FieldValue::String(value)),
            ParsedValue::Integer(value) => Ok(FieldValue::Integer(value)),
            ParsedValue::Float(value) => Ok(FieldValue::Float(value)),
            ParsedValue::Currency { amount, currency } => {
                Ok(FieldValue::Currency { amount, currency })
            }
            ParsedValue::EntityReference {
                entity_type,
                entity_id,
            } => {
                let composite_id = compose_entity_id(&entity_type, &entity_id);
                Ok(FieldValue::Reference(ReferenceValue::Entity(composite_id)))
            }
            ParsedValue::FieldReference {
                entity_type,
                entity_id,
                field_id,
            } => {
                let composite_id = compose_entity_id(&entity_type, &entity_id);
                Ok(FieldValue::Reference(ReferenceValue::Field(
                    composite_id,
                    FieldId(field_id),
                )))
            }
            ParsedValue::List(values) => {
                let converted_values: Result<Vec<FieldValue>, EntityConversionError> =
                    values.into_iter().map(|v| v.try_into()).collect();

                Ok(FieldValue::List(converted_values?))
            }
            ParsedValue::DateTime(value) => Ok(FieldValue::DateTime(value)),
            ParsedValue::Path(value) => Ok(FieldValue::Path(value)),
            ParsedValue::Enum(value) => Ok(FieldValue::Enum(value)),
        }
    }
}
