mod helpers;

use std::fs;

use firm_mcp::tools::write_source::{
    WriteSourceParams, force_success_result, rollback, success_result, validate_and_write,
    validation_error_result,
};
use helpers::{create_workspace, get_text, is_error, is_success};
use tempfile::TempDir;

#[cfg(test)]
mod tests {
    use super::*;

    // ============== validate_and_write Tests ==============

    #[test]
    fn test_validate_and_write_new_file() {
        let dir = TempDir::new().unwrap();

        let content = r#"schema test {
    field { name = "name" type = "string" required = true }
}"#;

        let params = WriteSourceParams {
            path: "new.firm".to_string(),
            content: content.to_string(),
            force: false,
        };

        let result = validate_and_write(dir.path(), &params);

        assert!(result.is_ok());
        let write_result = result.unwrap();
        assert!(!write_result.file_existed);
        assert!(write_result.original_content.is_none());

        // File should exist with exact content
        let written = fs::read_to_string(dir.path().join("new.firm")).unwrap();
        assert_eq!(written, content);
    }

    #[test]
    fn test_validate_and_write_update_existing() {
        let (dir, _workspace) = create_workspace(&[(
            "existing.firm",
            "schema old { field { name = \"x\" type = \"string\" required = true } }",
        )]);

        let new_content = r#"schema new {
    field { name = "y" type = "integer" required = true }
}"#;

        let params = WriteSourceParams {
            path: "existing.firm".to_string(),
            content: new_content.to_string(),
            force: false,
        };

        let result = validate_and_write(dir.path(), &params);

        assert!(result.is_ok());
        let write_result = result.unwrap();
        assert!(write_result.file_existed);
        assert!(write_result.original_content.is_some());
        assert!(
            write_result
                .original_content
                .unwrap()
                .contains("schema old")
        );

        // File should have exact new content
        let written = fs::read_to_string(dir.path().join("existing.firm")).unwrap();
        assert_eq!(written, new_content);
    }

    #[test]
    fn test_validate_and_write_creates_subdirectories() {
        let dir = TempDir::new().unwrap();

        let params = WriteSourceParams {
            path: "a/b/c/deep.firm".to_string(),
            content: "schema deep { field { name = \"x\" type = \"string\" required = true } }"
                .to_string(),
            force: false,
        };

        let result = validate_and_write(dir.path(), &params);

        assert!(result.is_ok());
        assert!(dir.path().join("a/b/c/deep.firm").exists());
    }

    #[test]
    fn test_validate_and_write_syntax_error_rejected() {
        let dir = TempDir::new().unwrap();

        let params = WriteSourceParams {
            path: "bad.firm".to_string(),
            content: "this is not valid { syntax".to_string(),
            force: false,
        };

        let result = validate_and_write(dir.path(), &params);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(is_error(&error));

        // File should NOT be created
        assert!(!dir.path().join("bad.firm").exists());
    }

    #[test]
    fn test_validate_and_write_unclosed_brace_rejected() {
        let dir = TempDir::new().unwrap();

        let params = WriteSourceParams {
            path: "unclosed.firm".to_string(),
            content: r#"schema test {
    field { name = "x" type = "string" required = true }
"#
            .to_string(), // Missing closing brace
            force: false,
        };

        let result = validate_and_write(dir.path(), &params);

        assert!(result.is_err());
        // File should NOT be created
        assert!(!dir.path().join("unclosed.firm").exists());
    }

    #[test]
    fn test_validate_and_write_not_firm_extension_rejected() {
        let dir = TempDir::new().unwrap();

        let params = WriteSourceParams {
            path: "file.txt".to_string(),
            content: "schema test {}".to_string(),
            force: false,
        };

        let result = validate_and_write(dir.path(), &params);

        assert!(result.is_err());
    }

    // ============== rollback Tests ==============

    #[test]
    fn test_rollback_restores_original_content() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.firm");

        // Write original
        fs::write(&file_path, "original content").unwrap();

        // Overwrite with new
        fs::write(&file_path, "new content").unwrap();

        // Rollback
        let success = rollback(
            dir.path(),
            "test.firm",
            Some("original content".to_string()),
        );

        assert!(success);
        let restored = fs::read_to_string(&file_path).unwrap();
        assert_eq!(restored, "original content");
    }

    #[test]
    fn test_rollback_deletes_new_file() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("new.firm");

        // Create new file
        fs::write(&file_path, "new content").unwrap();
        assert!(file_path.exists());

        // Rollback (no original content means delete)
        let success = rollback(dir.path(), "new.firm", None);

        assert!(success);
        assert!(!file_path.exists());
    }

    // ============== Result Helper Tests ==============

    #[test]
    fn test_success_result_created() {
        let result = success_result("new.firm", 100, false);

        assert!(is_success(&result));
        let text = get_text(&result);
        assert!(text.contains("Created"));
        assert!(text.contains("new.firm"));
        assert!(text.contains("100 bytes"));
        assert!(text.contains("valid"));
    }

    #[test]
    fn test_success_result_updated() {
        let result = success_result("existing.firm", 200, true);

        assert!(is_success(&result));
        let text = get_text(&result);
        assert!(text.contains("Updated"));
        assert!(text.contains("existing.firm"));
        assert!(text.contains("200 bytes"));
    }

    #[test]
    fn test_force_success_result() {
        let result = force_success_result("file.firm", 150, false, "missing schema for type X");

        assert!(is_success(&result));
        let text = get_text(&result);
        assert!(text.contains("Created"));
        assert!(text.contains("Warning"));
        assert!(text.contains("missing schema"));
    }

    #[test]
    fn test_validation_error_result_with_rollback() {
        let result = validation_error_result("invalid reference", true);

        assert!(is_error(&result));
        let text = get_text(&result);
        assert!(text.contains("invalid reference"));
        assert!(text.contains("rolled back"));
        assert!(text.contains("force: true"));
    }

    #[test]
    fn test_validation_error_result_rollback_failed() {
        let result = validation_error_result("some error", false);

        assert!(is_error(&result));
        let text = get_text(&result);
        assert!(text.contains("Failed to rollback"));
    }

    // ============== Edge Cases ==============

    #[test]
    fn test_validate_and_write_empty_content() {
        let dir = TempDir::new().unwrap();

        let params = WriteSourceParams {
            path: "empty.firm".to_string(),
            content: "".to_string(),
            force: false,
        };

        // Empty content should be valid (no syntax errors)
        let result = validate_and_write(dir.path(), &params);

        assert!(result.is_ok());
        let written = fs::read_to_string(dir.path().join("empty.firm")).unwrap();
        assert_eq!(written, "");
    }

    #[test]
    fn test_validate_and_write_comment_only() {
        let dir = TempDir::new().unwrap();

        let params = WriteSourceParams {
            path: "comments.firm".to_string(),
            content: "// This is just a comment\n// Another comment".to_string(),
            force: false,
        };

        let result = validate_and_write(dir.path(), &params);

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_and_write_multiple_schemas() {
        let dir = TempDir::new().unwrap();

        let params = WriteSourceParams {
            path: "multi.firm".to_string(),
            content: r#"
schema person {
    field { name = "name" type = "string" required = true }
}

schema organization {
    field { name = "name" type = "string" required = true }
}

person john { name = "John" }
organization acme { name = "ACME" }
"#
            .to_string(),
            force: false,
        };

        let result = validate_and_write(dir.path(), &params);

        assert!(result.is_ok());
    }
}
