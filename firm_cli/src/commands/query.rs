use std::path::PathBuf;

use firm_core::graph::Query;
use firm_lang::parser::query::parse_query;

use crate::errors::CliError;
use crate::files::load_current_graph;
use crate::ui::{self, OutputFormat};

/// Executes a query against the workspace entity graph.
pub fn query_entities(
    workspace_path: &PathBuf,
    query_string: String,
    output_format: OutputFormat,
) -> Result<(), CliError> {
    ui::header("Executing query");
    let graph = load_current_graph(workspace_path)?;

    // Parse the query
    let parsed_query = parse_query(&query_string).map_err(|e| {
        ui::error(&format!("Failed to parse query: {}", e));
        CliError::QueryError
    })?;

    // Convert to executable query
    let query: Query = parsed_query.try_into().map_err(|e| {
        ui::error(&format!("Failed to convert query: {}", e));
        CliError::QueryError
    })?;

    // Execute the query
    ui::debug("Executing query");
    let results = query.execute(&graph).map_err(|e| {
        ui::error(&format!("Query execution failed: {}", e));
        CliError::QueryError
    })?;

    ui::success(&format!("Query returned {} entities", results.len()));

    // Output results
    match output_format {
        OutputFormat::Pretty => {
            ui::pretty_output_entity_list(&results);
        }
        OutputFormat::Json => {
            ui::json_output(&results);
        }
    }

    Ok(())
}
