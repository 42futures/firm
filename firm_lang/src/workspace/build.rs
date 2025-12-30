use firm_core::{Entity, EntitySchema, EntityType};
use std::collections::HashMap;

use super::{Workspace, WorkspaceError};

/// Holds converted entities and schemas after the workspace is built.
#[derive(Debug)]
pub struct WorkspaceBuild {
    pub entities: Vec<Entity>,
    pub schemas: Vec<EntitySchema>,
}

impl WorkspaceBuild {
    pub fn new(entities: Vec<Entity>, schemas: Vec<EntitySchema>) -> Self {
        WorkspaceBuild { entities, schemas }
    }
}

impl Workspace {
    /// Build the workspace from all loaded files.
    pub fn build(&mut self) -> Result<WorkspaceBuild, WorkspaceError> {
        self.build_with_progress(|current, total, phase| {
            log::debug!("{}: {}/{}", phase, current, total);
        })
    }

    /// Build the workspace with progress reporting.
    pub fn build_with_progress<F>(
        &mut self,
        mut progress: F,
    ) -> Result<WorkspaceBuild, WorkspaceError>
    where
        F: FnMut(usize, usize, &str),
    {
        // Initialize empty schema collection
        let mut schemas: HashMap<EntityType, EntitySchema> = HashMap::new();

        let files_to_process = self.num_files();
        let mut files_processed = 0;
        progress(files_to_process, files_processed, "Building schemas");

        // First pass: Walk through workspace files to collect schemas
        for (path, file) in &self.files {
            let parsed_schemas = file.parsed.schemas();
            for parsed_schema in &parsed_schemas {
                let schema = EntitySchema::try_from(parsed_schema)
                    .map_err(|err| WorkspaceError::ParseError(path.clone(), err.to_string()))?;

                if schemas.contains_key(&schema.entity_type) {
                    return Err(WorkspaceError::ValidationError(
                        path.clone(),
                        "Stuff".to_string(),
                    ));
                }

                schemas.insert(schema.entity_type.clone(), schema);
            }
        }

        // Second pass: Walk through workspace files to build and validate entities against schemas
        let mut entities = Vec::new();

        files_processed = 0;

        for (path, file) in &self.files {
            progress(files_to_process, files_processed, "Building entities");

            let parsed_entities = file.parsed.entities();
            for parsed_entity in &parsed_entities {
                // Build the entity
                let entity = Entity::try_from(parsed_entity)
                    .map_err(|err| WorkspaceError::ParseError(path.clone(), err.to_string()))?;

                // Find the appropriate schema for this entity
                let schema = schemas.get(&entity.entity_type).ok_or_else(|| {
                    WorkspaceError::MissingSchemaError(path.clone(), entity.entity_type.clone())
                })?;

                // Validate the entity against its schema
                if let Err(validation_errors) = schema.validate(&entity) {
                    let error_msg = format!(
                        "Entity '{}' failed validation: {:?}",
                        entity.id, validation_errors
                    );
                    return Err(WorkspaceError::ValidationError(path.clone(), error_msg));
                }

                entities.push(entity);
            }

            files_processed += 1;
        }

        let schemas_vec = schemas.into_values().collect();
        Ok(WorkspaceBuild::new(entities, schemas_vec))
    }
}
