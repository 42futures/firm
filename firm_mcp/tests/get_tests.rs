mod helpers;

use firm_mcp::tools::get::{GetParams, execute};
use helpers::{create_workspace, get_text, is_error, is_success};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_entity_success() {
        let (_dir, mut workspace) = create_workspace(&[(
            "data.firm",
            r#"
schema person {
    field { name = "name" type = "string" required = true }
    field { name = "age" type = "integer" required = false }
}

person john {
    name = "John Doe"
    age = 42
}
"#,
        )]);

        let build = workspace.build().unwrap();
        let params = GetParams {
            r#type: "person".to_string(),
            id: "john".to_string(),
        };

        let result = execute(&build, &params);

        assert!(is_success(&result));
        let text = get_text(&result);
        assert!(text.contains("person.john"));
        assert!(text.contains("John Doe"));
        assert!(text.contains("42"));
    }

    #[test]
    fn test_get_entity_with_reference() {
        let (_dir, mut workspace) = create_workspace(&[(
            "data.firm",
            r#"
schema person {
    field { name = "name" type = "string" required = true }
    field { name = "manager" type = "reference" required = false }
}

person alice {
    name = "Alice"
}

person bob {
    name = "Bob"
    manager = person.alice
}
"#,
        )]);

        let build = workspace.build().unwrap();
        let params = GetParams {
            r#type: "person".to_string(),
            id: "bob".to_string(),
        };

        let result = execute(&build, &params);

        assert!(is_success(&result));
        let text = get_text(&result);
        assert!(text.contains("Bob"));
        assert!(text.contains("person.alice"));
    }

    #[test]
    fn test_get_entity_not_found() {
        let (_dir, mut workspace) = create_workspace(&[(
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

        let build = workspace.build().unwrap();
        let params = GetParams {
            r#type: "person".to_string(),
            id: "nonexistent".to_string(),
        };

        let result = execute(&build, &params);

        assert!(is_error(&result));
    }

    #[test]
    fn test_get_entity_wrong_type() {
        let (_dir, mut workspace) = create_workspace(&[(
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

        let build = workspace.build().unwrap();
        // Entity exists as person.john, but we're asking for organization.john
        let params = GetParams {
            r#type: "organization".to_string(),
            id: "john".to_string(),
        };

        let result = execute(&build, &params);

        assert!(is_error(&result));
    }

    #[test]
    fn test_get_schema_success() {
        let (_dir, mut workspace) = create_workspace(&[(
            "schemas.firm",
            r#"
schema person {
    field { name = "name" type = "string" required = true }
    field { name = "email" type = "string" required = false }
    field { name = "age" type = "integer" required = false }
}
"#,
        )]);

        let build = workspace.build().unwrap();
        let params = GetParams {
            r#type: "schema".to_string(),
            id: "person".to_string(),
        };

        let result = execute(&build, &params);

        assert!(is_success(&result));
        let text = get_text(&result);
        assert!(text.contains("person"));
        assert!(text.contains("name"));
        assert!(text.contains("email"));
        assert!(text.contains("age"));
    }

    #[test]
    fn test_get_schema_with_enum() {
        let (_dir, mut workspace) = create_workspace(&[(
            "schemas.firm",
            r#"
schema task {
    field { name = "title" type = "string" required = true }
    field { name = "status" type = "enum" allowed_values = ["todo", "in_progress", "done"] required = true }
}
"#,
        )]);

        let build = workspace.build().unwrap();
        let params = GetParams {
            r#type: "schema".to_string(),
            id: "task".to_string(),
        };

        let result = execute(&build, &params);

        assert!(is_success(&result));
        let text = get_text(&result);
        assert!(text.contains("task"));
        assert!(text.contains("status"));
    }

    #[test]
    fn test_get_schema_not_found() {
        let (_dir, mut workspace) = create_workspace(&[(
            "schemas.firm",
            r#"
schema person {
    field { name = "name" type = "string" required = true }
}
"#,
        )]);

        let build = workspace.build().unwrap();
        let params = GetParams {
            r#type: "schema".to_string(),
            id: "organization".to_string(),
        };

        let result = execute(&build, &params);

        assert!(is_error(&result));
    }

    #[test]
    fn test_get_distinguishes_entity_from_schema_with_same_name() {
        let (_dir, mut workspace) = create_workspace(&[(
            "data.firm",
            r#"
schema person {
    field { name = "name" type = "string" required = true }
}

person person {
    name = "A person named Person"
}
"#,
        )]);

        let build = workspace.build().unwrap();

        // Get the entity person.person
        let params = GetParams {
            r#type: "person".to_string(),
            id: "person".to_string(),
        };
        let result = execute(&build, &params);
        assert!(is_success(&result));
        assert!(get_text(&result).contains("A person named Person"));

        // Get the schema person
        let params = GetParams {
            r#type: "schema".to_string(),
            id: "person".to_string(),
        };
        let result = execute(&build, &params);
        assert!(is_success(&result));
        // Schema output should contain field definitions, not entity data
        assert!(get_text(&result).contains("name"));
        assert!(!get_text(&result).contains("A person named Person"));
    }
}
