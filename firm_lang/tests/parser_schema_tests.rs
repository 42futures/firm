#[cfg(test)]
mod tests {
    use firm_lang::parser::dsl::parse_source;

    #[test]
    fn test_basic_schema_parsing() {
        let source = r#"
            schema project {
                field {
                    name = "title"
                    type = "string"
                    required = true
                }

                field {
                    name = "description"
                    type = "string"
                    required = false
                }
            }
        "#;

        let parsed = parse_source(String::from(source), None).unwrap();
        assert!(!parsed.has_error());

        let schemas = parsed.schemas();
        assert_eq!(schemas.len(), 1);

        let schema = &schemas[0];
        assert_eq!(schema.name(), Some("project"));

        let fields = schema.fields();
        assert_eq!(fields.len(), 2);

        // Test first field (required)
        let title_field = &fields[0];
        assert_eq!(title_field.name().unwrap(), "title");
        assert_eq!(title_field.field_type().unwrap(), "string");
        assert_eq!(title_field.required(), true);

        // Test second field (optional)
        let desc_field = &fields[1];
        assert_eq!(desc_field.name().unwrap(), "description");
        assert_eq!(desc_field.field_type().unwrap(), "string");
        assert_eq!(desc_field.required(), false);
    }

    #[test]
    fn test_multiple_schemas() {
        let source = r#"
            schema user {
                field {
                    name = "username"
                    type = "string"
                    required = true
                }

                field {
                    name = "email"
                    type = "string"
                    required = true
                }
            }

            schema project {
                field {
                    name = "title"
                    type = "string"
                    required = true
                }

                field {
                    name = "budget"
                    type = "currency"
                }
            }

            schema task {
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

                field {
                    name = "completed"
                    type = "boolean"
                }
            }
        "#;

        let parsed = parse_source(String::from(source), None).unwrap();
        assert!(!parsed.has_error());

        let schemas = parsed.schemas();
        assert_eq!(schemas.len(), 3);

        // Test schema names
        let schema_names: Vec<_> = schemas.iter().map(|s| s.name().unwrap()).collect();
        assert!(schema_names.contains(&"user"));
        assert!(schema_names.contains(&"project"));
        assert!(schema_names.contains(&"task"));

        // Test user schema
        let user_schema = schemas.iter().find(|s| s.name() == Some("user")).unwrap();
        let user_fields = user_schema.fields();
        assert_eq!(user_fields.len(), 2);
        assert!(user_fields.iter().all(|f| f.required()));

        // Test project schema (has optional field)
        let project_schema = schemas
            .iter()
            .find(|s| s.name() == Some("project"))
            .unwrap();
        let project_fields = project_schema.fields();
        assert_eq!(project_fields.len(), 2);
        assert_eq!(project_fields[0].required(), true);
        assert_eq!(project_fields[1].required(), false); // Budget field defaults to false

        // Test task schema (mixed required/optional)
        let task_schema = schemas.iter().find(|s| s.name() == Some("task")).unwrap();
        let task_fields = task_schema.fields();
        assert_eq!(task_fields.len(), 3);
        assert_eq!(task_fields[0].required(), true); // title
        assert_eq!(task_fields[1].required(), false); // priority
        assert_eq!(task_fields[2].required(), false); // completed (no required field)
    }

    #[test]
    fn test_schema_field_types() {
        let source = r#"
            schema data_types {
                field {
                    name = "text_field"
                    type = "string"
                    required = true
                }

                field {
                    name = "number_field"
                    type = "integer"
                    required = true
                }

                field {
                    name = "decimal_field"
                    type = "float"
                    required = false
                }

                field {
                    name = "money_field"
                    type = "currency"
                    required = false
                }

                field {
                    name = "flag_field"
                    type = "boolean"
                    required = false
                }

                field {
                    name = "date_field"
                    type = "date"
                    required = false
                }

                field {
                    name = "datetime_field"
                    type = "datetime"
                    required = false
                }

                field {
                    name = "reference_field"
                    type = "reference"
                    required = false
                }

                field {
                    name = "list_field"
                    type = "list"
                    required = false
                }
            }
        "#;

        let parsed = parse_source(String::from(source), None).unwrap();
        assert!(!parsed.has_error());

        let schemas = parsed.schemas();
        assert_eq!(schemas.len(), 1);

        let schema = &schemas[0];
        let fields = schema.fields();
        assert_eq!(fields.len(), 9);

        // Test all field types are parsed correctly
        let expected_types = vec![
            "string",
            "integer",
            "float",
            "currency",
            "boolean",
            "date",
            "datetime",
            "reference",
            "list",
        ];

        for (i, expected_type) in expected_types.iter().enumerate() {
            assert_eq!(fields[i].field_type().unwrap(), *expected_type);
        }

        // Test required fields
        assert_eq!(fields[0].required(), true); // text_field
        assert_eq!(fields[1].required(), true); // number_field
        for i in 2..9 {
            assert_eq!(fields[i].required(), false); // All others are optional
        }
    }

    #[test]
    fn test_schema_field_without_required() {
        let source = r#"
            schema simple {
                field {
                    name = "title"
                    type = "string"
                }

                field {
                    name = "description"
                    type = "string"
                }
            }
        "#;

        let parsed = parse_source(String::from(source), None).unwrap();
        assert!(!parsed.has_error());

        let schemas = parsed.schemas();
        assert_eq!(schemas.len(), 1);

        let fields = schemas[0].fields();
        assert_eq!(fields.len(), 2);

        // Both fields should default to required = false
        assert_eq!(fields[0].required(), false);
        assert_eq!(fields[1].required(), false);
    }

    #[test]
    fn test_empty_schema() {
        let source = r#"
            schema empty {
            }
        "#;

        let parsed = parse_source(String::from(source), None).unwrap();
        assert!(!parsed.has_error());

        let schemas = parsed.schemas();
        assert_eq!(schemas.len(), 1);

        let schema = &schemas[0];
        assert_eq!(schema.name(), Some("empty"));
        assert_eq!(schema.fields().len(), 0);
    }

    #[test]
    fn test_schema_with_comments() {
        let source = r#"
            // User profile schema
            schema user_profile {
                // Basic identification
                field {
                    name = "username"
                    type = "string"
                    required = true
                }

                /* Contact information
                   for user communication */
                field {
                    name = "email"
                    type = "string"
                    required = true
                }

                // Optional bio field
                field {
                    name = "bio"
                    type = "string"
                    // Not required by default
                }
            }
        "#;

        let parsed = parse_source(String::from(source), None).unwrap();
        assert!(!parsed.has_error());

        let schemas = parsed.schemas();
        assert_eq!(schemas.len(), 1);

        let schema = &schemas[0];
        assert_eq!(schema.name(), Some("user_profile"));

        let fields = schema.fields();
        assert_eq!(fields.len(), 3);

        assert_eq!(fields[0].name().unwrap(), "username");
        assert_eq!(fields[0].required(), true);

        assert_eq!(fields[1].name().unwrap(), "email");
        assert_eq!(fields[1].required(), true);

        assert_eq!(fields[2].name().unwrap(), "bio");
        assert_eq!(fields[2].required(), false);
    }

    #[test]
    fn test_schema_error_invalid_field_structure() {
        let source = r#"
            schema broken {
                field {
                    name = "title"
                    // Missing type field - this should cause parsing errors
                    required = true
                }
            }
        "#;

        let parsed = parse_source(String::from(source), None).unwrap();
        assert!(!parsed.has_error()); // Tree-sitter parses successfully

        let schemas = parsed.schemas();
        assert_eq!(schemas.len(), 1);

        let fields = schemas[0].fields();
        assert_eq!(fields.len(), 1);

        // But accessing the type should fail
        let field = &fields[0];
        assert_eq!(field.name().unwrap(), "title");
        assert!(field.field_type().is_err()); // Should error due to missing type
        assert_eq!(field.required(), true);
    }

    #[test]
    fn test_schema_complex_field_names() {
        let source = r#"
            schema complex_names {
                field {
                    name = "user_id"
                    type = "string"
                    required = true
                }

                field {
                    name = "created_at"
                    type = "datetime"
                    required = true
                }

                field {
                    name = "is_active"
                    type = "boolean"
                    required = false
                }

                field {
                    name = "metadata_json"
                    type = "string"
                    required = false
                }
            }
        "#;

        let parsed = parse_source(String::from(source), None).unwrap();
        assert!(!parsed.has_error());

        let schemas = parsed.schemas();
        assert_eq!(schemas.len(), 1);

        let fields = schemas[0].fields();
        assert_eq!(fields.len(), 4);

        let field_names: Vec<_> = fields.iter().map(|f| f.name().unwrap()).collect();
        assert_eq!(
            field_names,
            vec!["user_id", "created_at", "is_active", "metadata_json"]
        );
    }
}
