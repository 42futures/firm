use log::debug;
use petgraph::{Direction, visit::EdgeRef};

use super::{EntityGraph, GraphError, Relationship};
use crate::{Entity, EntityId, EntityType, FieldId, FieldValue, ReferenceValue};

use std::collections::HashSet;

impl EntityGraph {
    /// Gets an entity in the graph by its ID.
    pub fn get_entity(&self, id: &EntityId) -> Option<&Entity> {
        debug!("Looking up entity '{}'", id);

        self.entity_map
            .get(id)
            .and_then(|&node_index| self.graph.node_weight(node_index))
    }

    /// Resolves an entity reference to the actual entity.
    pub fn resolve_entity_reference(
        &self,
        field_value: &FieldValue,
    ) -> Result<&Entity, GraphError> {
        debug!("Resolving entity reference: {:?}", field_value);

        match field_value {
            FieldValue::Reference(ReferenceValue::Entity(entity_id)) => self
                .get_entity(entity_id)
                .ok_or_else(|| GraphError::EntityNotFound(entity_id.clone())),
            _ => Err(GraphError::NotAnEntityReference),
        }
    }

    /// Resolves a field reference to the actual field value.
    pub fn resolve_field_reference(
        &self,
        field_value: &FieldValue,
    ) -> Result<&FieldValue, GraphError> {
        debug!("Resolving field reference: {:?}", field_value);

        match field_value {
            FieldValue::Reference(ReferenceValue::Field(entity_id, field_id)) => {
                self.search_field_reference(entity_id, field_id, 10, &mut HashSet::new())
            }
            _ => Err(GraphError::NotAFieldReference),
        }
    }

    /// Gets a collection of all entity types present.
    pub fn get_all_entity_types(&self) -> Vec<EntityType> {
        self.entity_type_map.keys().cloned().collect()
    }

    /// Gets all entities of a specific type.
    pub fn list_by_type(&self, entity_type: &EntityType) -> Vec<&Entity> {
        match self.entity_type_map.get(entity_type) {
            Some(nodes) => nodes
                .iter()
                .filter_map(|&node_index| self.graph.node_weight(node_index))
                .collect(),
            None => Vec::new(),
        }
    }

    /// Gets all entities that references an entity ID.
    ///
    /// Edges in the graph are directed, and here we can choose if we want only
    /// incoming references, outgoing references or both.
    pub fn get_related(&self, id: &EntityId, direction: Option<Direction>) -> Option<Vec<&Entity>> {
        match self.entity_map.get(id) {
            Some(node_index) => {
                let mut entities: Vec<&Entity> = match direction {
                    Some(Direction::Outgoing) => self
                        .graph
                        .edges_directed(node_index.clone(), Direction::Outgoing)
                        .map(|edge| &self.graph[edge.target()])
                        .collect(),
                    Some(Direction::Incoming) => self
                        .graph
                        .edges_directed(node_index.clone(), Direction::Incoming)
                        .map(|edge| &self.graph[edge.source()])
                        .collect(),
                    None => {
                        let mut all_entities = Vec::new();

                        // Collect targets of outgoing edges
                        all_entities.extend(
                            self.graph
                                .edges_directed(node_index.clone(), Direction::Outgoing)
                                .map(|edge| &self.graph[edge.target()]),
                        );

                        // Collect sources of incoming edges
                        all_entities.extend(
                            self.graph
                                .edges_directed(node_index.clone(), Direction::Incoming)
                                .map(|edge| &self.graph[edge.source()]),
                        );

                        all_entities
                    }
                };

                entities.sort_by_key(|entity| &entity.id);
                entities.dedup_by_key(|entity| &entity.id);

                Some(entities)
            }
            None => None,
        }
    }

    /// Searches for a field reference on a given entity by traversing the graph
    fn search_field_reference(
        &self,
        entity_id: &EntityId,
        field_id: &FieldId,
        max_depth: usize,
        visited: &mut HashSet<(EntityId, FieldId)>,
    ) -> Result<&FieldValue, GraphError> {
        if max_depth == 0 {
            debug!(
                "Max depth exceeded for field reference: {}.{}",
                entity_id, field_id
            );

            return Err(GraphError::MaxDepthExceeded);
        }

        // Check for cycles
        let reference_key = (entity_id.clone(), field_id.clone());
        if visited.contains(&reference_key) {
            debug!("Cyclic reference detected: {}.{}", entity_id, field_id);
            return Err(GraphError::CyclicReference);
        }
        visited.insert(reference_key);

        // Get entity
        let entity = self
            .get_entity(entity_id)
            .ok_or_else(|| GraphError::EntityNotFound(entity_id.clone()))?;

        // Get field
        let field = entity
            .get_field(field_id)
            .ok_or_else(|| GraphError::FieldNotFound(entity_id.clone(), field_id.clone()))?;

        // If it's another field reference, resolve it
        match field {
            FieldValue::Reference(ReferenceValue::Field(target_entity_id, target_field_id)) => {
                // Use graph traversal to find the target
                let source_node = self
                    .entity_map
                    .get(entity_id)
                    .ok_or_else(|| GraphError::EntityNotFound(entity_id.clone()))?;
                let target_node = self
                    .entity_map
                    .get(target_entity_id)
                    .ok_or_else(|| GraphError::EntityNotFound(target_entity_id.clone()))?;

                // Check if there's a field reference edge between these nodes
                let mut edge_found = false;
                for edge in self.graph.edges_connecting(*source_node, *target_node) {
                    if let Relationship::FieldReference {
                        from_field,
                        to_field,
                    } = edge.weight()
                    {
                        if from_field == field_id && to_field == target_field_id {
                            edge_found = true;
                            break;
                        }
                    }
                }

                if edge_found {
                    self.search_field_reference(
                        target_entity_id,
                        target_field_id,
                        max_depth - 1,
                        visited,
                    )
                } else {
                    Err(GraphError::GraphNotBuilt)
                }
            }
            _ => Ok(field),
        }
    }
}

impl FieldValue {
    /// Convenience method to resolve entity references directly on field values.
    pub fn resolve_entity_reference<'a>(
        &'a self,
        graph: &'a EntityGraph,
    ) -> Result<&'a Entity, GraphError> {
        graph.resolve_entity_reference(self)
    }

    /// Convenience method to resolve field references directly on field values.
    pub fn resolve_field_reference<'a>(
        &'a self,
        graph: &'a EntityGraph,
    ) -> Result<&'a FieldValue, GraphError> {
        graph.resolve_field_reference(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EntityType, FieldId};

    #[test]
    fn test_get_entity_by_id() {
        let mut graph = EntityGraph::new();

        let organization = Entity::new(EntityId::new("megacorp"), EntityType::new("organization"))
            .with_field(FieldId::new("name"), "MegaCorp Inc.");

        let person = Entity::new(EntityId::new("john_doe"), EntityType::new("person"))
            .with_field(FieldId::new("name"), "John Doe");

        graph
            .add_entities(vec![organization.clone(), person.clone()])
            .unwrap();

        // Test existing entities
        let retrieved_organization = graph.get_entity(&EntityId::new("megacorp"));
        assert!(retrieved_organization.is_some());
        assert_eq!(
            retrieved_organization.unwrap().id,
            EntityId::new("megacorp")
        );

        let retrieved_person = graph.get_entity(&EntityId::new("john_doe"));
        assert!(retrieved_person.is_some());
        assert_eq!(retrieved_person.unwrap().id, EntityId::new("john_doe"));

        // Test non-existing entity
        let non_existing = graph.get_entity(&EntityId::new("non_existing"));
        assert!(non_existing.is_none());
    }

    #[test]
    fn test_resolve_entity_reference_from_graph() {
        let mut graph = EntityGraph::new();

        let organization = Entity::new(EntityId::new("megacorp"), EntityType::new("organization"))
            .with_field(FieldId::new("name"), "MegaCorp Inc.");

        graph.add_entity(organization).unwrap();

        // Test valid entity reference
        let entity_ref = FieldValue::Reference(ReferenceValue::Entity(EntityId::new("megacorp")));
        let resolved = graph.resolve_entity_reference(&entity_ref);
        assert!(resolved.is_ok());
        assert_eq!(resolved.unwrap().id, EntityId::new("megacorp"));

        // Test invalid entity reference
        let invalid_ref =
            FieldValue::Reference(ReferenceValue::Entity(EntityId::new("non_existing")));
        let resolved_invalid = graph.resolve_entity_reference(&invalid_ref);
        assert!(resolved_invalid.is_err());
        assert_eq!(
            resolved_invalid.unwrap_err(),
            GraphError::EntityNotFound(EntityId::new("non_existing"))
        );

        // Test non-entity-reference field value
        let string_field = FieldValue::String("not a reference".to_string());
        let resolved_string = graph.resolve_entity_reference(&string_field);
        assert!(resolved_string.is_err());
        assert_eq!(
            resolved_string.unwrap_err(),
            GraphError::NotAnEntityReference
        );

        let bool_field = FieldValue::Boolean(true);
        let resolved_bool = graph.resolve_entity_reference(&bool_field);
        assert!(resolved_bool.is_err());
        assert_eq!(resolved_bool.unwrap_err(), GraphError::NotAnEntityReference);
    }

    #[test]
    fn test_resolve_field_reference_simple() {
        let mut graph = EntityGraph::new();

        let entity = Entity::new(EntityId::new("test_entity"), EntityType::new("person"))
            .with_field(FieldId::new("name"), "John Doe")
            .with_field(
                FieldId::new("name_ref"),
                FieldValue::Reference(ReferenceValue::Field(
                    EntityId::new("test_entity"),
                    FieldId::new("name"),
                )),
            );

        graph.add_entity(entity).unwrap();
        graph.build(); // Build the graph edges

        // Test resolving field reference
        let field_ref = FieldValue::Reference(ReferenceValue::Field(
            EntityId::new("test_entity"),
            FieldId::new("name_ref"),
        ));
        let resolved = graph.resolve_field_reference(&field_ref);
        assert!(resolved.is_ok());
        assert_eq!(
            resolved.unwrap(),
            &FieldValue::String("John Doe".to_string())
        );
    }

    #[test]
    fn test_resolve_field_reference_chain() {
        let mut graph = EntityGraph::new();

        let entity = Entity::new(EntityId::new("test_entity"), EntityType::new("person"))
            .with_field(FieldId::new("name"), "John Doe")
            .with_field(
                FieldId::new("name_ref1"),
                FieldValue::Reference(ReferenceValue::Field(
                    EntityId::new("test_entity"),
                    FieldId::new("name"),
                )),
            )
            .with_field(
                FieldId::new("name_ref2"),
                FieldValue::Reference(ReferenceValue::Field(
                    EntityId::new("test_entity"),
                    FieldId::new("name_ref1"),
                )),
            );

        graph.add_entity(entity).unwrap();
        graph.build(); // Build the graph edges

        // Test resolving chained field reference
        let field_ref = FieldValue::Reference(ReferenceValue::Field(
            EntityId::new("test_entity"),
            FieldId::new("name_ref2"),
        ));
        let resolved = graph.resolve_field_reference(&field_ref);
        assert!(resolved.is_ok());
        assert_eq!(
            resolved.unwrap(),
            &FieldValue::String("John Doe".to_string())
        );
    }

    #[test]
    fn test_resolve_field_reference_cycle_detection() {
        let mut graph = EntityGraph::new();

        let entity = Entity::new(EntityId::new("test_entity"), EntityType::new("person"))
            .with_field(
                FieldId::new("ref1"),
                FieldValue::Reference(ReferenceValue::Field(
                    EntityId::new("test_entity"),
                    FieldId::new("ref2"),
                )),
            )
            .with_field(
                FieldId::new("ref2"),
                FieldValue::Reference(ReferenceValue::Field(
                    EntityId::new("test_entity"),
                    FieldId::new("ref1"),
                )),
            );

        graph.add_entity(entity).unwrap();
        graph.build(); // Build the graph edges

        // Test cycle detection
        let field_ref = FieldValue::Reference(ReferenceValue::Field(
            EntityId::new("test_entity"),
            FieldId::new("ref1"),
        ));
        let resolved = graph.resolve_field_reference(&field_ref);
        assert!(resolved.is_err());
        assert_eq!(resolved.unwrap_err(), GraphError::CyclicReference);
    }

    #[test]
    fn test_resolve_field_reference_max_depth() {
        let mut graph = EntityGraph::new();

        // Create a chain of 15 field references (exceeds default limit of 10)
        let mut entity = Entity::new(EntityId::new("test_entity"), EntityType::new("person"))
            .with_field(FieldId::new("final"), "Final Value");

        for i in 0..15 {
            let field_name = format!("ref{}", i);
            let next_field = if i == 14 {
                FieldId::new("final")
            } else {
                FieldId::new(&format!("ref{}", i + 1))
            };

            entity = entity.with_field(
                FieldId::new(&field_name),
                FieldValue::Reference(ReferenceValue::Field(
                    EntityId::new("test_entity"),
                    next_field,
                )),
            );
        }

        graph.add_entity(entity).unwrap();
        graph.build(); // Build the graph edges

        // Test max depth exceeded
        let field_ref = FieldValue::Reference(ReferenceValue::Field(
            EntityId::new("test_entity"),
            FieldId::new("ref0"),
        ));
        let resolved = graph.resolve_field_reference(&field_ref);
        assert!(resolved.is_err());
        assert_eq!(resolved.unwrap_err(), GraphError::MaxDepthExceeded);
    }

    #[test]
    fn test_resolve_field_reference_entity_not_found() {
        let graph = EntityGraph::new();

        let field_ref = FieldValue::Reference(ReferenceValue::Field(
            EntityId::new("missing_entity"),
            FieldId::new("field"),
        ));
        let resolved = graph.resolve_field_reference(&field_ref);
        assert!(resolved.is_err());
        assert_eq!(
            resolved.unwrap_err(),
            GraphError::EntityNotFound(EntityId::new("missing_entity"))
        );
    }

    #[test]
    fn test_resolve_field_reference_field_not_found() {
        let mut graph = EntityGraph::new();

        let entity = Entity::new(EntityId::new("test_entity"), EntityType::new("person"))
            .with_field(FieldId::new("existing_field"), "value");

        graph.add_entity(entity).unwrap();

        let field_ref = FieldValue::Reference(ReferenceValue::Field(
            EntityId::new("test_entity"),
            FieldId::new("missing_field"),
        ));
        let resolved = graph.resolve_field_reference(&field_ref);
        assert!(resolved.is_err());
        assert_eq!(
            resolved.unwrap_err(),
            GraphError::FieldNotFound(EntityId::new("test_entity"), FieldId::new("missing_field"))
        );
    }

    #[test]
    fn test_resolve_field_reference_not_a_reference() {
        let graph = EntityGraph::new();

        // Test with non-field-reference values
        let string_field = FieldValue::String("not a reference".to_string());
        let resolved = graph.resolve_field_reference(&string_field);
        assert!(resolved.is_err());
        assert_eq!(resolved.unwrap_err(), GraphError::NotAFieldReference);

        let bool_field = FieldValue::Boolean(true);
        let resolved = graph.resolve_field_reference(&bool_field);
        assert!(resolved.is_err());
        assert_eq!(resolved.unwrap_err(), GraphError::NotAFieldReference);

        let entity_ref = FieldValue::Reference(ReferenceValue::Entity(EntityId::new("entity")));
        let resolved = graph.resolve_field_reference(&entity_ref);
        assert!(resolved.is_err());
        assert_eq!(resolved.unwrap_err(), GraphError::NotAFieldReference);
    }

    #[test]
    fn test_resolve_field_reference_graph_not_built() {
        let mut graph = EntityGraph::new();

        let entity1 = Entity::new(EntityId::new("entity1"), EntityType::new("person"))
            .with_field(FieldId::new("name"), "Entity 1")
            .with_field(
                FieldId::new("ref_to_2"),
                FieldValue::Reference(ReferenceValue::Field(
                    EntityId::new("entity2"),
                    FieldId::new("value"),
                )),
            );

        let entity2 = Entity::new(EntityId::new("entity2"), EntityType::new("person"))
            .with_field(FieldId::new("value"), "Entity 2 Value");

        graph.add_entities(vec![entity1, entity2]).unwrap();
        // Intentionally NOT calling graph.build()

        let field_ref = FieldValue::Reference(ReferenceValue::Field(
            EntityId::new("entity1"),
            FieldId::new("ref_to_2"),
        ));
        let resolved = graph.resolve_field_reference(&field_ref);
        assert!(resolved.is_err());
        assert_eq!(resolved.unwrap_err(), GraphError::GraphNotBuilt);
    }

    #[test]
    fn test_field_value_resolve_entity_reference_convenience() {
        let mut graph = EntityGraph::new();

        let organization = Entity::new(EntityId::new("megacorp"), EntityType::new("organization"))
            .with_field(FieldId::new("name"), "MegaCorp Inc.");

        graph.add_entity(organization).unwrap();

        // Test convenience method on EntityReference
        let entity_ref = FieldValue::Reference(ReferenceValue::Entity(EntityId::new("megacorp")));
        let resolved = entity_ref.resolve_entity_reference(&graph);
        assert!(resolved.is_ok());
        assert_eq!(resolved.unwrap().id, EntityId::new("megacorp"));

        // Test convenience method on non-EntityReference
        let string_field = FieldValue::String("not a reference".to_string());
        let resolved = string_field.resolve_entity_reference(&graph);
        assert!(resolved.is_err());
        assert_eq!(resolved.unwrap_err(), GraphError::NotAnEntityReference);
    }

    #[test]
    fn test_field_value_resolve_field_reference_convenience() {
        let mut graph = EntityGraph::new();

        let entity = Entity::new(EntityId::new("test_entity"), EntityType::new("person"))
            .with_field(FieldId::new("name"), "John Doe")
            .with_field(
                FieldId::new("name_ref"),
                FieldValue::Reference(ReferenceValue::Field(
                    EntityId::new("test_entity"),
                    FieldId::new("name"),
                )),
            );

        graph.add_entity(entity).unwrap();
        graph.build();

        // Test convenience method on FieldReference
        let field_ref = FieldValue::Reference(ReferenceValue::Field(
            EntityId::new("test_entity"),
            FieldId::new("name_ref"),
        ));
        let resolved = field_ref.resolve_field_reference(&graph);
        assert!(resolved.is_ok());
        assert_eq!(
            resolved.unwrap(),
            &FieldValue::String("John Doe".to_string())
        );

        // Test convenience method on non-FieldReference
        let string_field = FieldValue::String("not a reference".to_string());
        let resolved = string_field.resolve_field_reference(&graph);
        assert!(resolved.is_err());
    }

    #[test]
    fn test_list_by_type() {
        let mut graph = EntityGraph::new();

        // Create entities of different types
        let organization1 = Entity::new(EntityId::new("megacorp"), EntityType::new("organization"))
            .with_field(FieldId::new("name"), "MegaCorp Inc.");

        let organization2 = Entity::new(EntityId::new("techcorp"), EntityType::new("organization"))
            .with_field(FieldId::new("name"), "TechCorp Ltd.");

        let person1 = Entity::new(EntityId::new("john_doe"), EntityType::new("person"))
            .with_field(FieldId::new("name"), "John Doe");

        let person2 = Entity::new(EntityId::new("jane_smith"), EntityType::new("person"))
            .with_field(FieldId::new("name"), "Jane Smith");

        graph
            .add_entities(vec![
                organization1.clone(),
                organization2.clone(),
                person1.clone(),
                person2.clone(),
            ])
            .unwrap();

        // Test listing organizations
        let organizations = graph.list_by_type(&EntityType::new("organization"));
        assert_eq!(organizations.len(), 2);
        let org_ids: Vec<&EntityId> = organizations.iter().map(|e| &e.id).collect();
        assert!(org_ids.contains(&&EntityId::new("megacorp")));
        assert!(org_ids.contains(&&EntityId::new("techcorp")));

        // Test listing persons
        let persons = graph.list_by_type(&EntityType::new("person"));
        assert_eq!(persons.len(), 2);
        let person_ids: Vec<&EntityId> = persons.iter().map(|e| &e.id).collect();
        assert!(person_ids.contains(&&EntityId::new("john_doe")));
        assert!(person_ids.contains(&&EntityId::new("jane_smith")));

        // Test non-existing type
        let projects = graph.list_by_type(&EntityType::new("missing_project"));
        assert_eq!(projects.len(), 0);
    }

    #[test]
    fn test_get_related() {
        let mut graph = EntityGraph::new();

        // Create entities with relationships
        let organization = Entity::new(EntityId::new("megacorp"), EntityType::new("organization"))
            .with_field(FieldId::new("name"), "MegaCorp Inc.");

        let person1 = Entity::new(EntityId::new("john_doe"), EntityType::new("person"))
            .with_field(FieldId::new("name"), "John Doe")
            .with_field(
                FieldId::new("employer"),
                FieldValue::Reference(ReferenceValue::Entity(EntityId::new("megacorp"))),
            );

        let person2 = Entity::new(EntityId::new("jane_smith"), EntityType::new("person"))
            .with_field(FieldId::new("name"), "Jane Smith")
            .with_field(
                FieldId::new("employer"),
                FieldValue::Reference(ReferenceValue::Entity(EntityId::new("megacorp"))),
            );

        graph
            .add_entities(vec![organization.clone(), person1.clone(), person2.clone()])
            .unwrap();
        graph.build();

        // Test getting all related entities (both directions)
        let related_to_megacorp = graph.get_related(&EntityId::new("megacorp"), None);
        assert!(related_to_megacorp.is_some());
        let related = related_to_megacorp.unwrap();
        assert_eq!(related.len(), 2);

        let related_ids: Vec<&EntityId> = related.iter().map(|e| &e.id).collect();
        assert!(related_ids.contains(&&EntityId::new("john_doe")));
        assert!(related_ids.contains(&&EntityId::new("jane_smith")));

        // Test getting related entities in specific direction
        let incoming = graph.get_related(&EntityId::new("megacorp"), Some(Direction::Incoming));
        assert!(incoming.is_some());
        let incoming_entities = incoming.unwrap();
        assert_eq!(incoming_entities.len(), 2);

        let outgoing = graph.get_related(&EntityId::new("john_doe"), Some(Direction::Outgoing));
        assert!(outgoing.is_some());
        let outgoing_entities = outgoing.unwrap();
        assert_eq!(outgoing_entities.len(), 1);
        assert_eq!(outgoing_entities[0].id, EntityId::new("megacorp"));

        // Test non-existing entity
        let non_existing = graph.get_related(&EntityId::new("non_existing"), None);
        assert!(non_existing.is_none());
    }
}
