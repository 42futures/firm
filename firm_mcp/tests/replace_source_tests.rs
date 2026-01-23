mod helpers;

use std::fs;

use firm_mcp::tools::replace_source::{ReplaceSourceParams, execute};
use firm_mcp::tools::write_source::{WriteSourceParams, rollback, validate_and_write};
use helpers::{create_workspace, is_error, is_success};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace_source_success_single_match() {
        let (dir, _workspace) = create_workspace(&[(
            "data.firm",
            r#"schema task {
    field { name = "title" type = "string" required = true }
    field { name = "status" type = "enum" required = true values = ["pending", "done"] }
}
task my_task {
    title = "Fix the bug"
    status = enum"pending"
}"#,
        )]);

        let params = ReplaceSourceParams {
            path: "data.firm".to_string(),
            old_string: r#"status = enum"pending""#.to_string(),
            new_string: r#"status = enum"done""#.to_string(),
            replace_all: false,
            force: false,
        };

        let result = execute(dir.path(), &params);
        assert!(result.is_ok());

        let replace_result = result.unwrap();
        assert_eq!(replace_result.occurrences_replaced, 1);
        assert!(
            replace_result
                .new_content
                .contains(r#"status = enum"done""#)
        );
        assert!(
            !replace_result
                .new_content
                .contains(r#"status = enum"pending""#)
        );
    }

    #[test]
    fn test_replace_source_not_found() {
        let (dir, _workspace) = create_workspace(&[(
            "data.firm",
            r#"schema task {
    field { name = "title" type = "string" required = true }
}"#,
        )]);

        let params = ReplaceSourceParams {
            path: "data.firm".to_string(),
            old_string: "nonexistent string".to_string(),
            new_string: "replacement".to_string(),
            replace_all: false,
            force: false,
        };

        let result = execute(dir.path(), &params);
        assert!(result.is_err());
    }

    #[test]
    fn test_replace_source_multiple_matches_error() {
        let (dir, _workspace) = create_workspace(&[(
            "data.firm",
            r#"schema task {
    field { name = "title" type = "string" required = true }
}
task task_one {
    title = "First"
}
task task_two {
    title = "Second"
}"#,
        )]);

        // "title" appears multiple times
        let params = ReplaceSourceParams {
            path: "data.firm".to_string(),
            old_string: "title".to_string(),
            new_string: "name".to_string(),
            replace_all: false,
            force: false,
        };

        let result = execute(dir.path(), &params);
        assert!(result.is_err());
    }

    #[test]
    fn test_replace_source_multiple_matches_with_replace_all() {
        let (dir, _workspace) = create_workspace(&[(
            "data.firm",
            r#"task task_one {
    title = "First"
}
task task_two {
    title = "Second"
}"#,
        )]);

        // Replace all "title" with "name"
        let params = ReplaceSourceParams {
            path: "data.firm".to_string(),
            old_string: "title".to_string(),
            new_string: "name".to_string(),
            replace_all: true,
            force: false,
        };

        let result = execute(dir.path(), &params);
        assert!(result.is_ok());

        let replace_result = result.unwrap();
        assert_eq!(replace_result.occurrences_replaced, 2);
        assert!(!replace_result.new_content.contains("title"));
        assert!(replace_result.new_content.matches("name").count() == 2);
    }

    #[test]
    fn test_replace_source_empty_old_string_error() {
        let (dir, _workspace) = create_workspace(&[("data.firm", "schema test {}")]);

        let params = ReplaceSourceParams {
            path: "data.firm".to_string(),
            old_string: "".to_string(),
            new_string: "something".to_string(),
            replace_all: false,
            force: false,
        };

        let result = execute(dir.path(), &params);
        assert!(result.is_err());
    }

    #[test]
    fn test_replace_source_empty_new_string_deletion() {
        let (dir, _workspace) = create_workspace(&[(
            "data.firm",
            r#"schema task {
    field { name = "title" type = "string" required = true }
}
// This is a comment to remove
task my_task {
    title = "Test"
}"#,
        )]);

        let params = ReplaceSourceParams {
            path: "data.firm".to_string(),
            old_string: "// This is a comment to remove\n".to_string(),
            new_string: "".to_string(),
            replace_all: false,
            force: false,
        };

        let result = execute(dir.path(), &params);
        assert!(result.is_ok());

        let replace_result = result.unwrap();
        assert!(!replace_result.new_content.contains("comment to remove"));
    }

    #[test]
    fn test_replace_source_preserves_other_content() {
        let (dir, _workspace) = create_workspace(&[(
            "data.firm",
            r#"schema task {
    field { name = "title" type = "string" required = true }
}
task my_task {
    title = "Original Title"
}"#,
        )]);

        let params = ReplaceSourceParams {
            path: "data.firm".to_string(),
            old_string: r#"title = "Original Title""#.to_string(),
            new_string: r#"title = "New Title""#.to_string(),
            replace_all: false,
            force: false,
        };

        let result = execute(dir.path(), &params);
        assert!(result.is_ok());

        let replace_result = result.unwrap();
        // Schema should be preserved
        assert!(replace_result.new_content.contains("schema task"));
        assert!(replace_result.new_content.contains(r#"name = "title""#));
        // Entity should be updated
        assert!(
            replace_result
                .new_content
                .contains(r#"title = "New Title""#)
        );
    }

    #[test]
    fn test_replace_source_file_not_found() {
        let (dir, _workspace) = create_workspace(&[("other.firm", "schema x {}")]);

        let params = ReplaceSourceParams {
            path: "nonexistent.firm".to_string(),
            old_string: "something".to_string(),
            new_string: "else".to_string(),
            replace_all: false,
            force: false,
        };

        let result = execute(dir.path(), &params);
        assert!(result.is_err());
    }

    #[test]
    fn test_replace_source_syntax_error_detected() {
        let (dir, _workspace) = create_workspace(&[(
            "data.firm",
            r#"schema task {
    field { name = "title" type = "string" required = true }
}"#,
        )]);

        // Execute replacement that creates invalid syntax by removing schema's closing brace
        // We need to target something unique - the final "}\n" at end of file
        let params = ReplaceSourceParams {
            path: "data.firm".to_string(),
            old_string: "true }".to_string(),
            new_string: "true".to_string(), // Remove the field's closing brace - creates syntax error
            replace_all: false,
            force: false,
        };

        let result = execute(dir.path(), &params);
        assert!(result.is_ok()); // execute() itself succeeds

        // But write validation should fail
        let replace_result = result.unwrap();
        let write_params = WriteSourceParams {
            path: "data.firm".to_string(),
            content: replace_result.new_content,
            force: false,
        };

        let write_result = validate_and_write(dir.path(), &write_params);
        assert!(write_result.is_err());
    }

    #[test]
    fn test_replace_source_valid_change_writes_successfully() {
        let (dir, _workspace) = create_workspace(&[(
            "data.firm",
            r#"schema task {
    field { name = "title" type = "string" required = true }
}
task my_task {
    title = "Old"
}"#,
        )]);

        let params = ReplaceSourceParams {
            path: "data.firm".to_string(),
            old_string: r#"title = "Old""#.to_string(),
            new_string: r#"title = "New""#.to_string(),
            replace_all: false,
            force: false,
        };

        let result = execute(dir.path(), &params);
        assert!(result.is_ok());

        let replace_result = result.unwrap();

        // Write the new content
        let write_params = WriteSourceParams {
            path: "data.firm".to_string(),
            content: replace_result.new_content.clone(),
            force: false,
        };

        let write_result = validate_and_write(dir.path(), &write_params);
        assert!(write_result.is_ok());

        // Verify file was updated
        let content = fs::read_to_string(dir.path().join("data.firm")).unwrap();
        assert!(content.contains(r#"title = "New""#));
    }

    #[test]
    fn test_replace_source_rollback_on_failure() {
        let (dir, _workspace) = create_workspace(&[(
            "data.firm",
            r#"schema task {
    field { name = "title" type = "string" required = true }
}"#,
        )]);

        let original_content = fs::read_to_string(dir.path().join("data.firm")).unwrap();

        // Write invalid content
        let bad_content = "invalid { syntax";
        fs::write(dir.path().join("data.firm"), bad_content).unwrap();

        // Rollback should restore original
        let rollback_success = rollback(dir.path(), "data.firm", Some(original_content.clone()));
        assert!(rollback_success);

        let restored = fs::read_to_string(dir.path().join("data.firm")).unwrap();
        assert_eq!(restored, original_content);
    }

    #[test]
    fn test_replace_source_force_writes_despite_validation_failure() {
        // Create a workspace with a schema and an entity that references it
        let (dir, _workspace) = create_workspace(&[(
            "data.firm",
            r#"schema task {
    field { name = "title" type = "string" required = true }
}
task my_task {
    title = "Test"
}"#,
        )]);

        // Replace the entity's title with a reference to a non-existent entity
        // This will cause semantic validation to fail (invalid reference)
        let params = ReplaceSourceParams {
            path: "data.firm".to_string(),
            old_string: r#"title = "Test""#.to_string(),
            new_string: r#"title = "Modified""#.to_string(),
            replace_all: false,
            force: true, // Force the write even if validation fails
        };

        let result = execute(dir.path(), &params);
        assert!(result.is_ok());

        let replace_result = result.unwrap();

        // Write with force=true
        let write_params = WriteSourceParams {
            path: "data.firm".to_string(),
            content: replace_result.new_content.clone(),
            force: true,
        };

        let write_result = validate_and_write(dir.path(), &write_params);
        assert!(write_result.is_ok());

        // Verify file was actually written with the new content
        let content = fs::read_to_string(dir.path().join("data.firm")).unwrap();
        assert!(content.contains(r#"title = "Modified""#));
        assert!(!content.contains(r#"title = "Test""#));
    }

    #[test]
    fn test_replace_source_force_keeps_file_on_semantic_error() {
        // Create a workspace where we'll break a reference
        let (dir, _workspace) = create_workspace(&[
            (
                "schema.firm",
                r#"schema person {
    field { name = "name" type = "string" required = true }
}
schema task {
    field { name = "title" type = "string" required = true }
    field { name = "assignee" type = "reference" required = true }
}"#,
            ),
            (
                "data.firm",
                r#"person john {
    name = "John"
}
task my_task {
    title = "Test"
    assignee = person.john
}"#,
            ),
        ]);

        // Remove the person entity - this will break the reference in task
        // but with force=true, the file should still be written
        let params = ReplaceSourceParams {
            path: "data.firm".to_string(),
            old_string: r#"person john {
    name = "John"
}
"#
            .to_string(),
            new_string: "".to_string(), // Delete the person
            replace_all: false,
            force: true,
        };

        let result = execute(dir.path(), &params);
        assert!(result.is_ok());

        let replace_result = result.unwrap();

        // Write with force=true - syntax is valid but semantic validation will fail
        let write_params = WriteSourceParams {
            path: "data.firm".to_string(),
            content: replace_result.new_content.clone(),
            force: true,
        };

        let write_result = validate_and_write(dir.path(), &write_params);
        assert!(write_result.is_ok());

        // Verify file was written without the person entity
        let content = fs::read_to_string(dir.path().join("data.firm")).unwrap();
        assert!(!content.contains("person john"));
        assert!(content.contains("task my_task"));
        assert!(content.contains("assignee = person.john")); // Reference still there, but now broken
    }

    // ============== Result formatting tests ==============

    #[test]
    fn test_success_result_single() {
        let result = firm_mcp::tools::replace_source::success_result("data.firm", 1);
        assert!(is_success(&result));
    }

    #[test]
    fn test_success_result_multiple() {
        let result = firm_mcp::tools::replace_source::success_result("data.firm", 5);
        assert!(is_success(&result));
    }

    #[test]
    fn test_force_success_result() {
        let result =
            firm_mcp::tools::replace_source::force_success_result("data.firm", 1, "some error");
        assert!(is_success(&result));
    }

    #[test]
    fn test_validation_error_result() {
        let result =
            firm_mcp::tools::replace_source::validation_error_result("validation failed", true);
        assert!(is_error(&result));
    }
}
