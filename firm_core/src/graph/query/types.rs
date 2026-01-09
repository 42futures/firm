//! Core query types for executing queries against the entity graph

use crate::{Entity, EntityType};
use super::filter::FilterCondition;
use super::order::compare_entities_by_field;

/// Sort direction
#[derive(Debug, Clone, PartialEq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl Default for SortDirection {
    fn default() -> Self {
        SortDirection::Ascending
    }
}

/// A query that can be executed against an entity graph
#[derive(Debug, Clone)]
pub struct Query {
    pub from: EntitySelector,
    pub operations: Vec<QueryOperation>,
}

impl Query {
    /// Create a new query with a starting selector
    pub fn new(from: EntitySelector) -> Self {
        Self {
            from,
            operations: Vec::new(),
        }
    }

    /// Add an operation to the query
    pub fn with_operation(mut self, operation: QueryOperation) -> Self {
        self.operations.push(operation);
        self
    }

    /// Execute the query against an entity graph
    pub fn execute<'a>(&self, graph: &'a crate::graph::EntityGraph) -> Vec<&'a Entity> {
        // Start by selecting entities based on the "from" clause
        let mut entities = match &self.from {
            EntitySelector::Type(entity_type) => graph.list_by_type(entity_type),
            EntitySelector::All => {
                // Get all entity types and collect all entities
                let all_types = graph.get_all_entity_types();
                all_types
                    .iter()
                    .flat_map(|entity_type| graph.list_by_type(entity_type))
                    .collect()
            }
        };

        // Apply each operation in sequence
        for operation in &self.operations {
            entities = match operation {
                QueryOperation::Where(condition) => {
                    entities.into_iter().filter(|e| condition.matches(e)).collect()
                }
                QueryOperation::Order { field, direction } => {
                    let mut entities = entities;
                    entities.sort_by(|a, b| compare_entities_by_field(a, b, field, direction));
                    entities
                }
                QueryOperation::Limit(n) => {
                    entities.into_iter().take(*n).collect()
                }
                QueryOperation::Related { degrees, entity_type } => {
                    super::related::get_related_entities(
                        graph,
                        entities,
                        *degrees,
                        entity_type.as_ref(),
                    )
                }
            };
        }

        entities
    }
}

/// Selects the starting set of entities
#[derive(Debug, Clone, PartialEq)]
pub enum EntitySelector {
    /// Select entities of a specific type
    Type(EntityType),
    /// Select all entities (wildcard)
    All,
}

/// Operations that can be applied to entity collections
#[derive(Debug, Clone)]
pub enum QueryOperation {
    /// Filter entities by a condition
    Where(FilterCondition),
    /// Traverse to related entities
    Related {
        degrees: usize,
        entity_type: Option<EntityType>,
    },
    /// Sort entities by a field (or metadata)
    Order {
        field: super::filter::FieldRef,
        direction: SortDirection,
    },
    /// Limit the number of results
    Limit(usize),
}

/// Compare two entities by a specific field for sorting
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Entity, EntityId, EntityType, FieldId, FieldValue};

    fn create_test_graph() -> crate::graph::EntityGraph {
        let mut graph = crate::graph::EntityGraph::new();

        let person1 = Entity::new(EntityId::new("person1"), EntityType::new("person"))
            .with_field(FieldId::new("name"), "Alice")
            .with_field(FieldId::new("age"), FieldValue::Integer(30));

        let person2 = Entity::new(EntityId::new("person2"), EntityType::new("person"))
            .with_field(FieldId::new("name"), "Bob")
            .with_field(FieldId::new("age"), FieldValue::Integer(25));

        let task1 = Entity::new(EntityId::new("task1"), EntityType::new("task"))
            .with_field(FieldId::new("title"), "Important Task")
            .with_field(FieldId::new("is_completed"), true);

        let task2 = Entity::new(EntityId::new("task2"), EntityType::new("task"))
            .with_field(FieldId::new("title"), "Pending Task")
            .with_field(FieldId::new("is_completed"), false);

        graph.add_entities(vec![person1, person2, task1, task2]).unwrap();
        graph.build();

        graph
    }

    #[test]
    fn test_query_from_type() {
        let graph = create_test_graph();
        let query = Query::new(EntitySelector::Type(EntityType::new("person")));
        let results = query.execute(&graph);

        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|e| e.id == EntityId::new("person1")));
        assert!(results.iter().any(|e| e.id == EntityId::new("person2")));
    }

    #[test]
    fn test_query_from_all() {
        let graph = create_test_graph();
        let query = Query::new(EntitySelector::All);
        let results = query.execute(&graph);

        assert_eq!(results.len(), 4);
    }

    #[test]
    fn test_query_with_where() {
        let graph = create_test_graph();
        let query = Query::new(EntitySelector::Type(EntityType::new("task")))
            .with_operation(QueryOperation::Where(
                super::super::FilterCondition::new(
                    super::super::FieldRef::Regular(FieldId::new("is_completed")),
                    super::super::FilterOperator::Equal,
                    super::super::FilterValue::Boolean(false),
                )
            ));

        let results = query.execute(&graph);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, EntityId::new("task2"));
    }

    #[test]
    fn test_query_with_limit() {
        let graph = create_test_graph();
        let query = Query::new(EntitySelector::All)
            .with_operation(QueryOperation::Limit(2));

        let results = query.execute(&graph);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_query_with_where_and_limit() {
        let graph = create_test_graph();
        let query = Query::new(EntitySelector::Type(EntityType::new("person")))
            .with_operation(QueryOperation::Where(
                super::super::FilterCondition::new(
                    super::super::FieldRef::Regular(FieldId::new("age")),
                    super::super::FilterOperator::GreaterThan,
                    super::super::FilterValue::Integer(20),
                )
            ))
            .with_operation(QueryOperation::Limit(1));

        let results = query.execute(&graph);
        assert_eq!(results.len(), 1);
    }
}
