//! Read source tool implementation.

use rmcp::schemars;

/// Parameters for the read_source tool.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ReadSourceParams {
    /// Relative path to the .firm file (e.g., "schemas/person.firm", "core/main.firm").
    /// Use 'find_source' to locate the file path for a specific entity or schema.
    pub path: String,
}
