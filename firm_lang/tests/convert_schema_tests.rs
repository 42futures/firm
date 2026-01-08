use firm_core::{
    EntityType, FieldId,
    field::FieldType,
    schema::{EntitySchema, FieldMode},
};

use firm_lang::{convert::SchemaConversionError, parser::dsl::parse_source};

#[test]
fn test_convert_simple_schema() {
    let source = r#"
        schema project {
            field {
                name = "title"
                type = "string"
                required = true
            }

            field {
                name = "priority"
                type = "integer"
                required = false
            }
        }
    "#;

    let parsed = parse_source(String::from(source), None).unwrap();
    let schemas = parsed.schemas();
    assert_eq!(schemas.len(), 1);

    let schema: EntitySchema = (&schemas[0]).try_into().unwrap();

    assert_eq!(schema.entity_type, EntityType::new("project"));
    assert_eq!(schema.fields.len(), 2);

    // Check required field
    let title_field = &schema.fields[&FieldId("title".to_string())];
    assert_eq!(title_field.field_type, FieldType::String);
    assert_eq!(title_field.field_mode, FieldMode::Required);

    // Check optional field
    let priority_field = &schema.fields[&FieldId("priority".to_string())];
    assert_eq!(priority_field.field_type, FieldType::Integer);
    assert_eq!(priority_field.field_mode, FieldMode::Optional);
}

#[test]
fn test_convert_schema_with_various_types() {
    let source = r#"
        schema invoice {
            field {
                name = "amount"
                type = "currency"
                required = true
            }

            field {
                name = "paid"
                type = "boolean"
                required = false
            }

            field {
                name = "due_date"
                type = "datetime"
                required = true
            }
        }
    "#;

    let parsed = parse_source(String::from(source), None).unwrap();
    let schemas = parsed.schemas();
    assert_eq!(schemas.len(), 1);

    let schema: EntitySchema = (&schemas[0]).try_into().unwrap();

    assert_eq!(schema.entity_type, EntityType::new("invoice"));
    assert_eq!(schema.fields.len(), 3);

    // Check currency field
    let amount_field = &schema.fields[&FieldId("amount".to_string())];
    assert_eq!(amount_field.field_type, FieldType::Currency);
    assert_eq!(amount_field.field_mode, FieldMode::Required);

    // Check boolean field
    let paid_field = &schema.fields[&FieldId("paid".to_string())];
    assert_eq!(paid_field.field_type, FieldType::Boolean);
    assert_eq!(paid_field.field_mode, FieldMode::Optional);

    // Check datetime field
    let due_date_field = &schema.fields[&FieldId("due_date".to_string())];
    assert_eq!(due_date_field.field_type, FieldType::DateTime);
    assert_eq!(due_date_field.field_mode, FieldMode::Required);
}

#[test]
fn test_unknown_field_type_error() {
    let source = r#"
        schema test {
            field {
                name = "custom"
                type = "unknown_type"
                required = true
            }
        }
    "#;

    let parsed = parse_source(String::from(source), None).unwrap();
    let schemas = parsed.schemas();
    assert_eq!(schemas.len(), 1);

    let result: Result<EntitySchema, SchemaConversionError> = (&schemas[0]).try_into();
    assert!(matches!(
        result,
        Err(SchemaConversionError::UnknownFieldType(_))
    ));
}

#[test]
fn test_convert_multiple_schemas() {
    let source = r#"
        schema project {
            field {
                name = "title"
                type = "string"
                required = true
            }
        }

        schema invoice {
            field {
                name = "amount"
                type = "currency"
                required = true
            }
        }
    "#;

    let parsed = parse_source(String::from(source), None).unwrap();
    let schemas = parsed.schemas();
    assert_eq!(schemas.len(), 2);

    // Convert both schemas
    let project_schema: EntitySchema = (&schemas[0]).try_into().unwrap();
    let invoice_schema: EntitySchema = (&schemas[1]).try_into().unwrap();

    assert_eq!(project_schema.entity_type, EntityType::new("project"));
    assert_eq!(invoice_schema.entity_type, EntityType::new("invoice"));

    assert_eq!(project_schema.fields.len(), 1);
    assert_eq!(invoice_schema.fields.len(), 1);
}

#[test]
fn test_convert_schema_with_enum() {
    let source = r#"
        schema account {
            field {
                name = "name"
                type = "string"
                required = true
            }
            field {
                name = "status"
                type = "enum"
                allowed_values = ["prospect", "customer", "partner"]
                required = true
            }
        }
    "#;

    let parsed = parse_source(String::from(source), None).unwrap();
    let schemas = parsed.schemas();
    assert_eq!(schemas.len(), 1);

    let schema: EntitySchema = (&schemas[0]).try_into().unwrap();

    assert_eq!(schema.entity_type, EntityType::new("account"));
    assert_eq!(schema.fields.len(), 2);

    // Check enum field
    let status_field = &schema.fields[&FieldId("status".to_string())];
    assert_eq!(status_field.field_type, FieldType::Enum);
    assert_eq!(status_field.field_mode, FieldMode::Required);

    // Check allowed values
    assert!(status_field.allowed_values().is_some());
    let allowed = status_field.allowed_values().unwrap();
    assert_eq!(allowed.len(), 3);
    assert!(allowed.contains(&"prospect".to_string()));
    assert!(allowed.contains(&"customer".to_string()));
    assert!(allowed.contains(&"partner".to_string()));
}
