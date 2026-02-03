use firm_core::compose_entity_id;
use firm_lang::workspace::Workspace;
use std::path::PathBuf;

use super::{build_workspace, load_workspace_files};
use crate::errors::CliError;
use crate::files::load_current_graph;
use crate::ui::{self, OutputFormat};

/// Gets an entity or schema by type and ID/name.
pub fn get_item(
    workspace_path: &PathBuf,
    target_type: String,
    target_id: String,
    output_format: OutputFormat,
) -> Result<(), CliError> {
    // Special case: if target_type is "schema", get schema instead of entity
    if target_type == "schema" {
        return get_schema(workspace_path, target_id, output_format);
    }

    get_entity(workspace_path, target_type, target_id, output_format)
}

/// Gets a single entity by type and ID.
fn get_entity(
    workspace_path: &PathBuf,
    entity_type: String,
    entity_id: String,
    output_format: OutputFormat,
) -> Result<(), CliError> {
    ui::header("Getting entity by ID");
    let graph = load_current_graph(workspace_path)?;

    let id = compose_entity_id(&entity_type, &entity_id);
    match graph.get_entity(&id) {
        Some(entity) => {
            ui::success(&format!(
                "Found '{}' entity with ID '{}'",
                entity_type, entity_id
            ));

            match output_format {
                ui::OutputFormat::Pretty => ui::pretty_output_entity_single(entity),
                ui::OutputFormat::Json => ui::json_output(entity),
            }
            Ok(())
        }
        None => {
            ui::error(&format!(
                "Couldn't find '{}' entity with ID '{}'",
                entity_type, entity_id
            ));
            Err(CliError::QueryError)
        }
    }
}

/// Gets a single schema by name.
fn get_schema(
    workspace_path: &PathBuf,
    schema_name: String,
    output_format: OutputFormat,
) -> Result<(), CliError> {
    ui::header("Getting schema");
    let mut workspace = Workspace::new();
    load_workspace_files(workspace_path, &mut workspace).map_err(|_| CliError::BuildError)?;
    let build = build_workspace(workspace).map_err(|_| CliError::BuildError)?;

    // Find the schema by name
    let schema = build
        .schemas
        .iter()
        .find(|s| s.entity_type.as_str() == schema_name);

    match schema {
        Some(schema) => {
            ui::success(&format!("Found schema '{}'", schema_name));

            match output_format {
                OutputFormat::Pretty => ui::pretty_output_schema_single(schema),
                OutputFormat::Json => ui::json_output(schema),
            }
            Ok(())
        }
        None => {
            ui::error(&format!("Schema '{}' not found in workspace", schema_name));
            Err(CliError::QueryError)
        }
    }
}
