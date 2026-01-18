//! Related entity traversal for queries

use crate::graph::EntityGraph;
use crate::{Entity, EntityId, EntityType};
use std::collections::HashSet;

const MAX_DEGREES: usize = 5;

/// Get related entities by traversing the graph up to N degrees
///
/// This function starts with a set of entities and traverses relationships
/// up to the specified number of degrees. At each level, it finds all entities
/// related to the current set, deduplicates them, and continues to the next level.
///
/// # Arguments
/// * `graph` - The entity graph to traverse
/// * `starting_entities` - The initial set of entities to start from
/// * `degrees` - Number of relationship hops to traverse (max 3)
/// * `entity_type_filter` - Optional filter to only return entities of a specific type
///
/// # Returns
/// A deduplicated vector of all entities found within the specified degrees,
/// including the starting entities.
pub fn get_related_entities<'a>(
    graph: &'a EntityGraph,
    starting_entities: Vec<&'a Entity>,
    degrees: usize,
    entity_type_filter: Option<&EntityType>,
) -> Vec<&'a Entity> {
    // Cap degrees at MAX_DEGREES
    let degrees = degrees.min(MAX_DEGREES);

    if degrees < 1 {
        return starting_entities;
    }

    // Track all entities we've seen (including starting entities)
    let mut all_entities: HashSet<&EntityId> = starting_entities.iter().map(|e| &e.id).collect();

    // Current level starts with the starting entities
    let mut current_level: HashSet<&EntityId> = starting_entities.iter().map(|e| &e.id).collect();

    // Traverse N degrees
    for _ in 0..degrees {
        let mut next_level = HashSet::new();

        // For each entity in the current level, get its related entities
        for entity_id in &current_level {
            if let Some(related) = graph.get_related(entity_id, None) {
                for entity in related {
                    // Add to next level for further traversal
                    next_level.insert(&entity.id);
                    // Add to overall collection
                    all_entities.insert(&entity.id);
                }
            }
        }

        // Move to the next level
        current_level = next_level;

        // If no new entities were found, we can stop early
        if current_level.is_empty() {
            break;
        }
    }

    // Convert entity IDs back to entity references
    let mut result: Vec<&Entity> = all_entities
        .iter()
        .filter_map(|id| graph.get_entity(id))
        .collect();

    // Apply entity type filter if specified
    if let Some(filter_type) = entity_type_filter {
        result.retain(|e| &e.entity_type == filter_type);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EntityId, EntityType, FieldId, FieldValue, ReferenceValue};

    fn create_test_graph_linear() -> EntityGraph {
        let mut graph = EntityGraph::new();

        // Create a linear chain: person -> task -> project
        let person = Entity::new(EntityId::new("person1"), EntityType::new("person"))
            .with_field(FieldId::new("name"), "John Doe");

        let task = Entity::new(EntityId::new("task1"), EntityType::new("task"))
            .with_field(FieldId::new("title"), "Task 1")
            .with_field(
                FieldId::new("assignee"),
                FieldValue::Reference(ReferenceValue::Entity(EntityId::new("person1"))),
            );

        let project = Entity::new(EntityId::new("project1"), EntityType::new("project"))
            .with_field(FieldId::new("name"), "Project 1")
            .with_field(
                FieldId::new("task"),
                FieldValue::Reference(ReferenceValue::Entity(EntityId::new("task1"))),
            );

        graph.add_entities(vec![person, task, project]).unwrap();
        graph.build();

        graph
    }

    fn create_test_graph_complex() -> EntityGraph {
        let mut graph = EntityGraph::new();

        // Create a more complex graph:
        // person1 -> task1 -> project1
        //         -> task2 -> project1
        // person2 -> task2

        let person1 = Entity::new(EntityId::new("person1"), EntityType::new("person"))
            .with_field(FieldId::new("name"), "John Doe");

        let person2 = Entity::new(EntityId::new("person2"), EntityType::new("person"))
            .with_field(FieldId::new("name"), "Jane Smith");

        let task1 = Entity::new(EntityId::new("task1"), EntityType::new("task"))
            .with_field(FieldId::new("title"), "Task 1")
            .with_field(
                FieldId::new("assignee"),
                FieldValue::Reference(ReferenceValue::Entity(EntityId::new("person1"))),
            );

        let task2 = Entity::new(EntityId::new("task2"), EntityType::new("task"))
            .with_field(FieldId::new("title"), "Task 2")
            .with_field(
                FieldId::new("assignee"),
                FieldValue::Reference(ReferenceValue::Entity(EntityId::new("person1"))),
            )
            .with_field(
                FieldId::new("reviewer"),
                FieldValue::Reference(ReferenceValue::Entity(EntityId::new("person2"))),
            );

        let project1 = Entity::new(EntityId::new("project1"), EntityType::new("project"))
            .with_field(FieldId::new("name"), "Project 1")
            .with_field(
                FieldId::new("task1"),
                FieldValue::Reference(ReferenceValue::Entity(EntityId::new("task1"))),
            )
            .with_field(
                FieldId::new("task2"),
                FieldValue::Reference(ReferenceValue::Entity(EntityId::new("task2"))),
            );

        graph
            .add_entities(vec![person1, person2, task1, task2, project1])
            .unwrap();
        graph.build();

        graph
    }

    #[test]
    fn test_related_zero_degrees() {
        let graph = create_test_graph_linear();
        let person = graph.get_entity(&EntityId::new("person1")).unwrap();

        let result = get_related_entities(&graph, vec![person], 0, None);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, EntityId::new("person1"));
    }

    #[test]
    fn test_related_one_degree() {
        let graph = create_test_graph_linear();
        let person = graph.get_entity(&EntityId::new("person1")).unwrap();

        let result = get_related_entities(&graph, vec![person], 1, None);

        // Should include person1 and task1
        assert_eq!(result.len(), 2);
        let ids: Vec<&EntityId> = result.iter().map(|e| &e.id).collect();
        assert!(ids.contains(&&EntityId::new("person1")));
        assert!(ids.contains(&&EntityId::new("task1")));
    }

    #[test]
    fn test_related_two_degrees() {
        let graph = create_test_graph_linear();
        let person = graph.get_entity(&EntityId::new("person1")).unwrap();

        let result = get_related_entities(&graph, vec![person], 2, None);

        // Should include person1, task1, and project1
        assert_eq!(result.len(), 3);
        let ids: Vec<&EntityId> = result.iter().map(|e| &e.id).collect();
        assert!(ids.contains(&&EntityId::new("person1")));
        assert!(ids.contains(&&EntityId::new("task1")));
        assert!(ids.contains(&&EntityId::new("project1")));
    }

    #[test]
    fn test_related_with_entity_type_filter() {
        let graph = create_test_graph_linear();
        let person = graph.get_entity(&EntityId::new("person1")).unwrap();

        let result = get_related_entities(&graph, vec![person], 2, Some(&EntityType::new("task")));

        // Should only include task1
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, EntityId::new("task1"));
    }

    #[test]
    fn test_related_complex_graph() {
        let graph = create_test_graph_complex();
        let person1 = graph.get_entity(&EntityId::new("person1")).unwrap();

        let result = get_related_entities(&graph, vec![person1], 2, None);

        // person1 -> task1, task2
        // task1 -> project1
        // task2 -> project1, person2
        // Should include: person1, task1, task2, project1, person2
        assert_eq!(result.len(), 5);
        let ids: Vec<&EntityId> = result.iter().map(|e| &e.id).collect();
        assert!(ids.contains(&&EntityId::new("person1")));
        assert!(ids.contains(&&EntityId::new("person2")));
        assert!(ids.contains(&&EntityId::new("task1")));
        assert!(ids.contains(&&EntityId::new("task2")));
        assert!(ids.contains(&&EntityId::new("project1")));
    }

    #[test]
    fn test_related_multiple_starting_entities() {
        let graph = create_test_graph_complex();
        let person1 = graph.get_entity(&EntityId::new("person1")).unwrap();
        let person2 = graph.get_entity(&EntityId::new("person2")).unwrap();

        let result = get_related_entities(&graph, vec![person1, person2], 1, None);

        // person1 -> task1, task2
        // person2 -> task2
        // Should include: person1, person2, task1, task2
        assert_eq!(result.len(), 4);
        let ids: Vec<&EntityId> = result.iter().map(|e| &e.id).collect();
        assert!(ids.contains(&&EntityId::new("person1")));
        assert!(ids.contains(&&EntityId::new("person2")));
        assert!(ids.contains(&&EntityId::new("task1")));
        assert!(ids.contains(&&EntityId::new("task2")));
    }

    #[test]
    fn test_related_with_type_filter_no_matches() {
        let graph = create_test_graph_linear();
        let person = graph.get_entity(&EntityId::new("person1")).unwrap();

        let result = get_related_entities(
            &graph,
            vec![person],
            2,
            Some(&EntityType::new("organization")),
        );

        // No organizations in the graph
        assert_eq!(result.len(), 0);
    }
}
