//! Query tool implementation.

use firm_core::graph::{EntityGraph, Query, QueryResult};
use firm_lang::parser::query::parse_query;
use rmcp::model::{CallToolResult, Content};
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

/// Execute the query tool.
///
/// Parses and executes a Firm query, returning full details for all matching entities.
pub fn execute(graph: &EntityGraph, params: &QueryParams) -> CallToolResult {
    // Parse the query
    let parsed_query = match parse_query(&params.query) {
        Ok(q) => q,
        Err(e) => {
            return CallToolResult::error(vec![Content::text(format!(
                "Failed to parse query: {}",
                e
            ))]);
        }
    };

    // Convert to executable query
    let query: Query = match parsed_query.try_into() {
        Ok(q) => q,
        Err(e) => {
            return CallToolResult::error(vec![Content::text(format!(
                "Failed to convert query: {}",
                e
            ))]);
        }
    };

    // Execute the query
    let result = match query.execute(graph) {
        Ok(r) => r,
        Err(e) => {
            return CallToolResult::error(vec![Content::text(format!(
                "Query execution failed: {}",
                e
            ))]);
        }
    };

    // Format results
    match result {
        QueryResult::Entities(entities) => {
            if entities.is_empty() {
                return CallToolResult::success(vec![Content::text(
                    "No entities found matching the query.",
                )]);
            }
            let output: Vec<String> = entities.iter().map(|e| e.to_string()).collect();
            CallToolResult::success(vec![Content::text(output.join("\n---\n"))])
        }
        QueryResult::Aggregation(agg_result) => {
            CallToolResult::success(vec![Content::text(agg_result.to_string())])
        }
    }
}
