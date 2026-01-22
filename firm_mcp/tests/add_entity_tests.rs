mod helpers;

use firm_core::graph::EntityGraph;
use firm_mcp::tools::add_entity::{AddEntityParams, execute};
use helpers::create_workspace;
use std::collections::HashMap;
use std::fs;

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_add_entity_success() {
        let (dir, mut workspace) = create_workspace(&[(
            "schema.firm",
            r#"
schema task {
    field { name = "title" type = "string" required = true }
    field { name = "priority" type = "integer" required = false }
    field { name = "done" type = "boolean" required = true }
}
"#,
        )]);

        let build = workspace.build().unwrap();
        let mut graph = EntityGraph::new();
        graph.add_entities(build.entities.clone()).unwrap();

        let mut fields = HashMap::new();
        fields.insert("title".to_string(), serde_json::json!("Fix bug"));
        fields.insert("priority".to_string(), serde_json::json!(1));
        fields.insert("done".to_string(), serde_json::json!(false));

        let params = AddEntityParams {
            r#type: "task".to_string(),
            id: "bug_fix".to_string(),
            fields,
            to_file: None,
        };

        let result = execute(dir.path(), &build, &graph, &params);

        assert!(result.is_ok());
        let result_val = result.unwrap();
        assert_eq!(result_val.created_new_file, true);
        assert!(result_val.path.contains("generated"));
        assert!(result_val.path.contains("task.firm"));

        // Verify content
        let content = fs::read_to_string(dir.path().join(&result_val.path)).unwrap();
        assert!(content.contains("task bug_fix {"));
        assert!(content.contains(r#"title = "Fix bug""#));
        assert!(content.contains("priority = 1"));
        assert!(content.contains("done = false"));
    }

    #[test]
    fn test_add_entity_with_special_types() {
        let (dir, mut workspace) = create_workspace(&[(
            "schema.firm",
            r#"
schema event {
    field { name = "when" type = "datetime" required = true }
    field { name = "cost" type = "currency" required = true }
    field { name = "attachment" type = "path" required = false }
    field { name = "organizer" type = "reference" required = false }
}
"#,
        )]);

        let build = workspace.build().unwrap();
        let mut graph = EntityGraph::new();
        graph.add_entities(build.entities.clone()).unwrap();

        let mut fields = HashMap::new();
        fields.insert(
            "when".to_string(),
            serde_json::json!("2024-01-01T12:00:00Z"),
        );
        fields.insert("cost".to_string(), serde_json::json!("100.50 USD"));
        fields.insert("attachment".to_string(), serde_json::json!("docs/plan.md"));
        fields.insert("organizer".to_string(), serde_json::json!("person.john"));

        let params = AddEntityParams {
            r#type: "event".to_string(),
            id: "launch".to_string(),
            fields,
            to_file: Some("events/launch.firm".to_string()),
        };

        let result = execute(dir.path(), &build, &graph, &params);
        assert!(result.is_ok());
        let val = result.unwrap();

        // Verify content
        let content = fs::read_to_string(dir.path().join(&val.path)).unwrap();
        // Check DateTime format used by Firm DSL
        assert!(content.contains("2024-01-01 at 12:00 UTC"));
        assert!(content.contains("100.50 USD"));
        // Path should be relative to the file.
        // File is in `events/launch.firm`. Path is `docs/plan.md`.
        // Relative path should be `../docs/plan.md`.
        assert!(content.contains(r#"path"../docs/plan.md""#));
        assert!(content.contains("organizer = person.john"));
    }

    #[test]
    fn test_add_entity_validation_failure() {
        let (dir, mut workspace) = create_workspace(&[(
            "schema.firm",
            r#"
schema task {
    field { name = "title" type = "string" required = true }
}
"#,
        )]);

        let build = workspace.build().unwrap();
        let mut graph = EntityGraph::new();
        graph.add_entities(build.entities.clone()).unwrap();

        let fields = HashMap::new(); // Missing title

        let params = AddEntityParams {
            r#type: "task".to_string(),
            id: "bug_fix".to_string(),
            fields,
            to_file: None,
        };

        let result = execute(dir.path(), &build, &graph, &params);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Validation failed"));
        assert!(err.contains("Missing required field"));
    }

    #[test]
    fn test_add_entity_duplicate_id() {
        let (dir, mut workspace) = create_workspace(&[(
            "data.firm",
            r#"
schema task {
    field { name = "title" type = "string" required = true }
}
task bug_fix {
    title = "Existing"
}
"#,
        )]);

        let build = workspace.build().unwrap();
        let mut graph = EntityGraph::new();
        graph.add_entities(build.entities.clone()).unwrap();
        graph.build(); // Ensure lookups work

        let mut fields = HashMap::new();
        fields.insert("title".to_string(), serde_json::json!("Duplicate"));

        let params = AddEntityParams {
            r#type: "task".to_string(),
            id: "bug_fix".to_string(), // ID collision
            fields,
            to_file: None,
        };

        let result = execute(dir.path(), &build, &graph, &params);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already exists"));
    }
}
