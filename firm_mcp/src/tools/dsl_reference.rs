//! DSL reference tool implementation.
//!
//! Returns documentation for the Firm DSL syntax and query language.

use rmcp::model::{CallToolResult, Content};
use rmcp::schemars;

use super::dsl_reference_content::{DSL_REFERENCE, QUERY_REFERENCE};

/// Parameters for the dsl_reference tool.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct DslReferenceParams {
    /// Which reference to return: "dsl" for DSL syntax, "query" for query language, or "all" for both.
    /// Defaults to "all" if not specified.
    #[serde(default = "default_topic")]
    pub topic: String,
}

fn default_topic() -> String {
    "all".to_string()
}

/// Execute the dsl_reference tool.
pub fn execute(params: &DslReferenceParams) -> CallToolResult {
    let content = match params.topic.to_lowercase().as_str() {
        "dsl" => DSL_REFERENCE.to_string(),
        "query" => QUERY_REFERENCE.to_string(),
        "all" | "" => format!("{}\n\n---\n\n{}", DSL_REFERENCE, QUERY_REFERENCE),
        other => {
            return CallToolResult::error(vec![Content::text(format!(
                "Unknown topic '{}'. Valid options: 'dsl', 'query', 'all'",
                other
            ))]);
        }
    };

    CallToolResult::success(vec![Content::text(content)])
}
