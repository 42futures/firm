use firm_core::{EntitySchema, FieldType};

use super::GeneratorOptions;

/// Generate DSL for a single schema.
pub fn generate_schema(schema: &EntitySchema, options: &GeneratorOptions) -> String {
    let mut output = String::new();

    // Schema declaration and open block
    output.push_str(&format!("schema {} {{\n", schema.entity_type));

    // Generate fields in order
    for (field_id, field_schema) in schema.ordered_fields() {
        output.push_str(&format!("{}field {{\n", options.indent_style.indent_string(1)));
        output.push_str(&format!(
            "{}name = \"{}\"\n",
            options.indent_style.indent_string(2),
            field_id
        ));
        output.push_str(&format!(
            "{}type = \"{}\"\n",
            options.indent_style.indent_string(2),
            field_type_to_string(&field_schema.field_type)
        ));

        // For enum fields, include the allowed values
        if let Some(allowed_values) = field_schema.allowed_values() {
            let values_str = allowed_values
                .iter()
                .map(|v| format!("\"{}\"", v))
                .collect::<Vec<_>>()
                .join(", ");
            output.push_str(&format!(
                "{}allowed_values = [{}]\n",
                options.indent_style.indent_string(2),
                values_str
            ));
        }

        output.push_str(&format!(
            "{}required = {}\n",
            options.indent_style.indent_string(2),
            field_schema.is_required()
        ));
        output.push_str(&format!("{}}}\n", options.indent_style.indent_string(1)));
    }

    // Close schema block
    output.push_str("}\n");

    output
}

/// Convert FieldType to string representation for DSL.
fn field_type_to_string(field_type: &FieldType) -> &str {
    match field_type {
        FieldType::String => "string",
        FieldType::Integer => "integer",
        FieldType::Float => "float",
        FieldType::Boolean => "boolean",
        FieldType::Currency => "currency",
        FieldType::DateTime => "datetime",
        FieldType::Reference => "reference",
        FieldType::List => "list",
        FieldType::Path => "path",
        FieldType::Enum => "enum",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use firm_core::{EntityType, FieldId};

    #[test]
    fn test_generate_simple_schema() {
        let schema = EntitySchema::new(EntityType::new("person"))
            .with_required_field(FieldId::new("name"), FieldType::String)
            .with_optional_field(FieldId::new("email"), FieldType::String);

        let result = generate_schema(&schema, &GeneratorOptions::default());

        let expected = r#"schema person {
    field {
        name = "name"
        type = "string"
        required = true
    }
    field {
        name = "email"
        type = "string"
        required = false
    }
}
"#;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_generate_schema_with_various_types() {
        let schema = EntitySchema::new(EntityType::new("project"))
            .with_required_field(FieldId::new("name"), FieldType::String)
            .with_required_field(FieldId::new("status"), FieldType::String)
            .with_optional_field(FieldId::new("budget"), FieldType::Currency)
            .with_optional_field(FieldId::new("is_active"), FieldType::Boolean)
            .with_optional_field(FieldId::new("owner_ref"), FieldType::Reference);

        let result = generate_schema(&schema, &GeneratorOptions::default());

        let expected = r#"schema project {
    field {
        name = "name"
        type = "string"
        required = true
    }
    field {
        name = "status"
        type = "string"
        required = true
    }
    field {
        name = "budget"
        type = "currency"
        required = false
    }
    field {
        name = "is_active"
        type = "boolean"
        required = false
    }
    field {
        name = "owner_ref"
        type = "reference"
        required = false
    }
}
"#;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_generate_schema_with_enum() {
        let schema = EntitySchema::new(EntityType::new("account"))
            .with_required_field(FieldId::new("name"), FieldType::String)
            .with_required_enum(
                FieldId::new("status"),
                vec![
                    "prospect".to_string(),
                    "customer".to_string(),
                    "partner".to_string(),
                ],
            );

        let result = generate_schema(&schema, &GeneratorOptions::default());

        let expected = r#"schema account {
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
        assert_eq!(result, expected);
    }
}
