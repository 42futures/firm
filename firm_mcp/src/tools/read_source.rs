//! Read source tool implementation.

use std::path::Path;

use rmcp::model::{CallToolResult, Content};
use rmcp::schemars;

use crate::resources;

/// Parameters for the read_source tool.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ReadSourceParams {
    /// Relative path to the .firm file (e.g., "schemas/person.firm", "core/main.firm").
    /// Use 'find_source' to locate the file path for a specific entity or schema.
    pub path: String,
}

/// Execute the read_source tool.
///
/// Returns the raw DSL content of the specified .firm file.
pub fn execute(workspace_path: &Path, params: &ReadSourceParams) -> CallToolResult {
    match resources::read_source_file(workspace_path, &params.path) {
        Ok(contents) => CallToolResult::success(vec![Content::text(contents)]),
        Err(e) => CallToolResult::error(vec![Content::text(e)]),
    }
}
