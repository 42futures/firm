//! Build tool implementation.

use rmcp::schemars;

/// Parameters for the build tool.
/// This tool takes no parameters - it rebuilds and validates the entire workspace.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BuildParams {}
