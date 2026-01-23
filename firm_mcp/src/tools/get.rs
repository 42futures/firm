//! Get tool implementation.

use firm_core::compose_entity_id;
use firm_lang::workspace::WorkspaceBuild;
use rmcp::model::{CallToolResult, Content};
use rmcp::schemars;

/// Parameters for the get tool.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetParams {
    /// Entity type (e.g., "person", "organization"), or "schema" to get a schema definition.
    pub r#type: String,
    /// Entity ID (e.g., "john_doe") or schema name (e.g., "person").
    pub id: String,
}

/// Execute the get tool.
///
/// Returns full details of a single entity or schema.
pub fn execute(build: &WorkspaceBuild, params: &GetParams) -> CallToolResult {
    if params.r#type == "schema" {
        // Get schema by name
        let schema = build
            .schemas
            .iter()
            .find(|s| s.entity_type.as_str() == params.id);

        match schema {
            Some(schema) => CallToolResult::success(vec![Content::text(schema.to_string())]),
            None => CallToolResult::error(vec![Content::text(format!(
                "Schema '{}' not found. Use list with type='schema' to see available schemas.",
                params.id
            ))]),
        }
    } else {
        // Get entity by type and ID
        let id = compose_entity_id(&params.r#type, &params.id);
        match build.entities.iter().find(|e| e.id == id) {
            Some(entity) => CallToolResult::success(vec![Content::text(entity.to_string())]),
            None => CallToolResult::error(vec![Content::text(format!(
                "Entity '{}' with type '{}' not found. Use list with type='{}' to see available IDs.",
                params.id, params.r#type, params.r#type
            ))]),
        }
    }
}
