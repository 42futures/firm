#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use firm_lang::parser::dsl::parse_source;

    #[test]
    fn test_all_example_files_parse_without_errors() {
        let examples_dir = Path::new("examples");

        if !examples_dir.exists() {
            return;
        }

        let mut tested_files = 0;

        for entry in fs::read_dir(examples_dir).expect("Failed to read examples directory") {
            let entry = entry.expect("Failed to read directory entry");
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("firm") {
                let filename = path.file_name().unwrap().to_str().unwrap();

                let source =
                    fs::read_to_string(&path).expect(&format!("Failed to read file: {}", filename));

                let parsed = parse_source(source, None)
                    .expect(&format!("Failed to parse file: {}", filename));

                assert!(
                    !parsed.has_error(),
                    "Example file {} should not have parse errors",
                    filename
                );
                tested_files += 1;
            }
        }

        assert!(
            tested_files > 0,
            "No .firm files found in examples directory"
        );
    }

    #[test]
    fn test_mixed_entities_and_schemas() {
        let source = r#"
            schema project {
                field {
                    name = "title"
                    type = "string"
                    required = true
                }
            }

            contact john_doe {
                name = "John Doe"
                email = "john@example.com"
            }

            schema invoice {
                field {
                    name = "amount"
                    type = "currency"
                    required = true
                }
            }

            role manager {
                name = "Manager"
                level = 5
            }
        "#;

        let parsed = parse_source(String::from(source), None).unwrap();
        assert!(!parsed.has_error());

        let entities = parsed.entities();
        let schemas = parsed.schemas();

        assert_eq!(entities.len(), 2);
        assert_eq!(schemas.len(), 2);

        // Verify entities
        let entity_types: Vec<_> = entities.iter().map(|e| e.entity_type().unwrap()).collect();
        assert!(entity_types.contains(&"contact"));
        assert!(entity_types.contains(&"role"));

        // Verify schemas
        let schema_names: Vec<_> = schemas.iter().map(|s| s.name().unwrap()).collect();
        assert!(schema_names.contains(&"project"));
        assert!(schema_names.contains(&"invoice"));
    }

    #[test]
    fn test_empty_source() {
        let source = "";
        let parsed = parse_source(String::from(source), None).unwrap();
        assert!(!parsed.has_error());
        assert_eq!(parsed.entities().len(), 0);
        assert_eq!(parsed.schemas().len(), 0);
    }

    #[test]
    fn test_comments_only_source() {
        let source = r#"
            // This is a comment
            /* This is a
               multi-line comment */
            // Another comment
        "#;

        let parsed = parse_source(String::from(source), None).unwrap();
        assert!(!parsed.has_error());
        assert_eq!(parsed.entities().len(), 0);
        assert_eq!(parsed.schemas().len(), 0);
    }

    #[test]
    fn test_syntax_error_detection() {
        let source = r#"
            contact john_doe {
                name = "John Doe"
                // Missing closing brace
        "#;

        let parsed = parse_source(String::from(source), None).unwrap();
        assert!(parsed.has_error());
    }
}
