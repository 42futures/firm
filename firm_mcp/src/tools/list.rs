//! List tool implementation.

use firm_lang::workspace::WorkspaceBuild;
use rmcp::model::{CallToolResult, Content};
use rmcp::schemars;

/// Parameters for the list tool.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ListParams {
    /// Entity type to list (e.g., "person", "organization"), or "schema" to list all schemas.
    pub r#type: String,
}

/// Execute the list tool.
///
/// Returns all entity IDs of the given type, or all schema names if type is "schema".
pub fn execute(build: &WorkspaceBuild, params: &ListParams) -> CallToolResult {
    let result = if params.r#type == "schema" {
        // List all schema names
        let names: Vec<&str> = build
            .schemas
            .iter()
            .map(|s| s.entity_type.as_str())
            .collect();
        names.join("\n")
    } else {
        // List all entity IDs of the given type
        let ids: Vec<&str> = build
            .entities
            .iter()
            .filter(|e| e.entity_type.as_str() == params.r#type)
            .map(|e| e.id.as_str())
            .collect();
        ids.join("\n")
    };

    CallToolResult::success(vec![Content::text(result)])
}
