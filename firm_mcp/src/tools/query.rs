//! Query tool implementation.

use rmcp::schemars;

/// Parameters for the query tool.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct QueryParams {
    /// Query string using the Firm query language. Examples:
    /// - "from person" (all persons)
    /// - "from task | where is_completed == false" (incomplete tasks)
    /// - "from person | where name contains 'John' | limit 5"
    pub query: String,
}
