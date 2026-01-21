//! Get tool implementation.

use rmcp::schemars;

/// Parameters for the get tool.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetParams {
    /// Entity type (e.g., "person", "organization"), or "schema" to get a schema definition.
    pub r#type: String,
    /// Entity ID (e.g., "john_doe") or schema name (e.g., "person").
    pub id: String,
}
