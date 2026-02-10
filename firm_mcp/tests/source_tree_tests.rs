mod helpers;

use firm_mcp::tools::source_tree::execute;
use helpers::{create_workspace, get_text, is_success};
use tempfile::TempDir;

#[cfg(test)]
mod tests {
    use super::*;
    use firm_lang::workspace::Workspace;

    #[test]
    fn test_source_tree_single_file_at_root() {
        let (dir, workspace) = create_workspace(&[(
            "main.firm",
            r#"
schema person {
    field { name = "name" type = "string" required = true }
}
"#,
        )]);

        let result = execute(&workspace, dir.path());

        assert!(is_success(&result));
        assert_eq!(get_text(&result), "main.firm");
    }

    #[test]
    fn test_source_tree_files_in_subdirectories() {
        let (dir, workspace) = create_workspace(&[
            (
                "schemas/person.firm",
                r#"
schema person {
    field { name = "name" type = "string" required = true }
}
"#,
            ),
            (
                "schemas/task.firm",
                r#"
schema task {
    field { name = "name" type = "string" required = true }
}
"#,
            ),
            (
                "core/people.firm",
                r#"
person john {
    name = "John"
}
"#,
            ),
        ]);

        let result = execute(&workspace, dir.path());

        assert!(is_success(&result));
        let text = get_text(&result);

        // Should have directory groupings
        assert!(text.contains("schemas/"));
        assert!(text.contains("  person.firm"));
        assert!(text.contains("  task.firm"));
        assert!(text.contains("core/"));
        assert!(text.contains("  people.firm"));
    }

    #[test]
    fn test_source_tree_mixed_root_and_subdirectory() {
        let (dir, workspace) = create_workspace(&[
            (
                "main.firm",
                r#"
schema person {
    field { name = "name" type = "string" required = true }
}
"#,
            ),
            (
                "data/people.firm",
                r#"
person alice {
    name = "Alice"
}
"#,
            ),
        ]);

        let result = execute(&workspace, dir.path());

        assert!(is_success(&result));
        let text = get_text(&result);

        // Root file should not be indented
        assert!(text.contains("main.firm"));
        // Subdirectory file should be indented
        assert!(text.contains("data/"));
        assert!(text.contains("  people.firm"));
    }

    #[test]
    fn test_source_tree_empty_workspace() {
        let dir = TempDir::new().expect("Failed to create temp dir");
        let mut workspace = Workspace::new();
        workspace
            .load_directory(&dir.path().to_path_buf())
            .expect("Failed to load workspace");

        let result = execute(&workspace, dir.path());

        assert!(is_success(&result));
        assert_eq!(get_text(&result), "No .firm source files found.");
    }

    #[test]
    fn test_source_tree_nested_directories() {
        let (dir, workspace) = create_workspace(&[
            (
                "schemas/crm/contact.firm",
                r#"
schema contact {
    field { name = "name" type = "string" required = true }
}
"#,
            ),
            (
                "schemas/crm/account.firm",
                r#"
schema account {
    field { name = "name" type = "string" required = true }
}
"#,
            ),
        ]);

        let result = execute(&workspace, dir.path());

        assert!(is_success(&result));
        let text = get_text(&result);

        // Nested path should show as full directory
        assert!(text.contains("schemas/crm/"));
        assert!(text.contains("  account.firm"));
        assert!(text.contains("  contact.firm"));
    }

    #[test]
    fn test_source_tree_files_are_sorted() {
        let (dir, workspace) = create_workspace(&[
            ("z_last.firm", "schema z { field { name = \"n\" type = \"string\" required = true } }"),
            ("a_first.firm", "schema a { field { name = \"n\" type = \"string\" required = true } }"),
            ("m_middle.firm", "schema m { field { name = \"n\" type = \"string\" required = true } }"),
        ]);

        let result = execute(&workspace, dir.path());

        assert!(is_success(&result));
        let text = get_text(&result);
        let lines: Vec<&str> = text.lines().collect();

        assert_eq!(lines[0], "a_first.firm");
        assert_eq!(lines[1], "m_middle.firm");
        assert_eq!(lines[2], "z_last.firm");
    }
}
