mod helpers;

use firm_mcp::tools::delete_source::{DeleteSourceParams, execute, rollback};
use helpers::{create_workspace, is_error, is_success};

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_delete_source_success() {
        let (dir, _workspace) = create_workspace(&[(
            "data.firm",
            r#"
schema person {
    field { name = "name" type = "string" required = true }
}
"#,
        )]);

        let params = DeleteSourceParams {
            path: "data.firm".to_string(),
            force: false,
        };

        let result = execute(dir.path(), &params);

        assert!(result.is_ok());
        assert!(!dir.path().join("data.firm").exists());
    }

    #[test]
    fn test_delete_source_preserves_content_for_rollback() {
        let content = "schema person {\n    field { name = \"name\" type = \"string\" required = true }\n}\n";
        let (dir, _workspace) = create_workspace(&[("data.firm", content)]);

        let params = DeleteSourceParams {
            path: "data.firm".to_string(),
            force: false,
        };

        let result = execute(dir.path(), &params).unwrap();

        assert_eq!(result.original_content, content);
        assert!(!dir.path().join("data.firm").exists());
    }

    #[test]
    fn test_delete_source_in_subdirectory() {
        let (dir, _workspace) = create_workspace(&[(
            "schemas/person.firm",
            r#"
schema person {
    field { name = "name" type = "string" required = true }
}
"#,
        )]);

        let params = DeleteSourceParams {
            path: "schemas/person.firm".to_string(),
            force: false,
        };

        let result = execute(dir.path(), &params);

        assert!(result.is_ok());
        assert!(!dir.path().join("schemas/person.firm").exists());
    }

    #[test]
    fn test_delete_source_file_not_found() {
        let dir = TempDir::new().expect("Failed to create temp dir");

        let params = DeleteSourceParams {
            path: "nonexistent.firm".to_string(),
            force: false,
        };

        let result = execute(dir.path(), &params);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("File not found"));
    }

    #[test]
    fn test_delete_source_not_firm_extension() {
        let dir = TempDir::new().expect("Failed to create temp dir");
        fs::write(dir.path().join("data.txt"), "hello").unwrap();

        let params = DeleteSourceParams {
            path: "data.txt".to_string(),
            force: false,
        };

        let result = execute(dir.path(), &params);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains(".firm extension"));
        assert!(dir.path().join("data.txt").exists());
    }

    #[test]
    fn test_delete_source_path_traversal_rejected() {
        let (dir, _workspace) = create_workspace(&[(
            "data.firm",
            "schema test { field { name = \"n\" type = \"string\" required = true } }",
        )]);

        let params = DeleteSourceParams {
            path: "../escape.firm".to_string(),
            force: false,
        };

        let result = execute(dir.path(), &params);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid path"));
    }

    #[test]
    fn test_rollback_restores_file() {
        let content = "schema person { field { name = \"name\" type = \"string\" required = true } }";
        let (dir, _workspace) = create_workspace(&[("data.firm", content)]);

        // Delete the file
        let params = DeleteSourceParams {
            path: "data.firm".to_string(),
            force: false,
        };
        let result = execute(dir.path(), &params).unwrap();
        assert!(!dir.path().join("data.firm").exists());

        // Rollback should restore it
        let restored = rollback(dir.path(), "data.firm", &result.original_content);
        assert!(restored);
        assert!(dir.path().join("data.firm").exists());
        assert_eq!(fs::read_to_string(dir.path().join("data.firm")).unwrap(), content);
    }

    #[test]
    fn test_result_messages() {
        use firm_mcp::tools::delete_source::{
            force_success_result, success_result, validation_error_result,
        };
        use helpers::get_text;

        let success = success_result("data.firm");
        assert!(is_success(&success));
        assert!(get_text(&success).contains("Deleted"));

        let force = force_success_result("data.firm", "broken ref");
        assert!(is_success(&force));

        let error = validation_error_result("broken ref", true);
        assert!(is_error(&error));
        assert!(get_text(&error).contains("restored"));

        let error_no_rollback = validation_error_result("broken ref", false);
        assert!(is_error(&error_no_rollback));
        assert!(get_text(&error_no_rollback).contains("Failed to restore"));
    }
}
