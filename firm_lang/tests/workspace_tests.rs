use firm_core::EntityType;
use firm_lang::workspace::{Workspace, WorkspaceError};

use std::path::PathBuf;
use tempfile::TempDir;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_single_file() {
        let mut workspace = Workspace::new();
        let path = PathBuf::from("../example/core/main.firm");

        let result = workspace.load_file(&path, &PathBuf::from("../example"));
        assert!(result.is_ok(), "Should load example file successfully");
    }

    #[test]
    fn test_load_nonexistent_file() {
        let mut workspace = Workspace::new();
        let path = PathBuf::from("nonexistent.firm");

        let result = workspace.load_file(&path, &PathBuf::new());
        assert!(result.is_err(), "Should fail to load nonexistent file");

        match result {
            Err(WorkspaceError::IoError(_)) => {}
            _ => panic!("Expected IoError for nonexistent file"),
        }
    }

    #[test]
    fn test_load_directory() {
        use std::fs;

        let temp_dir = TempDir::new().unwrap();
        let entity_file = temp_dir.path().join("entities.firm");
        let schema_file = temp_dir.path().join("schemas.firm");

        fs::write(&entity_file, "person john { first_name = \"John\" }")
            .expect("Write entity file");
        fs::write(
            &schema_file,
            "schema test { field { name = \"test\" type = \"string\" required = true } }",
        )
        .expect("Write schema file");

        let mut workspace = Workspace::new();
        let result = workspace.load_directory(&temp_dir.path().to_path_buf());
        assert!(result.is_ok(), "Should load directory successfully");
    }

    #[test]
    fn test_load_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        let mut workspace = Workspace::new();
        let result = workspace.load_directory(&temp_path.to_path_buf());

        assert!(result.is_ok(), "Should handle empty directory gracefully");
    }

    #[test]
    fn test_build_validates_entities_successfully() {
        use std::fs;

        let temp_dir = TempDir::new().unwrap();
        let mixed_file_path = temp_dir.path().join("valid_entities.firm");

        let content = r#"
schema person {
    field {
        name = "name"
        type = "string"
        required = true
    }
}

schema organization {
    field {
        name = "name"
        type = "string"
        required = true
    }
    field {
        name = "primary_email"
        type = "string"
        required = false
    }
}

person john {
    name = "John Doe"
}

organization acme {
    name = "Acme Corp"
    primary_email = "info@acme.com"
}
"#;
        fs::write(&mixed_file_path, content).expect("Should write file");

        let mut workspace = Workspace::new();
        workspace
            .load_file(&mixed_file_path, &PathBuf::new())
            .expect("Should load file");

        let result = workspace.build();
        assert!(result.is_ok(), "Build should succeed with valid entities");

        let build = result.unwrap();
        assert_eq!(build.entities.len(), 2, "Should have 2 entities");
        assert_eq!(build.schemas.len(), 2, "Should have 2 schemas");
    }

    #[test]
    fn test_build_fails_validation_missing_required_field() {
        use std::fs;

        let temp_dir = TempDir::new().unwrap();
        let invalid_file_path = temp_dir.path().join("invalid_entity.firm");

        // Person entity missing required name field
        let content = r#"
schema person {
    field {
        name = "name"
        type = "string"
        required = true
    }
}

person john {
    email = "john@example.com"
}
"#;
        fs::write(&invalid_file_path, content).expect("Should write file");

        let mut workspace = Workspace::new();
        workspace
            .load_file(&invalid_file_path, &PathBuf::new())
            .expect("Should load file");

        let result = workspace.build();
        assert!(result.is_err(), "Build should fail validation");

        match result {
            Err(WorkspaceError::ValidationError(_, _)) => {
                // Expected validation error
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_build_fails_no_schema_for_entity_type() {
        use std::fs;

        let temp_dir = TempDir::new().unwrap();
        let custom_file_path = temp_dir.path().join("custom_entity.firm");

        // Custom entity type without corresponding schema
        let content = r#"
custom_unknown john {
    name = "John"
}
"#;
        fs::write(&custom_file_path, content).expect("Should write file");

        let mut workspace = Workspace::new();
        workspace
            .load_file(&custom_file_path, &PathBuf::new())
            .expect("Should load file");

        let result = workspace.build();
        assert!(
            result.is_err(),
            "Build should fail - no schema for custom type"
        );

        match result {
            Err(WorkspaceError::MissingSchemaError(_, _)) => {
                // Expected missing schema error
            }
            _ => panic!("Expected MissingSchemaError"),
        }
    }

    #[test]
    fn test_build_with_custom_schema_validation() {
        use std::fs;

        let temp_dir = TempDir::new().unwrap();
        let schema_and_entity_file = temp_dir.path().join("custom_with_schema.firm");

        let content = r#"
schema custom_employee {
    field {
        name = "person_ref"
        type = "reference"
        required = true
    }

    field {
        name = "title"
        type = "string"
        required = true
    }
}

custom_employee emp1 {
    person_ref = person.john
    title = "Engineer"
}
"#;
        fs::write(&schema_and_entity_file, content).expect("Should write file");

        let mut workspace = Workspace::new();
        workspace
            .load_file(&schema_and_entity_file, &PathBuf::new())
            .expect("Should load file");

        let result = workspace.build();
        assert!(
            result.is_ok(),
            "Build should succeed with matching custom schema"
        );

        let build = result.unwrap();
        assert_eq!(build.entities.len(), 1, "Should have 1 entity");
        assert_eq!(build.schemas.len(), 1, "Should have custom schema");
    }

    #[test]
    fn test_build_preserves_file_paths_in_errors() {
        use std::fs;

        let temp_dir = TempDir::new().unwrap();
        let invalid_file_path = temp_dir.path().join("invalid_validation.firm");

        // Create a person entity missing required first_name field
        let content = r#"
person broken_person {
    last_name = "Smith"
}
"#;
        fs::write(&invalid_file_path, content).expect("Should write file");

        let mut workspace = Workspace::new();
        workspace
            .load_file(&invalid_file_path, &PathBuf::new())
            .expect("Should load file");

        let result = workspace.build();
        assert!(result.is_err(), "Build should fail validation");
    }

    #[test]
    fn test_build_preserves_file_paths_in_schema_parse_errors() {
        use std::fs;

        let temp_dir = TempDir::new().unwrap();
        let invalid_schema_file = temp_dir.path().join("invalid_schema.firm");

        // Create a schema with invalid field type
        let content = r#"
schema broken_schema {
    field {
        name = "test_field"
        type = "unknown_type"
        required = true
    }
}
"#;
        fs::write(&invalid_schema_file, content).expect("Should write file");

        let mut workspace = Workspace::new();
        workspace
            .load_file(&invalid_schema_file, &PathBuf::new())
            .expect("Should load file");

        let result = workspace.build();
        assert!(result.is_err(), "Build should fail with schema parse error");

        match result {
            Err(WorkspaceError::ParseError(path, message)) => {
                assert_eq!(
                    path, invalid_schema_file,
                    "Error should reference the correct schema file path"
                );
                assert!(
                    message.contains("unknown_type") || message.contains("Unknown field type"),
                    "Error message should mention the unknown field type"
                );
            }
            _ => panic!("Expected ParseError with file path for schema"),
        }
    }

    #[test]
    fn test_build_schema_map_optimization_multiple_entities() {
        use std::fs;

        let temp_dir = TempDir::new().unwrap();
        let entities_file = temp_dir.path().join("multiple_entities.firm");

        // Create multiple entities of the same type to test schema map efficiency
        let content = r#"
schema person {
    field {
        name = "name"
        type = "string"
        required = true
    }
}

schema organization {
    field {
        name = "name"
        type = "string"
        required = true
    }
}

person john {
    name = "John Doe"
}

person jane {
    name = "Jane Smith"
}

organization acme {
    name = "Acme Corp"
}

organization globo {
    name = "Globo Industries"
}
"#;
        fs::write(&entities_file, content).expect("Should write file");

        let mut workspace = Workspace::new();
        workspace
            .load_file(&entities_file, &PathBuf::new())
            .expect("Should load file");

        let result = workspace.build();

        assert!(
            result.is_ok(),
            "Build should succeed with multiple entities"
        );

        let build = result.unwrap();
        assert_eq!(build.entities.len(), 4, "Should have 4 entities");

        // Verify we have the expected entity types
        let person_count = build
            .entities
            .iter()
            .filter(|e| e.entity_type == EntityType::new("person"))
            .count();
        let org_count = build
            .entities
            .iter()
            .filter(|e| e.entity_type == EntityType::new("organization"))
            .count();

        assert_eq!(person_count, 2, "Should have 2 person entities");
        assert_eq!(org_count, 2, "Should have 2 organization entities");
        assert_eq!(build.schemas.len(), 2, "Should have 2 schemas");
    }
}
