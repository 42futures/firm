use firm_core::{
    EntityType, FieldId,
    field::FieldType,
    schema::{EntitySchema, FieldMode, FieldSchema},
};

use super::SchemaConversionError;
use crate::parser::dsl::ParsedSchema;

/// Converts a ParsedSchema to an EntitySchema.
impl TryFrom<&ParsedSchema<'_>> for EntitySchema {
    type Error = SchemaConversionError;

    fn try_from(parsed: &ParsedSchema) -> Result<Self, SchemaConversionError> {
        let schema_name = parsed
            .name()
            .ok_or(SchemaConversionError::MissingSchemaName)?;

        let entity_type = EntityType::new(schema_name.to_string());
        let mut schema = EntitySchema::new(entity_type);

        for (order, field) in parsed.fields().iter().enumerate() {
            let field_name = field
                .name()
                .map_err(|_| SchemaConversionError::MissingFieldName)?;

            let field_type_str = field
                .field_type()
                .map_err(|_| SchemaConversionError::MissingFieldType)?;

            let field_type = convert_field_type(&field_type_str)?;

            let field_mode = if field.required() {
                FieldMode::Required
            } else {
                FieldMode::Optional
            };

            let field_schema = if field_type == FieldType::Enum {
                // For enum fields, check if allowed values are provided
                if let Some(allowed_values) = field.allowed_values() {
                    FieldSchema::new_enum(field_mode, order, allowed_values)
                } else {
                    // Enum without allowed values - treat as regular field
                    FieldSchema::new(field_type, field_mode, order)
                }
            } else {
                FieldSchema::new(field_type, field_mode, order)
            };

            schema.fields.insert(FieldId(field_name), field_schema);
        }

        Ok(schema)
    }
}

/// Converts a field type string to a FieldType enum.
fn convert_field_type(type_str: &str) -> Result<FieldType, SchemaConversionError> {
    match type_str {
        "boolean" => Ok(FieldType::Boolean),
        "string" => Ok(FieldType::String),
        "integer" => Ok(FieldType::Integer),
        "float" => Ok(FieldType::Float),
        "currency" => Ok(FieldType::Currency),
        "reference" => Ok(FieldType::Reference),
        "list" => Ok(FieldType::List),
        "datetime" => Ok(FieldType::DateTime),
        "path" => Ok(FieldType::Path),
        "enum" => Ok(FieldType::Enum),
        _ => Err(SchemaConversionError::UnknownFieldType(
            type_str.to_string(),
        )),
    }
}
