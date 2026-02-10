use std::collections::HashMap;

use log::debug;
use petgraph::{Graph, graph::NodeIndex};
use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

mod access;
mod export;
mod graph_errors;
mod query;

pub use graph_errors::GraphError;
pub use petgraph::Direction;
pub use query::*;

use crate::{Entity, EntityId, EntityType, FieldId, FieldValue, ReferenceValue};

/// Defines a relationship between entities in the graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Relationship {
    EntityReference {
        from_field: FieldId,
    },
    FieldReference {
        from_field: FieldId,
        to_field: FieldId,
    },
}

/// The entity graph tracks all Firm entities and their relationships.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityGraph {
    graph: Graph<Entity, Relationship>,
    entity_map: HashMap<EntityId, NodeIndex>,
    #[serde(
        serialize_with = "serialize_entity_type_map",
        deserialize_with = "deserialize_entity_type_map"
    )]
    entity_type_map: HashMap<EntityType, Vec<NodeIndex>>,
}

impl Default for EntityGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl EntityGraph {
    /// Creates a new entity graph, ready to be populated and built.
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            entity_map: HashMap::new(),
            entity_type_map: HashMap::new(),
        }
    }

    /// Clears graph data, allowing it to be populated and built from scratch.
    pub fn clear(&mut self) {
        debug!("Clearing graph data");
        self.graph.clear();
        self.entity_map.clear();
        self.entity_type_map.clear();
    }

    /// Adds a new entity to the graph.
    /// Note: After an entity is added, the graph should be re-built.
    pub fn add_entity(&mut self, entity: Entity) -> Result<(), GraphError> {
        debug!("Adding new entity '{}' to graph", entity.id);

        if self.entity_map.contains_key(&entity.id) {
            debug!("Entity '{}' already exists, skipping add", entity.id);
            return Err(GraphError::EntityAlreadyExists(entity.id));
        }

        let node_index = self.graph.add_node(entity.clone());
        self.entity_map.insert(entity.id.clone(), node_index);

        self.entity_type_map
            .entry(entity.entity_type)
            .or_default()
            .push(node_index);

        Ok(())
    }

    /// Adds a collection of entities to the graph.
    pub fn add_entities(&mut self, entities: Vec<Entity>) -> Result<(), GraphError> {
        for entity in entities {
            self.add_entity(entity)?;
        }

        Ok(())
    }

    /// Builds relationships for all entities in the graph.
    ///
    /// Note: We always clear the edges and build from scratch.
    /// This means that it's best to add all your entities in bulk first, then build.
    /// The implementation could be improved by letting the relationships be progressively built.
    pub fn build(&mut self) {
        debug!(
            "Building relationships for graph with {} entities",
            self.graph.node_count()
        );

        self.graph.clear_edges();

        // Collect the edges to add first to avoid borrowing conflicts
        let mut edges_to_add = Vec::new();

        // Iterate through all entities in the graph
        for (from_node_index, node) in self.graph.raw_nodes().iter().enumerate() {
            let entity = &node.weight;

            // Iterate through all fields on the entity
            for (field_name, field_value) in &entity.fields {
                self.collect_relationships_from_field(
                    NodeIndex::new(from_node_index),
                    field_name,
                    field_value,
                    &mut edges_to_add,
                );
            }
        }

        // Add all the edges
        for (from_index, to_index, relationship) in edges_to_add {
            self.graph.add_edge(from_index, to_index, relationship);
        }
    }

    /// Map graph relationships from reference fields.
    /// We do this by populating an edge list which are later added to the graph.
    fn collect_relationships_from_field(
        &self,
        from_node: NodeIndex,
        field_name: &FieldId,
        field_value: &FieldValue,
        edges_to_add: &mut Vec<(NodeIndex, NodeIndex, Relationship)>,
    ) {
        match field_value {
            FieldValue::Reference(ReferenceValue::Entity(target_id)) => {
                if let Some(&to_node_index) = self.entity_map.get(target_id) {
                    let relationship = Relationship::EntityReference {
                        from_field: field_name.clone(),
                    };
                    edges_to_add.push((from_node, to_node_index, relationship));
                }
            }
            FieldValue::Reference(ReferenceValue::Field(target_entity_id, target_field_id)) => {
                if let Some(&to_node_index) = self.entity_map.get(target_entity_id) {
                    let relationship = Relationship::FieldReference {
                        from_field: field_name.clone(),
                        to_field: target_field_id.clone(),
                    };
                    edges_to_add.push((from_node, to_node_index, relationship));
                }
            }
            FieldValue::List(items) => {
                for item in items {
                    self.collect_relationships_from_field(
                        from_node,
                        field_name,
                        item,
                        edges_to_add,
                    );
                }
            }
            _ => {}
        }
    }
}

/// Custom serialization for the entity type map.
fn serialize_entity_type_map<S>(
    map: &HashMap<EntityType, Vec<NodeIndex>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut ser_map = serializer.serialize_map(Some(map.len()))?;
    for (k, v) in map {
        ser_map.serialize_entry(&k.to_string(), v)?;
    }
    ser_map.end()
}

/// Custom deserialization for the entity type map.
fn deserialize_entity_type_map<'de, D>(
    deserializer: D,
) -> Result<HashMap<EntityType, Vec<NodeIndex>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct EntityTypeMapVisitor;

    impl<'de> Visitor<'de> for EntityTypeMapVisitor {
        type Value = HashMap<EntityType, Vec<NodeIndex>>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a map with string keys")
        }

        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut map = HashMap::new();
            while let Some((key, value)) = access.next_entry::<String, Vec<NodeIndex>>()? {
                let entity_type = EntityType::from(key.as_str());
                map.insert(entity_type, value);
            }
            Ok(map)
        }
    }

    deserializer.deserialize_map(EntityTypeMapVisitor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Entity, EntityId, EntityType, FieldValue};

    // Helper functions
    fn create_organization(id: &str, name: &str) -> Entity {
        Entity::new(EntityId::new(id), EntityType::new("organization"))
            .with_field(FieldId::new("name"), name)
    }

    fn create_person(id: &str, name: &str) -> Entity {
        Entity::new(EntityId::new(id), EntityType::new("person"))
            .with_field(FieldId::new("name"), name)
    }

    fn create_person_with_employer(id: &str, name: &str, employer_id: &str) -> Entity {
        Entity::new(EntityId::new(id), EntityType::new("person"))
            .with_field(FieldId::new("name"), name)
            .with_field(
                FieldId::new("employer"),
                FieldValue::Reference(ReferenceValue::Entity(EntityId::new(employer_id))),
            )
    }

    fn setup_basic_graph() -> (EntityGraph, Entity, Entity) {
        let graph = EntityGraph::new();
        let organization = create_organization("megacorp", "MegaCorp Inc.");
        let person = create_person_with_employer("john_doe", "John Doe", "megacorp");
        (graph, organization, person)
    }

    fn assert_basic_graph_structure(graph: &EntityGraph) {
        assert!(graph.entity_map.contains_key(&EntityId::new("megacorp")));
        assert!(graph.entity_map.contains_key(&EntityId::new("john_doe")));
        assert_eq!(graph.graph.edge_count(), 1);
    }

    #[test]
    fn test_build_graph_iteratively() {
        let (mut graph, organization, person) = setup_basic_graph();

        graph.add_entity(organization).unwrap();
        graph.add_entity(person).unwrap();
        graph.build();

        assert_basic_graph_structure(&graph);
    }

    #[test]
    fn test_build_graph_bulk() {
        let (mut graph, organization, person) = setup_basic_graph();

        graph.add_entities(vec![organization, person]).unwrap();
        graph.build();

        assert_basic_graph_structure(&graph);
    }

    #[test]
    fn test_add_duplicate_entity_returns_error() {
        let mut graph = EntityGraph::new();
        let entity1 = create_person("duplicate_id", "First Entity");

        assert!(graph.add_entity(entity1).is_ok());

        let entity2 = create_organization("duplicate_id", "Second Entity");
        let result = graph.add_entity(entity2);

        assert!(result.is_err());
        if let Err(GraphError::EntityAlreadyExists(entity_id)) = result {
            assert_eq!(entity_id, EntityId::new("duplicate_id"));
        }

        assert_eq!(graph.entity_map.len(), 1);
        assert_eq!(graph.graph.node_count(), 1);
    }

    #[test]
    fn test_add_entities_stops_on_duplicate() {
        let mut graph = EntityGraph::new();
        graph.add_entity(create_person("first", "First")).unwrap();

        let entities = vec![
            create_person("second", "Second"),
            create_organization("first", "Duplicate"),
            create_person("third", "Third"),
        ];

        assert!(graph.add_entities(entities).is_err());
        assert_eq!(graph.entity_map.len(), 2);
        assert!(graph.entity_map.contains_key(&EntityId::new("first")));
        assert!(graph.entity_map.contains_key(&EntityId::new("second")));
        assert!(!graph.entity_map.contains_key(&EntityId::new("third")));
    }

    #[test]
    fn test_list_field_references() {
        let mut graph = EntityGraph::new();

        let person1 =
            create_person("john", "John Doe").with_field(FieldId::new("email"), "john@example.com");
        let person2 = create_person("jane", "Jane Smith")
            .with_field(FieldId::new("email"), "jane@example.com");

        let email_campaign = create_organization("campaign1", "Newsletter Campaign").with_field(
            FieldId::new("recipient_emails"),
            FieldValue::List(vec![
                FieldValue::Reference(ReferenceValue::Field(
                    EntityId::new("john"),
                    FieldId::new("email"),
                )),
                FieldValue::Reference(ReferenceValue::Field(
                    EntityId::new("jane"),
                    FieldId::new("email"),
                )),
            ]),
        );

        graph
            .add_entities(vec![person1, person2, email_campaign])
            .unwrap();
        graph.build();

        assert_eq!(graph.graph.edge_count(), 2);
        assert_eq!(graph.graph.node_count(), 3);
    }

    fn test_serialization_roundtrip(graph: &EntityGraph) -> EntityGraph {
        let serialized = serde_json::to_string(graph).unwrap();
        serde_json::from_str(&serialized).unwrap()
    }

    #[test]
    fn test_graph_is_serializable() {
        let (mut graph, organization, person) = setup_basic_graph();
        graph.add_entities(vec![organization, person]).unwrap();
        graph.build();

        let deserialized = test_serialization_roundtrip(&graph);
        assert_basic_graph_structure(&deserialized);
    }

    #[test]
    fn test_graph_with_currency_field_serialization() {
        use iso_currency::Currency;
        use rust_decimal::Decimal;

        let mut graph = EntityGraph::new();
        let expected_amount = Decimal::new(12345, 2);
        let expected_currency = Currency::USD;

        let entity = create_organization("test_entity", "Test").with_field(
            FieldId::new("price"),
            FieldValue::Currency {
                amount: expected_amount,
                currency: expected_currency,
            },
        );

        graph.add_entity(entity).unwrap();
        graph.build();

        let deserialized = test_serialization_roundtrip(&graph);
        let node_idx = deserialized.entity_map[&EntityId::new("test_entity")];
        let node = &deserialized.graph[node_idx];

        if let Some(FieldValue::Currency { amount, currency }) =
            node.get_field(&FieldId::new("price"))
        {
            assert_eq!(*amount, expected_amount);
            assert_eq!(*currency, expected_currency);
        } else {
            panic!("Currency field not preserved");
        }
    }

    #[test]
    fn test_graph_with_datetime_field_serialization() {
        use chrono::{FixedOffset, TimeZone};

        let mut graph = EntityGraph::new();
        let offset = FixedOffset::east_opt(5 * 3600).unwrap();
        let expected_dt = offset.with_ymd_and_hms(2023, 1, 1, 12, 0, 0).unwrap();

        let entity = create_organization("test_entity", "Test").with_field(
            FieldId::new("created_at"),
            FieldValue::DateTime(expected_dt),
        );

        graph.add_entity(entity).unwrap();
        graph.build();

        let deserialized = test_serialization_roundtrip(&graph);
        let node_idx = deserialized.entity_map[&EntityId::new("test_entity")];
        let node = &deserialized.graph[node_idx];

        assert_eq!(
            node.get_field(&FieldId::new("created_at")),
            Some(&FieldValue::DateTime(expected_dt))
        );
    }

    #[test]
    fn test_graph_list_serialization() {
        let mut graph = EntityGraph::new();
        let expected_tags = vec![
            FieldValue::String("tag1".to_string()),
            FieldValue::String("tag2".to_string()),
            FieldValue::String("tag3".to_string()),
        ];

        let entity = create_organization("test_entity", "Test").with_field(
            FieldId::new("tags"),
            FieldValue::List(expected_tags.clone()),
        );

        graph.add_entity(entity).unwrap();
        graph.build();

        let deserialized = test_serialization_roundtrip(&graph);
        let node_idx = deserialized.entity_map[&EntityId::new("test_entity")];
        let node = &deserialized.graph[node_idx];

        if let Some(FieldValue::List(items)) = node.get_field(&FieldId::new("tags")) {
            assert_eq!(*items, expected_tags);
        } else {
            panic!("List field not preserved");
        }
    }

    #[test]
    fn test_graph_with_field_reference_serialization() {
        let mut graph = EntityGraph::new();
        let target_entity = create_person("target", "Target");
        let source_entity = create_organization("source", "Source").with_field(
            FieldId::new("ref_field"),
            FieldValue::Reference(ReferenceValue::Field(
                EntityId::new("target"),
                FieldId::new("name"),
            )),
        );

        graph
            .add_entities(vec![target_entity, source_entity])
            .unwrap();
        graph.build();

        let deserialized = test_serialization_roundtrip(&graph);
        assert_eq!(deserialized.graph.node_count(), 2);
        assert_eq!(deserialized.graph.edge_count(), 1);

        let source_idx = deserialized.entity_map[&EntityId::new("source")];
        let source_node = &deserialized.graph[source_idx];

        assert_eq!(
            source_node.get_field(&FieldId::new("ref_field")),
            Some(&FieldValue::Reference(ReferenceValue::Field(
                EntityId::new("target"),
                FieldId::new("name")
            )))
        );
    }

    #[test]
    fn test_entity_id_hashmap_serialization() {
        use std::collections::HashMap;

        let mut map = HashMap::new();
        map.insert(EntityId::new("test1"), 42);
        map.insert(EntityId::new("test2"), 84);

        let serialized = serde_json::to_string(&map).unwrap();
        println!("EntityId HashMap serialized: {}", serialized);

        let deserialized: HashMap<EntityId, i32> = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.len(), 2);
        assert_eq!(deserialized[&EntityId::new("test1")], 42);
        assert_eq!(deserialized[&EntityId::new("test2")], 84);
    }
}
