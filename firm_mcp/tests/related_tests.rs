mod helpers;

use firm_core::graph::EntityGraph;
use firm_mcp::tools::related::{RelatedDirection, RelatedParams, execute};
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
    fn test_related_both_directions() {
        let graph = create_graph(&[(
            "data.firm",
            r#"
schema person {
    field { name = "name" type = "string" required = true }
    field { name = "manager" type = "reference" required = false }
}

person alice { name = "Alice" }
person bob { name = "Bob" manager = person.alice }
person charlie { name = "Charlie" manager = person.alice }
"#,
        )]);

        // Alice has incoming refs from Bob and Charlie
        let params = RelatedParams {
            r#type: "person".to_string(),
            id: "alice".to_string(),
            direction: None,
        };

        let result = execute(&graph, &params);

        assert!(is_success(&result));
        let text = get_text(&result);
        assert!(text.contains("person.bob"));
        assert!(text.contains("person.charlie"));
    }

    #[test]
    fn test_related_incoming_only() {
        let graph = create_graph(&[(
            "data.firm",
            r#"
schema person {
    field { name = "name" type = "string" required = true }
    field { name = "manager" type = "reference" required = false }
}

person alice { name = "Alice" }
person bob { name = "Bob" manager = person.alice }
"#,
        )]);

        // Alice has incoming ref from Bob
        let params = RelatedParams {
            r#type: "person".to_string(),
            id: "alice".to_string(),
            direction: Some(RelatedDirection::Incoming),
        };

        let result = execute(&graph, &params);

        assert!(is_success(&result));
        let text = get_text(&result);
        assert!(text.contains("person.bob"));
    }

    #[test]
    fn test_related_outgoing_only() {
        let graph = create_graph(&[(
            "data.firm",
            r#"
schema person {
    field { name = "name" type = "string" required = true }
    field { name = "manager" type = "reference" required = false }
}

person alice { name = "Alice" }
person bob { name = "Bob" manager = person.alice }
"#,
        )]);

        // Bob has outgoing ref to Alice
        let params = RelatedParams {
            r#type: "person".to_string(),
            id: "bob".to_string(),
            direction: Some(RelatedDirection::Outgoing),
        };

        let result = execute(&graph, &params);

        assert!(is_success(&result));
        let text = get_text(&result);
        assert!(text.contains("person.alice"));
    }

    #[test]
    fn test_related_no_relationships() {
        let graph = create_graph(&[(
            "data.firm",
            r#"
schema person {
    field { name = "name" type = "string" required = true }
}

person alice { name = "Alice" }
person bob { name = "Bob" }
"#,
        )]);

        let params = RelatedParams {
            r#type: "person".to_string(),
            id: "alice".to_string(),
            direction: None,
        };

        let result = execute(&graph, &params);

        assert!(is_success(&result));
        assert!(get_text(&result).contains("No related entities"));
    }

    #[test]
    fn test_related_entity_not_found() {
        let graph = create_graph(&[(
            "data.firm",
            r#"
schema person {
    field { name = "name" type = "string" required = true }
}

person alice { name = "Alice" }
"#,
        )]);

        let params = RelatedParams {
            r#type: "person".to_string(),
            id: "nonexistent".to_string(),
            direction: None,
        };

        let result = execute(&graph, &params);

        assert!(is_error(&result));
        assert!(get_text(&result).contains("not found"));
    }

    #[test]
    fn test_related_cross_type() {
        let graph = create_graph(&[(
            "data.firm",
            r#"
schema person {
    field { name = "name" type = "string" required = true }
    field { name = "employer" type = "reference" required = false }
}

schema organization {
    field { name = "name" type = "string" required = true }
}

organization acme { name = "ACME Corp" }
person alice { name = "Alice" employer = organization.acme }
person bob { name = "Bob" employer = organization.acme }
"#,
        )]);

        // ACME has incoming refs from Alice and Bob
        let params = RelatedParams {
            r#type: "organization".to_string(),
            id: "acme".to_string(),
            direction: Some(RelatedDirection::Incoming),
        };

        let result = execute(&graph, &params);

        assert!(is_success(&result));
        let text = get_text(&result);
        assert!(text.contains("person.alice"));
        assert!(text.contains("person.bob"));
    }

    #[test]
    fn test_related_chain() {
        let graph = create_graph(&[(
            "data.firm",
            r#"
schema person {
    field { name = "name" type = "string" required = true }
    field { name = "manager" type = "reference" required = false }
}

person ceo { name = "CEO" }
person vp { name = "VP" manager = person.ceo }
person manager { name = "Manager" manager = person.vp }
person employee { name = "Employee" manager = person.manager }
"#,
        )]);

        // VP has outgoing to CEO, incoming from Manager
        let params = RelatedParams {
            r#type: "person".to_string(),
            id: "vp".to_string(),
            direction: None,
        };

        let result = execute(&graph, &params);

        assert!(is_success(&result));
        let text = get_text(&result);
        assert!(text.contains("person.ceo")); // outgoing
        assert!(text.contains("person.manager")); // incoming
        // Should NOT include employee (that's 2 hops away)
        assert!(!text.contains("person.employee"));
    }

    #[test]
    fn test_related_incoming_no_refs() {
        let graph = create_graph(&[(
            "data.firm",
            r#"
schema person {
    field { name = "name" type = "string" required = true }
    field { name = "manager" type = "reference" required = false }
}

person alice { name = "Alice" }
person bob { name = "Bob" manager = person.alice }
"#,
        )]);

        // Bob has no incoming refs (only outgoing)
        let params = RelatedParams {
            r#type: "person".to_string(),
            id: "bob".to_string(),
            direction: Some(RelatedDirection::Incoming),
        };

        let result = execute(&graph, &params);

        assert!(is_success(&result));
        assert!(get_text(&result).contains("No related entities"));
    }
}
