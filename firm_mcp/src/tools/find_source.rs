//! Find source tool implementation.

use std::path::Path;

use firm_lang::workspace::Workspace;
use rmcp::model::{CallToolResult, Content};
use rmcp::schemars;

use crate::resources;

/// Parameters for the find_source tool.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FindSourceParams {
    /// Entity type (e.g., "person"), or "schema" to find a schema's source file.
    pub r#type: String,
    /// Entity ID (e.g., "john_doe") or schema name (e.g., "person").
    pub id: String,
}

/// Execute the find_source tool.
///
/// Returns the relative path to the .firm file containing the entity or schema definition.
pub fn execute(
    workspace: &Workspace,
    workspace_path: &Path,
    params: &FindSourceParams,
) -> CallToolResult {
    let source_path = if params.r#type == "schema" {
        workspace.find_schema_source(&params.id)
    } else {
        workspace.find_entity_source(&params.r#type, &params.id)
    };

    match source_path {
        Some(path) => {
            let relative = resources::to_relative_path(workspace_path, &path)
                .unwrap_or_else(|| path.to_string_lossy().to_string());
            CallToolResult::success(vec![Content::text(relative)])
        }
        None => {
            let msg = if params.r#type == "schema" {
                format!(
                    "Schema '{}' not found. Use list with type='schema' to see available schemas.",
                    params.id
                )
            } else {
                format!(
                    "Entity '{}' with type '{}' not found. Use list with type='{}' to see available IDs.",
                    params.id, params.r#type, params.r#type
                )
            };
            CallToolResult::error(vec![Content::text(msg)])
        }
    }
}
