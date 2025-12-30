pub mod from_entity;
pub mod from_field;
pub mod from_schema;
pub mod from_value;
pub mod generator_options;

use firm_core::{Entity, EntitySchema};

use from_entity::generate_entity;
use from_schema::generate_schema;
use generator_options::GeneratorOptions;

/// Generates Firm DSL for a collection of entities.
pub fn generate_dsl(entities: &[Entity]) -> String {
    generate_dsl_with_options(entities, &GeneratorOptions::default())
}

/// Generates DSL with formatting options.
pub fn generate_dsl_with_options(entities: &[Entity], options: &GeneratorOptions) -> String {
    let mut output = String::new();

    for (i, entity) in entities.iter().enumerate() {
        if i > 0 && options.blank_lines_between_entities {
            output.push('\n');
        }

        output.push_str(&generate_entity(entity, options));
    }

    output
}

/// Generates Firm DSL for a single schema.
pub fn generate_schema_dsl(schema: &EntitySchema) -> String {
    generate_schema(schema, &GeneratorOptions::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate::generator_options::{GeneratorOptions, IndentStyle};
    use firm_core::{Entity, EntityId, EntityType, FieldId, FieldValue, ReferenceValue};
    use iso_currency::Currency;
    use rust_decimal::Decimal;

    #[test]
    fn test_generate_empty_entities_list() {
        let result = generate_dsl(&[]);
        assert_eq!(result, "");
    }

    #[test]
    fn test_generate_multiple_entities() {
        // Create a person
        let person = Entity {
            id: EntityId("person.daniel_rothmann".to_string()),
            entity_type: EntityType::new("person"),
            fields: [
                (
                    FieldId("first_name".to_string()),
                    FieldValue::String("Daniel".to_string()),
                ),
                (
                    FieldId("last_name".to_string()),
                    FieldValue::String("Rothmann".to_string()),
                ),
                (
                    FieldId("primary_email".to_string()),
                    FieldValue::String("daniel@42futures.com".to_string()),
                ),
            ]
            .into(),
        };

        // Create an organization
        let organization = Entity {
            id: EntityId("organization.main".to_string()),
            entity_type: EntityType::new("organization"),
            fields: [
                (
                    FieldId("name".to_string()),
                    FieldValue::String("42futures".to_string()),
                ),
                (
                    FieldId("primary_email".to_string()),
                    FieldValue::String("hello@42futures.com".to_string()),
                ),
            ]
            .into(),
        };

        // Create a project with references
        let project = Entity {
            id: EntityId("project.firm_language".to_string()),
            entity_type: EntityType::new("project"),
            fields: [
                (
                    FieldId("name".to_string()),
                    FieldValue::String("Firm Language Development".to_string()),
                ),
                (
                    FieldId("owner_ref".to_string()),
                    FieldValue::Reference(ReferenceValue::Entity(EntityId(
                        "person.daniel_rothmann".to_string(),
                    ))),
                ),
                (
                    FieldId("organization_ref".to_string()),
                    FieldValue::Reference(ReferenceValue::Entity(EntityId(
                        "organization.main".to_string(),
                    ))),
                ),
                (
                    FieldId("budget".to_string()),
                    FieldValue::Currency {
                        amount: Decimal::from_str_exact("150000").unwrap(),
                        currency: Currency::EUR,
                    },
                ),
                (
                    FieldId("technologies".to_string()),
                    FieldValue::List(vec![
                        FieldValue::String("Rust".to_string()),
                        FieldValue::String("Tree-sitter".to_string()),
                        FieldValue::String("WASM".to_string()),
                    ]),
                ),
            ]
            .into(),
        };

        let result = generate_dsl(&[person, organization, project]);

        let expected = r#"person daniel_rothmann {
    first_name = "Daniel"
    last_name = "Rothmann"
    primary_email = "daniel@42futures.com"
}

organization main {
    name = "42futures"
    primary_email = "hello@42futures.com"
}

project firm_language {
    name = "Firm Language Development"
    owner_ref = person.daniel_rothmann
    organization_ref = organization.main
    budget = 150000 EUR
    technologies = ["Rust", "Tree-sitter", "WASM"]
}
"#;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_generate_with_custom_options() {
        let entities = vec![
            Entity {
                id: EntityId("person.alice".to_string()),
                entity_type: EntityType::new("person"),
                fields: [(
                    FieldId("name".to_string()),
                    FieldValue::String("Alice".to_string()),
                )]
                .into(),
            },
            Entity {
                id: EntityId("person.bob".to_string()),
                entity_type: EntityType::new("person"),
                fields: [(
                    FieldId("name".to_string()),
                    FieldValue::String("Bob".to_string()),
                )]
                .into(),
            },
        ];

        let options = GeneratorOptions {
            indent_style: IndentStyle::Spaces(2),
            blank_lines_between_entities: false,
            ..Default::default()
        };

        let result = generate_dsl_with_options(&entities, &options);

        let expected = r#"person alice {
  name = "Alice"
}
person bob {
  name = "Bob"
}
"#;
        assert_eq!(result, expected);
    }
}
