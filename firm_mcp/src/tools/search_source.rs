//! Search source tool implementation.

use std::fmt::Write;
use std::fs;
use std::path::Path;

use firm_lang::workspace::Workspace;
use rmcp::model::{CallToolResult, Content};
use rmcp::schemars;

use crate::resources;

/// Maximum number of matching lines to return.
const MAX_MATCHES: usize = 50;

/// Parameters for the search_source tool.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SearchSourceParams {
    /// Text to search for across all .firm source files.
    pub query: String,

    /// If true, match case exactly. Default: false (case-insensitive).
    #[serde(default)]
    pub case_sensitive: bool,
}

/// Execute the search_source tool.
pub fn execute(workspace: &Workspace, workspace_path: &Path, params: &SearchSourceParams) -> CallToolResult {
    if params.query.is_empty() {
        return CallToolResult::error(vec![Content::text("Search query cannot be empty.")]);
    }

    let query_lower = params.query.to_lowercase();
    let mut paths: Vec<String> = workspace
        .file_paths()
        .iter()
        .filter_map(|path| resources::to_relative_path(workspace_path, path))
        .collect();
    paths.sort();

    let mut output = String::new();
    let mut total_matches: usize = 0;
    let mut truncated = false;

    for rel_path in &paths {
        let abs_path = workspace_path.join(rel_path);
        let content = match fs::read_to_string(&abs_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let mut file_matches = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let matches = if params.case_sensitive {
                line.contains(&params.query)
            } else {
                line.to_lowercase().contains(&query_lower)
            };

            if matches {
                if total_matches >= MAX_MATCHES {
                    truncated = true;
                    break;
                }
                file_matches.push((line_num + 1, line));
                total_matches += 1;
            }
        }

        if !file_matches.is_empty() {
            writeln!(output, "{}:", rel_path).unwrap();
            for (line_num, line) in &file_matches {
                writeln!(output, "  {}:  {}", line_num, line.trim()).unwrap();
            }
            writeln!(output).unwrap();
        }

        if truncated {
            break;
        }
    }

    if total_matches == 0 {
        return CallToolResult::success(vec![Content::text(format!(
            "No matches found for '{}'.",
            params.query
        ))]);
    }

    let mut result = output.trim_end().to_string();
    if truncated {
        write!(
            result,
            "\n\n... results truncated at {} matches. Narrow your search for more specific results.",
            MAX_MATCHES
        )
        .unwrap();
    }

    CallToolResult::success(vec![Content::text(result)])
}
