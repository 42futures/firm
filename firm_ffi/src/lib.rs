uniffi::setup_scaffolding!();

mod types;

pub use types::{
    FirmDirection, FirmEntity, FirmField, FirmFieldValue, FirmQueryResult, FirmSchema,
    FirmSchemaField,
};

use std::path::PathBuf;
use std::sync::Mutex;

use firm_core::graph::{EntityGraph, Query, QueryResult};
use firm_core::{Entity, EntityType, compose_entity_id};
use firm_lang::generate::generate_dsl;
use firm_lang::parser::query::parse_query;
use firm_lang::workspace::{Workspace, WorkspaceBuild};

/// Errors from the FFI layer.
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum FirmError {
    #[error("Parse error: {message}")]
    ParseError { message: String },
    #[error("Build error: {message}")]
    BuildError { message: String },
    #[error("Query error: {message}")]
    QueryError { message: String },
    #[error("Not built: call build() first")]
    NotBuilt,
    #[error("{message}")]
    Other { message: String },
}

/// Opaque handle to a Firm workspace with built graph.
#[derive(uniffi::Object)]
pub struct FirmSession {
    inner: Mutex<SessionInner>,
}

struct SessionInner {
    workspace: Workspace,
    build: Option<WorkspaceBuild>,
    graph: Option<EntityGraph>,
}

impl Default for FirmSession {
    fn default() -> Self {
        Self::new()
    }
}

#[uniffi::export]
impl FirmSession {
    /// Create a new empty session.
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(SessionInner {
                workspace: Workspace::new(),
                build: None,
                graph: None,
            }),
        }
    }

    /// Load .firm source text with a virtual path.
    pub fn load_source(&self, content: String, path: String) -> Result<(), FirmError> {
        let mut inner = self.inner.lock().map_err(|e| FirmError::Other {
            message: e.to_string(),
        })?;
        inner
            .workspace
            .load_string(content, PathBuf::from(&path))
            .map_err(|e| FirmError::ParseError {
                message: e.to_string(),
            })?;
        // Invalidate previous build
        inner.build = None;
        inner.graph = None;
        Ok(())
    }

    /// Parse and validate all loaded sources, build entity graph.
    pub fn build(&self) -> Result<(), FirmError> {
        let mut inner = self.inner.lock().map_err(|e| FirmError::Other {
            message: e.to_string(),
        })?;
        let workspace_build =
            inner
                .workspace
                .build()
                .map_err(|e| FirmError::BuildError {
                    message: e.to_string(),
                })?;

        let mut graph = EntityGraph::new();
        graph
            .add_entities(workspace_build.entities.clone())
            .map_err(|e| FirmError::BuildError {
                message: format!("{:?}", e),
            })?;
        graph.build();

        inner.build = Some(workspace_build);
        inner.graph = Some(graph);
        Ok(())
    }

    /// Execute a Firm query string.
    pub fn query(&self, query_string: String) -> Result<FirmQueryResult, FirmError> {
        let inner = self.inner.lock().map_err(|e| FirmError::Other {
            message: e.to_string(),
        })?;
        let graph = inner.graph.as_ref().ok_or(FirmError::NotBuilt)?;

        let parsed_query = parse_query(&query_string).map_err(|e| FirmError::QueryError {
            message: format!("Failed to parse query: {}", e),
        })?;

        let query: Query = parsed_query.try_into().map_err(
            |e: firm_lang::convert::to_query::QueryConversionError| FirmError::QueryError {
                message: format!("Failed to convert query: {}", e),
            },
        )?;

        let result = query.execute(graph).map_err(|e| FirmError::QueryError {
            message: format!("Query execution failed: {}", e),
        })?;

        match result {
            QueryResult::Entities(entities) => Ok(FirmQueryResult::Entities {
                entities: entities.iter().map(|e| FirmEntity::from(*e)).collect(),
            }),
            QueryResult::Aggregation(agg) => Ok(FirmQueryResult::Aggregation {
                value: agg.to_string(),
            }),
        }
    }

    /// Get an entity by type and ID.
    pub fn get_entity(
        &self,
        entity_type: String,
        id: String,
    ) -> Result<Option<FirmEntity>, FirmError> {
        let inner = self.inner.lock().map_err(|e| FirmError::Other {
            message: e.to_string(),
        })?;
        let graph = inner.graph.as_ref().ok_or(FirmError::NotBuilt)?;

        let composite_id = compose_entity_id(&entity_type, &id);
        Ok(graph.get_entity(&composite_id).map(FirmEntity::from))
    }

    /// List entity IDs for a given type.
    pub fn list_entities(&self, entity_type: String) -> Result<Vec<String>, FirmError> {
        let inner = self.inner.lock().map_err(|e| FirmError::Other {
            message: e.to_string(),
        })?;
        let graph = inner.graph.as_ref().ok_or(FirmError::NotBuilt)?;

        let et = EntityType::new(&entity_type);
        let entities = graph.list_by_type(&et);
        Ok(entities.iter().map(|e| e.id.to_string()).collect())
    }

    /// List all schema type names.
    pub fn list_schemas(&self) -> Result<Vec<String>, FirmError> {
        let inner = self.inner.lock().map_err(|e| FirmError::Other {
            message: e.to_string(),
        })?;
        let build = inner.build.as_ref().ok_or(FirmError::NotBuilt)?;

        Ok(build
            .schemas
            .iter()
            .map(|s| s.entity_type.to_string())
            .collect())
    }

    /// Get related entities.
    pub fn get_related(
        &self,
        entity_type: String,
        id: String,
        direction: FirmDirection,
    ) -> Result<Vec<FirmEntity>, FirmError> {
        let inner = self.inner.lock().map_err(|e| FirmError::Other {
            message: e.to_string(),
        })?;
        let graph = inner.graph.as_ref().ok_or(FirmError::NotBuilt)?;

        let composite_id = compose_entity_id(&entity_type, &id);
        let dir = match direction {
            FirmDirection::Outgoing => Some(firm_core::graph::Direction::Outgoing),
            FirmDirection::Incoming => Some(firm_core::graph::Direction::Incoming),
            FirmDirection::Both => None,
        };

        match graph.get_related(&composite_id, dir) {
            Some(entities) => Ok(entities.iter().map(|e| FirmEntity::from(*e)).collect()),
            None => Ok(vec![]),
        }
    }

    /// Get the structured schema definition for an entity type.
    pub fn get_schema(&self, entity_type: String) -> Result<Option<FirmSchema>, FirmError> {
        let inner = self.inner.lock().map_err(|e| FirmError::Other {
            message: e.to_string(),
        })?;
        let build = inner.build.as_ref().ok_or(FirmError::NotBuilt)?;
        let et = EntityType::new(&entity_type);
        Ok(build
            .schemas
            .iter()
            .find(|s| s.entity_type == et)
            .map(|s| {
                let fields = s
                    .ordered_fields()
                    .iter()
                    .map(|(field_id, field_schema)| FirmSchemaField {
                        name: field_id.to_string(),
                        field_type: field_schema.expected_type().to_string(),
                        required: field_schema.is_required(),
                        allowed_values: field_schema.allowed_values().cloned(),
                    })
                    .collect();
                FirmSchema {
                    entity_type: s.entity_type.to_string(),
                    fields,
                }
            }))
    }

    /// Generate .firm DSL text for an entity.
    pub fn generate_entity_dsl(&self, entity: FirmEntity) -> Result<String, FirmError> {
        let core_entity =
            Entity::try_from(&entity).map_err(|e| FirmError::Other { message: e })?;
        Ok(generate_dsl(&[core_entity]))
    }

    /// Find the virtual path containing a given entity.
    pub fn find_entity_source(
        &self,
        entity_type: String,
        id: String,
    ) -> Result<Option<String>, FirmError> {
        let inner = self.inner.lock().map_err(|e| FirmError::Other {
            message: e.to_string(),
        })?;
        Ok(inner
            .workspace
            .find_entity_source(&entity_type, &id)
            .map(|p| p.to_string_lossy().into_owned()))
    }

    /// Get the loaded source content for a virtual path.
    pub fn get_source(&self, path: String) -> Result<Option<String>, FirmError> {
        let inner = self.inner.lock().map_err(|e| FirmError::Other {
            message: e.to_string(),
        })?;
        Ok(inner
            .workspace
            .get_source(&PathBuf::from(&path))
            .map(|s| s.to_string()))
    }
}
