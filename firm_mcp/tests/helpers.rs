//! Shared test helpers for firm_mcp tests.

#![allow(dead_code)]

use std::fs;

use firm_lang::workspace::Workspace;
use rmcp::model::{CallToolResult, RawContent};
use tempfile::TempDir;

/// Extract the text content from a CallToolResult.
pub fn get_text(result: &CallToolResult) -> String {
    assert_eq!(result.content.len(), 1, "Expected exactly one content item");
    match &result.content[0].raw {
        RawContent::Text(text_content) => text_content.text.clone(),
        _ => panic!("Expected text content"),
    }
}

/// Check if the result is a success.
pub fn is_success(result: &CallToolResult) -> bool {
    result.is_error == Some(false)
}

/// Check if the result is an error.
pub fn is_error(result: &CallToolResult) -> bool {
    result.is_error == Some(true)
}

/// Create a test workspace with the given files.
///
/// Returns the TempDir (must be kept alive) and the loaded Workspace.
pub fn create_workspace(files: &[(&str, &str)]) -> (TempDir, Workspace) {
    let dir = TempDir::new().expect("Failed to create temp dir");

    for (path, content) in files {
        let file_path = dir.path().join(path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create parent dirs");
        }
        fs::write(&file_path, content).expect("Failed to write file");
    }

    let mut workspace = Workspace::new();
    workspace
        .load_directory(&dir.path().to_path_buf())
        .expect("Failed to load workspace");

    (dir, workspace)
}
