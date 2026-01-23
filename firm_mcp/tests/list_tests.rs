mod helpers;

use std::fs;

use firm_mcp::tools::list::{ListParams, execute};
use helpers::{create_workspace, get_text, is_success};
use tempfile::TempDir;

#[cfg(test)]
mod tests {
    use super::*;
    use firm_lang::workspace::Workspace;

    #[test]
    fn test_list_entities_single_type() {
        let (_dir, mut workspace) = create_workspace(&[(
            "data.firm",
            r#"
schema person {
    field { name = "name" type = "string" required = true }
}

person john { name = "John" }
person jane { name = "Jane" }
person bob { name = "Bob" }
"#,
        )]);

        let build = workspace.build().unwrap();
        let params = ListParams {
            r#type: "person".to_string(),
        };

        let result = execute(&build, &params);

        assert!(is_success(&result));
        let text = get_text(&result);
        assert!(text.contains("person.john"));
        assert!(text.contains("person.jane"));
        assert!(text.contains("person.bob"));
    }

    #[test]
    fn test_list_entities_filters_by_type() {
        let (_dir, mut workspace) = create_workspace(&[(
            "data.firm",
            r#"
schema person {
    field { name = "name" type = "string" required = true }
}

schema organization {
    field { name = "name" type = "string" required = true }
}

person alice { name = "Alice" }
person bob { name = "Bob" }
organization acme { name = "ACME" }
organization globex { name = "Globex" }
"#,
        )]);

        let build = workspace.build().unwrap();

        // List only persons
        let params = ListParams {
            r#type: "person".to_string(),
        };
        let result = execute(&build, &params);
        let text = get_text(&result);
        assert!(text.contains("person.alice"));
        assert!(text.contains("person.bob"));
        assert!(!text.contains("organization"));

        // List only organizations
        let params = ListParams {
            r#type: "organization".to_string(),
        };
        let result = execute(&build, &params);
        let text = get_text(&result);
        assert!(text.contains("organization.acme"));
        assert!(text.contains("organization.globex"));
        assert!(!text.contains("person"));
    }

    #[test]
    fn test_list_entities_nonexistent_type_returns_empty() {
        let (_dir, mut workspace) = create_workspace(&[(
            "data.firm",
            r#"
schema person {
    field { name = "name" type = "string" required = true }
}

person john { name = "John" }
"#,
        )]);

        let build = workspace.build().unwrap();
        let params = ListParams {
            r#type: "project".to_string(),
        };

        let result = execute(&build, &params);

        assert!(is_success(&result));
        assert_eq!(get_text(&result), "");
    }

    #[test]
    fn test_list_entities_no_entities_of_type() {
        let dir = TempDir::new().expect("Failed to create temp dir");
        let schema_file = dir.path().join("schema.firm");
        fs::write(
            &schema_file,
            r#"schema person { field { name = "name" type = "string" required = true } }"#,
        )
        .unwrap();

        let mut workspace = Workspace::new();
        workspace
            .load_directory(&dir.path().to_path_buf())
            .expect("Failed to load workspace");
        let build = workspace.build().unwrap();

        let params = ListParams {
            r#type: "person".to_string(),
        };

        let result = execute(&build, &params);

        assert!(is_success(&result));
        assert_eq!(get_text(&result), "");
    }

    #[test]
    fn test_list_schemas() {
        let (_dir, mut workspace) = create_workspace(&[(
            "schemas.firm",
            r#"
schema person {
    field { name = "name" type = "string" required = true }
}

schema organization {
    field { name = "name" type = "string" required = true }
}

schema project {
    field { name = "title" type = "string" required = true }
}
"#,
        )]);

        let build = workspace.build().unwrap();
        let params = ListParams {
            r#type: "schema".to_string(),
        };

        let result = execute(&build, &params);

        assert!(is_success(&result));
        let text = get_text(&result);
        assert!(text.contains("person"));
        assert!(text.contains("organization"));
        assert!(text.contains("project"));
    }

    #[test]
    fn test_list_schemas_single() {
        let (_dir, mut workspace) = create_workspace(&[(
            "schema.firm",
            r#"
schema task {
    field { name = "title" type = "string" required = true }
}
"#,
        )]);

        let build = workspace.build().unwrap();
        let params = ListParams {
            r#type: "schema".to_string(),
        };

        let result = execute(&build, &params);

        assert!(is_success(&result));
        assert_eq!(get_text(&result), "task");
    }

    #[test]
    fn test_list_schemas_across_files() {
        let (_dir, mut workspace) = create_workspace(&[
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

        let build = workspace.build().unwrap();
        let params = ListParams {
            r#type: "schema".to_string(),
        };

        let result = execute(&build, &params);

        assert!(is_success(&result));
        let text = get_text(&result);
        assert!(text.contains("person"));
        assert!(text.contains("project"));
    }
}
