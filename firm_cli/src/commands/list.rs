use firm_lang::workspace::Workspace;
use std::path::PathBuf;

use super::{build_workspace, load_workspace_files};
use crate::errors::CliError;
use crate::files::load_current_graph;
use crate::ui::{self, OutputFormat};

/// Lists entities of a type or all schemas.
pub fn list_items(
    workspace_path: &PathBuf,
    target_type: String,
    output_format: OutputFormat,
) -> Result<(), CliError> {
    // Special case: if target_type is "schema", list all schemas instead of entities
    if target_type == "schema" {
        return list_schemas(workspace_path, output_format);
    }

    list_entities(workspace_path, target_type, output_format)
}

/// Lists entities of a given type in the workspace.
fn list_entities(
    workspace_path: &PathBuf,
    entity_type: String,
    output_format: OutputFormat,
) -> Result<(), CliError> {
    ui::header("Listing entities by type");
    let graph = load_current_graph(&workspace_path)?;

    let entities = graph.list_by_type(&entity_type.as_str().into());
    ui::success(&format!(
        "Found {} entities with type '{}'",
        entities.len(),
        entity_type,
    ));

    match output_format {
        OutputFormat::Pretty => ui::pretty_output_entity_list(&entities),
        OutputFormat::Json => ui::json_output(&entities),
    }

    Ok(())
}

/// Lists all schemas in the workspace.
fn list_schemas(workspace_path: &PathBuf, output_format: OutputFormat) -> Result<(), CliError> {
    ui::header("Listing schemas");
    let mut workspace = Workspace::new();
    load_workspace_files(&workspace_path, &mut workspace).map_err(|_| CliError::BuildError)?;
    let build = build_workspace(workspace).map_err(|_| CliError::BuildError)?;

    ui::success(&format!(
        "Found {} schemas for this workspace",
        build.schemas.len()
    ));

    match output_format {
        OutputFormat::Pretty => ui::pretty_output_schema_list(&build.schemas.iter().collect()),
        OutputFormat::Json => ui::json_output(&build.schemas),
    }
    Ok(())
}
