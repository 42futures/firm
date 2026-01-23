//! Build tool implementation.

use rmcp::model::{CallToolResult, Content};
use rmcp::schemars;

/// Parameters for the build tool.
/// This tool takes no parameters - it rebuilds and validates the entire workspace.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BuildParams {}

/// Create a success result for build.
pub fn success_result(entity_count: usize, schema_count: usize) -> CallToolResult {
    CallToolResult::success(vec![Content::text(format!(
        "Workspace is valid. {} entities, {} schemas.",
        entity_count, schema_count
    ))])
}

/// Create an error result for build.
pub fn error_result(error: &str) -> CallToolResult {
    CallToolResult::error(vec![Content::text(format!(
        "Workspace validation failed: {}",
        error
    ))])
}
