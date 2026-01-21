//! List tool implementation.

use rmcp::schemars;

/// Parameters for the list tool.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ListParams {
    /// Entity type to list (e.g., "person", "organization"), or "schema" to list all schemas.
    pub r#type: String,
}
