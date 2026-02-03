use firm_lang::workspace::Workspace;
use std::path::PathBuf;

use super::load_workspace_files;
use crate::errors::CliError;
use crate::ui::{self, OutputFormat};

/// Finds the source file for an entity or schema by its type and ID/name.
pub fn find_item_source(
    workspace_path: &PathBuf,
    target_type: String,
    target_id: String,
    output_format: OutputFormat,
) -> Result<(), CliError> {
    // Load workspace files (parse DSL but don't build/validate)
    let mut workspace = Workspace::new();
    load_workspace_files(workspace_path, &mut workspace).map_err(|_| CliError::BuildError)?;

    // Special case: if entity_type is "schema", search for schemas instead of entities
    let source_path = if target_type == "schema" {
        workspace.find_schema_source(&target_id)
    } else {
        workspace.find_entity_source(&target_type, &target_id)
    };

    match source_path {
        Some(source_path) => {
            match output_format {
                OutputFormat::Pretty => {
                    let is_schema = target_type == "schema";
                    let item_type = if is_schema { "schema" } else { "entity" };
                    let identifier = if is_schema { "name" } else { "ID" };
                    ui::success(&format!(
                        "Found source file for '{}' {} with {} '{}'",
                        target_type, item_type, identifier, target_id
                    ));
                    ui::raw_output(&source_path.display().to_string());
                }
                OutputFormat::Json => {
                    #[derive(serde::Serialize)]
                    struct SourceResult {
                        target_type: String,
                        target_id: String,
                        source_path: PathBuf,
                    }
                    ui::json_output(&SourceResult {
                        target_type,
                        target_id,
                        source_path,
                    });
                }
            }
            Ok(())
        }
        None => {
            let error_msg = if target_type == "schema" {
                format!("Schema with name '{}' not found in workspace", target_id)
            } else {
                format!(
                    "Entity '{}' with type '{}' not found in workspace",
                    target_id, target_type
                )
            };
            ui::error(&error_msg);
            Err(CliError::QueryError)
        }
    }
}
