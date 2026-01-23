mod helpers;

use std::fs;

use firm_mcp::tools::read_source::{ReadSourceParams, execute};
use helpers::{create_workspace, get_text, is_error, is_success};
use tempfile::TempDir;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_source_success() {
        let (dir, _workspace) = create_workspace(&[(
            "data.firm",
            r#"schema person {
    field { name = "name" type = "string" required = true }
}

person john {
    name = "John Doe"
}
"#,
        )]);

        let params = ReadSourceParams {
            path: "data.firm".to_string(),
        };

        let result = execute(dir.path(), &params);

        assert!(is_success(&result));
    }

    #[test]
    fn test_read_source_in_subdirectory() {
        let (dir, _workspace) = create_workspace(&[(
            "schemas/person.firm",
            r#"schema person {
    field { name = "name" type = "string" required = true }
}
"#,
        )]);

        let params = ReadSourceParams {
            path: "schemas/person.firm".to_string(),
        };

        let result = execute(dir.path(), &params);

        assert!(is_success(&result));
    }

    #[test]
    fn test_read_source_preserves_formatting() {
        let content = r#"// This is a comment
schema task {
    field {
        name = "title"
        type = "string"
        required = true
    }
}

task example {
    title = "Example Task"
}
"#;

        let (dir, _workspace) = create_workspace(&[("tasks.firm", content)]);

        let params = ReadSourceParams {
            path: "tasks.firm".to_string(),
        };

        let result = execute(dir.path(), &params);

        assert!(is_success(&result));
        // Content should be preserved exactly
        assert_eq!(get_text(&result), content);
    }

    #[test]
    fn test_read_source_file_not_found() {
        let dir = TempDir::new().expect("Failed to create temp dir");

        let params = ReadSourceParams {
            path: "nonexistent.firm".to_string(),
        };

        let result = execute(dir.path(), &params);

        assert!(is_error(&result));
    }

    #[test]
    fn test_read_source_not_firm_file() {
        let dir = TempDir::new().expect("Failed to create temp dir");
        let txt_file = dir.path().join("readme.txt");
        fs::write(&txt_file, "This is a text file").unwrap();

        let params = ReadSourceParams {
            path: "readme.txt".to_string(),
        };

        let result = execute(dir.path(), &params);

        assert!(is_error(&result));
    }

    #[test]
    fn test_read_source_path_traversal_blocked() {
        let (dir, _workspace) = create_workspace(&[("data.firm", "schema test {}")]);

        // Try to read a file outside the workspace using ../
        let params = ReadSourceParams {
            path: "../../../etc/passwd".to_string(),
        };

        let result = execute(dir.path(), &params);

        assert!(is_error(&result));
    }

    #[test]
    fn test_read_source_absolute_path_blocked() {
        let (dir, _workspace) = create_workspace(&[("data.firm", "schema test {}")]);

        // Try to use an absolute path
        let params = ReadSourceParams {
            path: "/etc/passwd".to_string(),
        };

        let result = execute(dir.path(), &params);

        assert!(is_error(&result));
    }

    #[test]
    fn test_read_source_empty_file() {
        let dir = TempDir::new().expect("Failed to create temp dir");
        let empty_file = dir.path().join("empty.firm");
        fs::write(&empty_file, "").unwrap();

        let params = ReadSourceParams {
            path: "empty.firm".to_string(),
        };

        let result = execute(dir.path(), &params);

        assert!(is_success(&result));
        assert_eq!(get_text(&result), "");
    }

    #[test]
    fn test_read_source_deeply_nested() {
        let (dir, _workspace) = create_workspace(&[(
            "a/b/c/d/deep.firm",
            "schema deep { field { name = \"x\" type = \"string\" required = true } }",
        )]);

        let params = ReadSourceParams {
            path: "a/b/c/d/deep.firm".to_string(),
        };

        let result = execute(dir.path(), &params);

        assert!(is_success(&result));
    }
}
