mod helpers;

use firm_core::graph::EntityGraph;
use firm_mcp::tools::query::{QueryParams, execute};
use helpers::{create_workspace, get_text, is_error, is_success};

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to build workspace and graph together.
    fn create_graph(files: &[(&str, &str)]) -> EntityGraph {
        let (_dir, mut workspace) = create_workspace(files);
        let build = workspace.build().unwrap();

        let mut graph = EntityGraph::new();
        graph.add_entities(build.entities).unwrap();
        graph.build();
        graph
    }

    #[test]
    fn test_query_from_type() {
        let graph = create_graph(&[(
            "data.firm",
            r#"
schema person {
    field { name = "name" type = "string" required = true }
}

person alice { name = "Alice" }
person bob { name = "Bob" }
person charlie { name = "Charlie" }
"#,
        )]);

        let params = QueryParams {
            query: "from person".to_string(),
        };

        let result = execute(&graph, &params);

        assert!(is_success(&result));
        let text = get_text(&result);
        assert!(text.contains("Alice"));
        assert!(text.contains("Bob"));
        assert!(text.contains("Charlie"));
    }

    #[test]
    fn test_query_no_results() {
        let graph = create_graph(&[(
            "data.firm",
            r#"
schema person {
    field { name = "name" type = "string" required = true }
}

person alice { name = "Alice" }
"#,
        )]);

        let params = QueryParams {
            query: "from organization".to_string(),
        };

        let result = execute(&graph, &params);

        assert!(is_success(&result));
        assert!(!get_text(&result).contains("person"));
    }

    #[test]
    fn test_query_where_string_equals() {
        let graph = create_graph(&[(
            "data.firm",
            r#"
schema person {
    field { name = "name" type = "string" required = true }
}

person alice { name = "Alice" }
person bob { name = "Bob" }
person charlie { name = "Charlie" }
"#,
        )]);

        let params = QueryParams {
            query: "from person | where name == \"Bob\"".to_string(),
        };

        let result = execute(&graph, &params);

        assert!(is_success(&result));
        let text = get_text(&result);
        assert!(text.contains("Bob"));
        assert!(!text.contains("Alice"));
        assert!(!text.contains("Charlie"));
    }

    #[test]
    fn test_query_where_string_contains() {
        let graph = create_graph(&[(
            "data.firm",
            r#"
schema person {
    field { name = "name" type = "string" required = true }
}

person alice { name = "Alice Smith" }
person bob { name = "Bob Jones" }
person charlie { name = "Charlie Smith" }
"#,
        )]);

        let params = QueryParams {
            query: "from person | where name contains \"Smith\"".to_string(),
        };

        let result = execute(&graph, &params);

        assert!(is_success(&result));
        let text = get_text(&result);
        assert!(text.contains("Alice"));
        assert!(text.contains("Charlie"));
        assert!(!text.contains("Bob"));
    }

    #[test]
    fn test_query_where_boolean() {
        let graph = create_graph(&[(
            "data.firm",
            r#"
schema task {
    field { name = "title" type = "string" required = true }
    field { name = "completed" type = "boolean" required = true }
}

task task1 { title = "First" completed = true }
task task2 { title = "Second" completed = false }
task task3 { title = "Third" completed = true }
"#,
        )]);

        let params = QueryParams {
            query: "from task | where completed == false".to_string(),
        };

        let result = execute(&graph, &params);

        assert!(is_success(&result));
        let text = get_text(&result);
        assert!(text.contains("Second"));
        assert!(!text.contains("First"));
        assert!(!text.contains("Third"));
    }

    #[test]
    fn test_query_where_integer_comparison() {
        let graph = create_graph(&[(
            "data.firm",
            r#"
schema person {
    field { name = "name" type = "string" required = true }
    field { name = "age" type = "integer" required = true }
}

person young { name = "Young" age = 20 }
person middle { name = "Middle" age = 40 }
person old { name = "Old" age = 60 }
"#,
        )]);

        let params = QueryParams {
            query: "from person | where age > 30".to_string(),
        };

        let result = execute(&graph, &params);

        assert!(is_success(&result));
        let text = get_text(&result);
        assert!(text.contains("Middle"));
        assert!(text.contains("Old"));
        assert!(!text.contains("Young"));
    }

    #[test]
    fn test_query_parse_error() {
        let graph = create_graph(&[(
            "data.firm",
            r#"
schema person {
    field { name = "name" type = "string" required = true }
}
"#,
        )]);

        let params = QueryParams {
            query: "this is not valid query syntax".to_string(),
        };

        let result = execute(&graph, &params);

        assert!(is_error(&result));
    }

    #[test]
    fn test_query_empty_query_string() {
        let graph = create_graph(&[(
            "data.firm",
            r#"
schema person {
    field { name = "name" type = "string" required = true }
}
"#,
        )]);

        let params = QueryParams {
            query: "".to_string(),
        };

        let result = execute(&graph, &params);

        assert!(is_error(&result));
    }
}
