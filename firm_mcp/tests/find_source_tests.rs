mod helpers;

use firm_mcp::tools::find_source::{FindSourceParams, execute};
use helpers::{create_workspace, get_text, is_error, is_success};
use tempfile::TempDir;

#[cfg(test)]
mod tests {
    use super::*;
    use firm_lang::workspace::Workspace;

    #[test]
    fn test_find_entity_source_success() {
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

        let params = FindSourceParams {
            r#type: "person".to_string(),
            id: "john".to_string(),
        };

        let result = execute(&workspace, dir.path(), &params);

        assert!(is_success(&result));
        assert_eq!(get_text(&result), "people.firm");
    }

    #[test]
    fn test_find_entity_source_in_subdirectory() {
        let (dir, workspace) = create_workspace(&[(
            "entities/people.firm",
            r#"
schema person {
    field { name = "name" type = "string" required = true }
}

person jane {
    name = "Jane Smith"
}
"#,
        )]);

        let params = FindSourceParams {
            r#type: "person".to_string(),
            id: "jane".to_string(),
        };

        let result = execute(&workspace, dir.path(), &params);

        assert!(is_success(&result));
        assert_eq!(get_text(&result), "entities/people.firm");
    }

    #[test]
    fn test_find_entity_source_not_found() {
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

        let params = FindSourceParams {
            r#type: "person".to_string(),
            id: "nonexistent".to_string(),
        };

        let result = execute(&workspace, dir.path(), &params);

        assert!(is_error(&result));
    }

    #[test]
    fn test_find_entity_wrong_type() {
        let (dir, workspace) = create_workspace(&[(
            "data.firm",
            r#"
schema person {
    field { name = "name" type = "string" required = true }
}

person john {
    name = "John"
}
"#,
        )]);

        // Looking for organization.john but only person.john exists
        let params = FindSourceParams {
            r#type: "organization".to_string(),
            id: "john".to_string(),
        };

        let result = execute(&workspace, dir.path(), &params);

        assert!(is_error(&result));
    }

    #[test]
    fn test_find_entity_across_multiple_files() {
        let (dir, workspace) = create_workspace(&[
            (
                "schemas.firm",
                r#"
schema person {
    field { name = "name" type = "string" required = true }
}

schema organization {
    field { name = "name" type = "string" required = true }
}
"#,
            ),
            (
                "people.firm",
                r#"
person alice {
    name = "Alice"
}
"#,
            ),
            (
                "orgs.firm",
                r#"
organization acme {
    name = "ACME Corp"
}
"#,
            ),
        ]);

        // Find person in people.firm
        let params = FindSourceParams {
            r#type: "person".to_string(),
            id: "alice".to_string(),
        };
        let result = execute(&workspace, dir.path(), &params);
        assert!(is_success(&result));
        assert_eq!(get_text(&result), "people.firm");

        // Find organization in orgs.firm
        let params = FindSourceParams {
            r#type: "organization".to_string(),
            id: "acme".to_string(),
        };
        let result = execute(&workspace, dir.path(), &params);
        assert!(is_success(&result));
        assert_eq!(get_text(&result), "orgs.firm");
    }

    #[test]
    fn test_find_schema_source_success() {
        let (dir, workspace) = create_workspace(&[(
            "schemas/person.firm",
            r#"
schema person {
    field { name = "name" type = "string" required = true }
    field { name = "age" type = "integer" required = false }
}
"#,
        )]);

        let params = FindSourceParams {
            r#type: "schema".to_string(),
            id: "person".to_string(),
        };

        let result = execute(&workspace, dir.path(), &params);

        assert!(is_success(&result));
        assert_eq!(get_text(&result), "schemas/person.firm");
    }

    #[test]
    fn test_find_schema_source_not_found() {
        let (dir, workspace) = create_workspace(&[(
            "schemas.firm",
            r#"
schema person {
    field { name = "name" type = "string" required = true }
}
"#,
        )]);

        let params = FindSourceParams {
            r#type: "schema".to_string(),
            id: "organization".to_string(),
        };

        let result = execute(&workspace, dir.path(), &params);

        assert!(is_error(&result));
    }

    #[test]
    fn test_find_schema_across_multiple_files() {
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
                "schemas/project.firm",
                r#"
schema project {
    field { name = "title" type = "string" required = true }
}
"#,
            ),
        ]);

        // Find person schema
        let params = FindSourceParams {
            r#type: "schema".to_string(),
            id: "person".to_string(),
        };
        let result = execute(&workspace, dir.path(), &params);
        assert!(is_success(&result));
        assert_eq!(get_text(&result), "schemas/person.firm");

        // Find project schema
        let params = FindSourceParams {
            r#type: "schema".to_string(),
            id: "project".to_string(),
        };
        let result = execute(&workspace, dir.path(), &params);
        assert!(is_success(&result));
        assert_eq!(get_text(&result), "schemas/project.firm");
    }

    #[test]
    fn test_find_source_empty_workspace() {
        let dir = TempDir::new().expect("Failed to create temp dir");
        let mut workspace = Workspace::new();
        workspace
            .load_directory(&dir.path().to_path_buf())
            .expect("Failed to load workspace");

        let params = FindSourceParams {
            r#type: "person".to_string(),
            id: "anyone".to_string(),
        };

        let result = execute(&workspace, dir.path(), &params);

        assert!(is_error(&result));
    }
}
