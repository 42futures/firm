//! Find source tool implementation.

use rmcp::schemars;

/// Parameters for the find_source tool.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FindSourceParams {
    /// Entity type (e.g., "person"), or "schema" to find a schema's source file.
    pub r#type: String,
    /// Entity ID (e.g., "john_doe") or schema name (e.g., "person").
    pub id: String,
}
