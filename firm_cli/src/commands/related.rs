use firm_core::compose_entity_id;
use std::path::PathBuf;

use crate::errors::CliError;
use crate::files::load_current_graph;
use crate::query::CliDirection;
use crate::ui::{self, OutputFormat};

/// Gets entities related to a specific entity.
pub fn get_related_entities(
    workspace_path: &PathBuf,
    entity_type: String,
    entity_id: String,
    direction: Option<CliDirection>,
    output_format: OutputFormat,
) -> Result<(), CliError> {
    ui::header("Getting related entities");
    let graph = load_current_graph(&workspace_path)?;

    let id = compose_entity_id(&entity_type, &entity_id);
    match graph.get_related(&id, direction.clone().map(|d| d.into())) {
        Some(entities) => {
            let direction_text = match direction {
                Some(CliDirection::To) => "references to",
                Some(CliDirection::From) => "references from",
                None => "relationships for",
            };

            ui::success(&format!(
                "Found {} {} '{}' entity with ID '{}'",
                entities.len(),
                direction_text,
                entity_type,
                entity_id
            ));

            match output_format {
                OutputFormat::Pretty => ui::pretty_output_entity_list(&entities),
                OutputFormat::Json => ui::json_output(&entities),
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
