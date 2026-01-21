//! Write source tool implementation.

use rmcp::schemars;

/// Parameters for the write_source tool.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct WriteSourceParams {
    /// Relative path to the .firm file (e.g., "schemas/person.firm", "core/main.firm").
    /// Use 'find_source' to locate the file path for a specific entity or schema.
    pub path: String,

    /// The DSL content to write to the file. Must be valid Firm DSL syntax.
    pub content: String,

    /// If true, write the file even if workspace validation fails.
    /// Use this to fix a broken workspace where normal writes would be rolled back.
    /// The file must still have valid syntax. Default: false.
    #[serde(default)]
    pub force: bool,
}
