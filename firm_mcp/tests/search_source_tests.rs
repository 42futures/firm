mod helpers;

use firm_mcp::tools::search_source::{SearchSourceParams, execute};
use helpers::{create_workspace, get_text, is_error, is_success};

#[cfg(test)]
mod tests {
    use super::*;
    use firm_lang::workspace::Workspace;
    use tempfile::TempDir;

    #[test]
    fn test_search_finds_matching_lines() {
        let (dir, workspace) = create_workspace(&[(
            "people.firm",
            r#"
schema person {
    field { name = "name" type = "string" required = true }
}

person john {
    name = "John Doe"
}
"#,
        )]);

        let params = SearchSourceParams {
            query: "John".to_string(),
            case_sensitive: false,
        };

        let result = execute(&workspace, dir.path(), &params);

        assert!(is_success(&result));
        let text = get_text(&result);
        assert!(text.contains("people.firm:"));
        assert!(text.contains("John Doe"));
    }

    #[test]
    fn test_search_case_insensitive_by_default() {
        let (dir, workspace) = create_workspace(&[(
            "data.firm",
            r#"
schema task {
    field { name = "name" type = "string" required = true }
}

task my_task {
    name = "Important Task"
}
"#,
        )]);

        let params = SearchSourceParams {
            query: "important".to_string(),
            case_sensitive: false,
        };

        let result = execute(&workspace, dir.path(), &params);

        assert!(is_success(&result));
        assert!(get_text(&result).contains("Important Task"));
    }

    #[test]
    fn test_search_case_sensitive() {
        let (dir, workspace) = create_workspace(&[(
            "data.firm",
            r#"
schema task {
    field { name = "name" type = "string" required = true }
}

task my_task {
    name = "Important Task"
}
"#,
        )]);

        let params = SearchSourceParams {
            query: "important".to_string(),
            case_sensitive: true,
        };

        let result = execute(&workspace, dir.path(), &params);

        assert!(is_success(&result));
        assert!(get_text(&result).contains("No matches found"));
    }

    #[test]
    fn test_search_across_multiple_files() {
        let (dir, workspace) = create_workspace(&[
            (
                "schemas.firm",
                r#"
schema person {
    field { name = "name" type = "string" required = true }
}
"#,
            ),
            (
                "people.firm",
                r#"
person alice {
    name = "Alice Smith"
}
"#,
            ),
            (
                "orgs.firm",
                r#"
schema organization {
    field { name = "name" type = "string" required = true }
}

organization smith_co {
    name = "Smith & Co"
}
"#,
            ),
        ]);

        let params = SearchSourceParams {
            query: "smith".to_string(),
            case_sensitive: false,
        };

        let result = execute(&workspace, dir.path(), &params);

        assert!(is_success(&result));
        let text = get_text(&result);
        // Should find matches in both files
        assert!(text.contains("people.firm:"));
        assert!(text.contains("orgs.firm:"));
    }

    #[test]
    fn test_search_no_matches() {
        let (dir, workspace) = create_workspace(&[(
            "data.firm",
            r#"
schema person {
    field { name = "name" type = "string" required = true }
}
"#,
        )]);

        let params = SearchSourceParams {
            query: "nonexistent_term".to_string(),
            case_sensitive: false,
        };

        let result = execute(&workspace, dir.path(), &params);

        assert!(is_success(&result));
        assert!(get_text(&result).contains("No matches found"));
    }

    #[test]
    fn test_search_empty_query_rejected() {
        let (dir, workspace) = create_workspace(&[(
            "data.firm",
            "schema test { field { name = \"n\" type = \"string\" required = true } }",
        )]);

        let params = SearchSourceParams {
            query: "".to_string(),
            case_sensitive: false,
        };

        let result = execute(&workspace, dir.path(), &params);

        assert!(is_error(&result));
    }

    #[test]
    fn test_search_empty_workspace() {
        let dir = TempDir::new().expect("Failed to create temp dir");
        let mut workspace = Workspace::new();
        workspace
            .load_directory(&dir.path().to_path_buf())
            .expect("Failed to load workspace");

        let params = SearchSourceParams {
            query: "anything".to_string(),
            case_sensitive: false,
        };

        let result = execute(&workspace, dir.path(), &params);

        assert!(is_success(&result));
        assert!(get_text(&result).contains("No matches found"));
    }

    #[test]
    fn test_search_truncates_at_limit() {
        // Generate a file with more than 50 matching lines
        let mut content = String::from(
            "schema item {\n    field { name = \"name\" type = \"string\" required = true }\n}\n\n",
        );
        for i in 0..60 {
            content.push_str(&format!("item entry_{} {{\n    name = \"match\"\n}}\n\n", i));
        }

        let (dir, workspace) = create_workspace(&[("data.firm", &content)]);

        let params = SearchSourceParams {
            query: "match".to_string(),
            case_sensitive: false,
        };

        let result = execute(&workspace, dir.path(), &params);

        assert!(is_success(&result));
        let text = get_text(&result);
        assert!(text.contains("truncated"));

        // Should have exactly 50 match lines (indented with "  ")
        let match_lines = text.lines().filter(|l| l.starts_with("  ")).count();
        assert_eq!(match_lines, 50);
    }

    #[test]
    fn test_search_shows_line_numbers() {
        let (dir, workspace) = create_workspace(&[(
            "data.firm",
            "schema person {\n    field { name = \"name\" type = \"string\" required = true }\n}\n\nperson john {\n    name = \"John\"\n}\n",
        )]);

        let params = SearchSourceParams {
            query: "john".to_string(),
            case_sensitive: false,
        };

        let result = execute(&workspace, dir.path(), &params);

        assert!(is_success(&result));
        let text = get_text(&result);
        // "person john {" is on line 5, "name = "John"" on line 6
        assert!(text.contains("5:"));
        assert!(text.contains("6:"));
    }
}
