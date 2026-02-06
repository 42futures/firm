//! Core query types for executing queries against the entity graph

use std::fmt;

use iso_currency::Currency;
use rust_decimal::Decimal;
use serde::Serialize;

use super::QueryError;
use super::filter::{CompoundFilterCondition, FieldRef};
use super::order::compare_entities_by_field;
use crate::{Entity, EntityType, FieldValue};

/// Sort direction
#[derive(Debug, Clone, PartialEq)]
#[derive(Default)]
pub enum SortDirection {
    #[default]
    Ascending,
    Descending,
}


/// Terminal aggregation that transforms the query result set
#[derive(Debug, Clone)]
pub enum Aggregation {
    /// Select specific field values from entities
    Select(Vec<FieldRef>),
    /// Count entities (None = count all, Some = count entities with field)
    Count(Option<FieldRef>),
    /// Sum a numeric field
    Sum(FieldRef),
    /// Average a numeric field
    Average(FieldRef),
    /// Median of a numeric field
    Median(FieldRef),
}

/// The result of executing a query
#[derive(Debug)]
pub enum QueryResult<'a> {
    /// Standard entity results (no aggregation)
    Entities(Vec<&'a Entity>),
    /// Aggregation result
    Aggregation(AggregationResult),
}

/// Result of an aggregation operation
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum AggregationResult {
    /// Rows of field values from a select query
    Select {
        columns: Vec<String>,
        rows: Vec<Vec<Option<FieldValue>>>,
    },
    /// A count result
    Count(usize),
    /// A sum result
    Sum(AggregateValue),
    /// An average result
    Average(f64),
    /// A median result
    Median(f64),
}

impl fmt::Display for AggregationResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AggregationResult::Count(n) => write!(f, "{}", n),
            AggregationResult::Sum(val) => write!(f, "{}", val),
            AggregationResult::Average(val) => write!(f, "{}", val),
            AggregationResult::Median(val) => write!(f, "{}", val),
            AggregationResult::Select { columns, rows } => {
                writeln!(f, "{}", columns.join("\t"))?;
                for row in rows {
                    let cells: Vec<String> = row
                        .iter()
                        .map(|v| match v {
                            Some(val) => val.to_string(),
                            None => "-".to_string(),
                        })
                        .collect();
                    writeln!(f, "{}", cells.join("\t"))?;
                }
                Ok(())
            }
        }
    }
}

/// A value produced by a numeric aggregation
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum AggregateValue {
    Integer(i64),
    Float(f64),
    Currency {
        amount: Decimal,
        currency: Currency,
    },
}

impl fmt::Display for AggregateValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AggregateValue::Integer(n) => write!(f, "{}", n),
            AggregateValue::Float(n) => write!(f, "{}", n),
            AggregateValue::Currency { amount, currency } => {
                write!(f, "{} {}", amount, currency.code())
            }
        }
    }
}

/// A query that can be executed against an entity graph
#[derive(Debug, Clone)]
pub struct Query {
    pub from: EntitySelector,
    pub operations: Vec<QueryOperation>,
    pub aggregation: Option<Aggregation>,
}

impl Query {
    /// Create a new query with a starting selector
    pub fn new(from: EntitySelector) -> Self {
        Self {
            from,
            operations: Vec::new(),
            aggregation: None,
        }
    }

    /// Add an operation to the query
    pub fn with_operation(mut self, operation: QueryOperation) -> Self {
        self.operations.push(operation);
        self
    }

    /// Set the terminal aggregation for the query
    pub fn with_aggregation(mut self, aggregation: Aggregation) -> Self {
        self.aggregation = Some(aggregation);
        self
    }

    /// Execute the query against an entity graph
    pub fn execute<'a>(
        &self,
        graph: &'a crate::graph::EntityGraph,
    ) -> Result<QueryResult<'a>, QueryError> {
        // Start by selecting entities based on the "from" clause
        let mut entities = match &self.from {
            EntitySelector::Type(entity_type) => {
                // Check if the entity type exists in the graph
                let all_types = graph.get_all_entity_types();
                if !all_types.contains(entity_type) {
                    return Err(QueryError::UnknownEntityType {
                        requested: entity_type.to_string(),
                        available: all_types.iter().map(|t| t.to_string()).collect(),
                    });
                }
                graph.list_by_type(entity_type)
            }
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
                    let mut filtered = Vec::new();
                    for e in entities {
                        if condition.matches(e)? {
                            filtered.push(e);
                        }
                    }
                    filtered
                }
                QueryOperation::Order { field, direction } => {
                    let mut entities = entities;
                    entities.sort_by(|a, b| compare_entities_by_field(a, b, field, direction));
                    entities
                }
                QueryOperation::Limit(n) => entities.into_iter().take(*n).collect(),
                QueryOperation::Related {
                    degrees,
                    entity_type,
                } => super::related::get_related_entities(
                    graph,
                    entities,
                    *degrees,
                    entity_type.as_ref(),
                ),
            };
        }

        // Apply terminal aggregation if present
        match &self.aggregation {
            None => Ok(QueryResult::Entities(entities)),
            Some(aggregation) => {
                let result = aggregation.execute(&entities)?;
                Ok(QueryResult::Aggregation(result))
            }
        }
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
    /// Filter entities by a compound condition
    Where(CompoundFilterCondition),
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

    /// Helper to extract entities from a QueryResult, panicking if it's an aggregation.
    fn unwrap_entities<'a>(result: QueryResult<'a>) -> Vec<&'a Entity> {
        match result {
            QueryResult::Entities(entities) => entities,
            QueryResult::Aggregation(_) => panic!("Expected entities, got aggregation"),
        }
    }

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

        graph
            .add_entities(vec![person1, person2, task1, task2])
            .unwrap();
        graph.build();

        graph
    }

    #[test]
    fn test_query_from_type() {
        let graph = create_test_graph();
        let query = Query::new(EntitySelector::Type(EntityType::new("person")));
        let results = unwrap_entities(query.execute(&graph).unwrap());

        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|e| e.id == EntityId::new("person1")));
        assert!(results.iter().any(|e| e.id == EntityId::new("person2")));
    }

    #[test]
    fn test_query_from_all() {
        let graph = create_test_graph();
        let query = Query::new(EntitySelector::All);
        let results = unwrap_entities(query.execute(&graph).unwrap());

        assert_eq!(results.len(), 4);
    }

    #[test]
    fn test_query_with_where() {
        let graph = create_test_graph();
        let query = Query::new(EntitySelector::Type(EntityType::new("task"))).with_operation(
            QueryOperation::Where(super::super::CompoundFilterCondition::single(
                super::super::FilterCondition::new(
                    super::super::FieldRef::Regular(FieldId::new("is_completed")),
                    super::super::FilterOperator::Equal,
                    super::super::FilterValue::Boolean(false),
                ),
            )),
        );

        let results = unwrap_entities(query.execute(&graph).unwrap());
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, EntityId::new("task2"));
    }

    #[test]
    fn test_query_with_limit() {
        let graph = create_test_graph();
        let query = Query::new(EntitySelector::All).with_operation(QueryOperation::Limit(2));

        let results = unwrap_entities(query.execute(&graph).unwrap());
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_query_with_where_and_limit() {
        let graph = create_test_graph();
        let query = Query::new(EntitySelector::Type(EntityType::new("person")))
            .with_operation(QueryOperation::Where(
                super::super::CompoundFilterCondition::single(
                    super::super::FilterCondition::new(
                        super::super::FieldRef::Regular(FieldId::new("age")),
                        super::super::FilterOperator::GreaterThan,
                        super::super::FilterValue::Integer(20),
                    ),
                ),
            ))
            .with_operation(QueryOperation::Limit(1));

        let results = unwrap_entities(query.execute(&graph).unwrap());
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_query_unknown_entity_type() {
        let graph = create_test_graph();
        // "tasks" doesn't exist, only "task" does
        let query = Query::new(EntitySelector::Type(EntityType::new("tasks")));
        let result = query.execute(&graph);

        assert!(matches!(result, Err(QueryError::UnknownEntityType { .. })));

        // Verify the error contains helpful info
        if let Err(QueryError::UnknownEntityType {
            requested,
            available,
        }) = result
        {
            assert_eq!(requested, "tasks");
            assert!(available.contains(&"task".to_string()));
            assert!(available.contains(&"person".to_string()));
        }
    }

    // --- Aggregation integration tests ---

    fn unwrap_aggregation(result: QueryResult) -> AggregationResult {
        match result {
            QueryResult::Aggregation(agg) => agg,
            QueryResult::Entities(_) => panic!("Expected aggregation, got entities"),
        }
    }

    #[test]
    fn test_query_with_count_aggregation() {
        let graph = create_test_graph();
        let query = Query::new(EntitySelector::Type(EntityType::new("person")))
            .with_aggregation(Aggregation::Count(None));
        let result = unwrap_aggregation(query.execute(&graph).unwrap());
        assert_eq!(result, AggregationResult::Count(2));
    }

    #[test]
    fn test_query_with_aggregation_after_where() {
        let graph = create_test_graph();
        let query = Query::new(EntitySelector::Type(EntityType::new("task")))
            .with_operation(QueryOperation::Where(
                super::super::CompoundFilterCondition::single(
                    super::super::FilterCondition::new(
                        super::super::FieldRef::Regular(FieldId::new("is_completed")),
                        super::super::FilterOperator::Equal,
                        super::super::FilterValue::Boolean(false),
                    ),
                ),
            ))
            .with_aggregation(Aggregation::Count(None));
        let result = unwrap_aggregation(query.execute(&graph).unwrap());
        assert_eq!(result, AggregationResult::Count(1));
    }

    #[test]
    fn test_query_without_aggregation_returns_entities() {
        let graph = create_test_graph();
        let query = Query::new(EntitySelector::Type(EntityType::new("person")));
        let result = query.execute(&graph).unwrap();
        assert!(matches!(result, QueryResult::Entities(_)));
    }
}
