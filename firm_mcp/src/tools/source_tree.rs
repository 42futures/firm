//! Source tree tool implementation.

use std::collections::BTreeMap;
use std::path::Path;

use firm_lang::workspace::Workspace;
use rmcp::model::{CallToolResult, Content};
use rmcp::schemars;

use crate::resources;

/// Parameters for the source_tree tool.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SourceTreeParams {}

/// Execute the source_tree tool.
pub fn execute(workspace: &Workspace, workspace_path: &Path) -> CallToolResult {
    let mut paths: Vec<String> = workspace
        .file_paths()
        .iter()
        .filter_map(|path| resources::to_relative_path(workspace_path, path))
        .collect();

    paths.sort();

    if paths.is_empty() {
        return CallToolResult::success(vec![Content::text("No .firm source files found.")]);
    }

    // Group files by directory
    let mut tree: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for path in &paths {
        let (dir, file) = match path.rsplit_once('/') {
            Some((d, f)) => (d.to_string(), f.to_string()),
            None => (String::new(), path.clone()),
        };
        tree.entry(dir).or_default().push(file);
    }

    // Render as indented tree
    let mut output = String::new();
    for (dir, files) in &tree {
        if dir.is_empty() {
            for file in files {
                output.push_str(file);
                output.push('\n');
            }
        } else {
            output.push_str(dir);
            output.push_str("/\n");
            for file in files {
                output.push_str("  ");
                output.push_str(file);
                output.push('\n');
            }
        }
    }

    CallToolResult::success(vec![Content::text(output.trim_end())])
}
