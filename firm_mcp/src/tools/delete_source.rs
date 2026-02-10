//! Delete source tool implementation.

use std::fs;
use std::path::Path;

use rmcp::model::{CallToolResult, Content};
use rmcp::schemars;

use crate::resources;

/// Parameters for the delete_source tool.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct DeleteSourceParams {
    /// Relative path to the .firm file to delete (e.g., "generated/task.firm").
    pub path: String,

    /// If true, keep the file deleted even if workspace validation fails afterward.
    /// Default: false (restores the file if deletion breaks the workspace).
    #[serde(default)]
    pub force: bool,
}

/// Result of a successful delete operation.
#[derive(Debug)]
pub struct DeleteResult {
    /// The original file content, kept for potential rollback.
    pub original_content: String,
}

/// Execute the delete_source tool.
///
/// Validates the path, reads the file for rollback, and deletes it.
/// The caller (server.rs) is responsible for rebuilding and rolling back if needed.
pub fn execute(workspace_path: &Path, params: &DeleteSourceParams) -> Result<DeleteResult, String> {
    if !params.path.ends_with(".firm") {
        return Err(format!(
            "Path must end with .firm extension: {}",
            params.path
        ));
    }

    let absolute_path = resources::to_absolute_path(workspace_path, &params.path)
        .ok_or_else(|| format!("Invalid path (must be within workspace): {}", params.path))?;

    if !absolute_path.exists() {
        return Err(format!("File not found: {}", params.path));
    }

    // Read content before deleting so we can rollback
    let original_content =
        fs::read_to_string(&absolute_path).map_err(|e| format!("Failed to read file: {}", e))?;

    fs::remove_file(&absolute_path).map_err(|e| format!("Failed to delete file: {}", e))?;

    Ok(DeleteResult { original_content })
}

/// Restore a deleted file.
pub fn rollback(workspace_path: &Path, path: &str, content: &str) -> bool {
    let absolute_path = workspace_path.join(path);
    if let Some(parent) = absolute_path.parent() {
        if fs::create_dir_all(parent).is_err() {
            return false;
        }
    }
    fs::write(&absolute_path, content).is_ok()
}

pub fn success_result(path: &str) -> CallToolResult {
    CallToolResult::success(vec![Content::text(format!(
        "Deleted '{}'. Workspace is valid.",
        path
    ))])
}

pub fn force_success_result(path: &str, error: &str) -> CallToolResult {
    CallToolResult::success(vec![Content::text(format!(
        "Deleted '{}'. Warning: workspace has validation errors: {}. \
         Use 'build' to check status after making more changes.",
        path, error
    ))])
}

pub fn validation_error_result(error: &str, rollback_success: bool) -> CallToolResult {
    let rollback_msg = if rollback_success {
        "File has been restored."
    } else {
        "Warning: Failed to restore file."
    };

    CallToolResult::error(vec![Content::text(format!(
        "Deletion would break the workspace: {}. {} Use 'force: true' to delete anyway.",
        error, rollback_msg
    ))])
}
