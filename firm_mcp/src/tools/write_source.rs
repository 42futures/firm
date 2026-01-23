//! Write source tool implementation.

use std::fs;
use std::path::Path;

use firm_lang::parser::dsl::parse_source;
use rmcp::model::{CallToolResult, Content};
use rmcp::schemars;

use crate::resources;

/// Parameters for the write_source tool.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct WriteSourceParams {
    /// Relative path to the .firm file (e.g., "schemas/person.firm", "core/main.firm").
    /// Use 'find_source' to locate the file path for a specific entity or schema.
    pub path: String,

    /// The DSL content to write to the file. Must be valid Firm DSL syntax.
    pub content: String,

    /// If true, write the file even if workspace validation fails.
    /// Use this to fix a broken workspace where normal writes would be rolled back.
    /// The file must still have valid syntax. Default: false.
    #[serde(default)]
    pub force: bool,
}

/// Result of syntax validation and file write.
#[derive(Debug)]
pub struct WriteResult {
    /// Whether the file existed before (vs. newly created)
    pub file_existed: bool,
    /// Original file content for rollback (None if file was new)
    pub original_content: Option<String>,
}

/// Validate syntax and write the file.
///
/// Returns Ok(WriteResult) if syntax is valid and file was written.
/// Returns Err(CallToolResult) if syntax validation failed.
///
/// After calling this, the caller should rebuild the workspace and handle
/// rollback if rebuild fails (unless force mode is enabled).
pub fn validate_and_write(
    workspace_path: &Path,
    params: &WriteSourceParams,
) -> Result<WriteResult, CallToolResult> {
    // First, validate the content by parsing it (syntax check - always required)
    let parsed = match parse_source(params.content.clone(), None) {
        Ok(parsed) => parsed,
        Err(e) => {
            return Err(CallToolResult::error(vec![Content::text(format!(
                "Failed to parse DSL: {}",
                e
            ))]));
        }
    };

    // Check for syntax errors in the parse tree (always required, even with force)
    if parsed.has_error() {
        return Err(CallToolResult::error(vec![Content::text(
            "Invalid DSL syntax: the content contains parse errors. \
             Please check for unclosed braces, missing values, or malformed references.",
        )]));
    }

    // Get absolute path for the file
    let absolute_path = workspace_path.join(&params.path);

    // Read existing file content for potential rollback (None if file doesn't exist)
    let original_content = fs::read_to_string(&absolute_path).ok();
    let file_existed = original_content.is_some();

    // Write the new content
    if let Err(e) = resources::write_source_file(workspace_path, &params.path, &params.content) {
        return Err(CallToolResult::error(vec![Content::text(e)]));
    }

    Ok(WriteResult {
        file_existed,
        original_content,
    })
}

/// Rollback a write operation by restoring the original file or deleting a new file.
pub fn rollback(workspace_path: &Path, path: &str, original_content: Option<String>) -> bool {
    let absolute_path = workspace_path.join(path);

    let result = if let Some(original) = original_content {
        fs::write(&absolute_path, original)
    } else {
        fs::remove_file(&absolute_path)
    };

    result.is_ok()
}

/// Create a success result for write_source.
pub fn success_result(path: &str, content_len: usize, file_existed: bool) -> CallToolResult {
    let action = if file_existed { "Updated" } else { "Created" };
    CallToolResult::success(vec![Content::text(format!(
        "{} {} ({} bytes). Workspace is valid.",
        action, path, content_len
    ))])
}

/// Create a success result for write_source with force mode (validation warning).
pub fn force_success_result(
    path: &str,
    content_len: usize,
    file_existed: bool,
    error: &str,
) -> CallToolResult {
    let action = if file_existed { "Updated" } else { "Created" };
    CallToolResult::success(vec![Content::text(format!(
        "{} {} ({} bytes). Warning: workspace has validation errors: {}. \
         Use 'build' to check status after making more changes.",
        action, path, content_len, error
    ))])
}

/// Create an error result for write_source when validation fails and rollback occurred.
pub fn validation_error_result(error: &str, rollback_success: bool) -> CallToolResult {
    let rollback_msg = if rollback_success {
        "Changes have been rolled back."
    } else {
        "Warning: Failed to rollback changes."
    };

    CallToolResult::error(vec![Content::text(format!(
        "Validation failed: {}. {} Use 'force: true' to write anyway.",
        error, rollback_msg
    ))])
}
