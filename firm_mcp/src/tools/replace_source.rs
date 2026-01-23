//! Replace source tool implementation.

use std::path::Path;

use rmcp::model::{CallToolResult, Content};
use rmcp::schemars;

use crate::resources;

/// Parameters for the replace_source tool.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ReplaceSourceParams {
    /// Relative path to the .firm file (e.g., "schemas/person.firm", "core/main.firm").
    /// Use 'find_source' to locate the file path for a specific entity or schema.
    pub path: String,

    /// The exact string to find and replace. Must exist in the file.
    /// If found multiple times, set 'replace_all' to true or provide more context to make it unique.
    pub old_string: String,

    /// The replacement string. Can be empty to delete the old_string.
    pub new_string: String,

    /// If true, replace all occurrences of old_string. Default: false.
    /// When false, the tool errors if old_string appears more than once.
    #[serde(default)]
    pub replace_all: bool,

    /// If true, write the file even if workspace validation fails.
    /// Use this to fix a broken workspace where normal writes would be rolled back.
    /// The file must still have valid syntax. Default: false.
    #[serde(default)]
    pub force: bool,
}

/// Result of the replacement operation (before validation).
#[derive(Debug)]
pub struct ReplaceResult {
    /// The new content after replacement.
    pub new_content: String,
    /// Number of occurrences that were replaced.
    pub occurrences_replaced: usize,
    /// Original file content for rollback.
    pub original_content: String,
}

/// Execute the string replacement on the file content.
///
/// This performs the replacement but does NOT write the file or validate syntax.
/// The caller is responsible for validation and writing.
pub fn execute(
    workspace_path: &Path,
    params: &ReplaceSourceParams,
) -> Result<ReplaceResult, CallToolResult> {
    // Validate old_string is not empty
    if params.old_string.is_empty() {
        return Err(CallToolResult::error(vec![Content::text(
            "old_string cannot be empty.",
        )]));
    }

    // Read current file content
    let content = resources::read_source_file(workspace_path, &params.path)
        .map_err(|e| CallToolResult::error(vec![Content::text(e)]))?;

    // Count occurrences
    let occurrences = content.matches(&params.old_string).count();

    // Validate occurrences
    if occurrences == 0 {
        return Err(not_found_error(&params.old_string, &params.path));
    }

    if occurrences > 1 && !params.replace_all {
        return Err(multiple_matches_error(occurrences, &params.old_string));
    }

    // Perform replacement
    let new_content = if params.replace_all {
        content.replace(&params.old_string, &params.new_string)
    } else {
        content.replacen(&params.old_string, &params.new_string, 1)
    };

    let occurrences_replaced = if params.replace_all { occurrences } else { 1 };

    Ok(ReplaceResult {
        new_content,
        occurrences_replaced,
        original_content: content,
    })
}

/// Create a success result for replace_source.
pub fn success_result(path: &str, occurrences_replaced: usize) -> CallToolResult {
    let msg = if occurrences_replaced == 1 {
        format!("Replaced 1 occurrence in '{}'. Workspace is valid.", path)
    } else {
        format!(
            "Replaced {} occurrences in '{}'. Workspace is valid.",
            occurrences_replaced, path
        )
    };
    CallToolResult::success(vec![Content::text(msg)])
}

/// Create a success result for replace_source with force mode (validation warning).
pub fn force_success_result(
    path: &str,
    occurrences_replaced: usize,
    error: &str,
) -> CallToolResult {
    let msg = if occurrences_replaced == 1 {
        format!(
            "Replaced 1 occurrence in '{}'. Warning: workspace has validation errors: {}. \
             Use 'build' to check status after making more changes.",
            path, error
        )
    } else {
        format!(
            "Replaced {} occurrences in '{}'. Warning: workspace has validation errors: {}. \
             Use 'build' to check status after making more changes.",
            occurrences_replaced, path, error
        )
    };
    CallToolResult::success(vec![Content::text(msg)])
}

/// Create an error result when old_string is not found.
fn not_found_error(old_string: &str, path: &str) -> CallToolResult {
    // Truncate long strings for display
    let display_str = if old_string.len() > 100 {
        format!("{}...", &old_string[..100])
    } else {
        old_string.to_string()
    };

    CallToolResult::error(vec![Content::text(format!(
        "old_string not found in '{}': \"{}\"",
        path, display_str
    ))])
}

/// Create an error result when old_string appears multiple times.
fn multiple_matches_error(count: usize, old_string: &str) -> CallToolResult {
    // Truncate long strings for display
    let display_str = if old_string.len() > 100 {
        format!("{}...", &old_string[..100])
    } else {
        old_string.to_string()
    };

    CallToolResult::error(vec![Content::text(format!(
        "old_string found {} times: \"{}\". \
         Set 'replace_all: true' to replace all occurrences, \
         or provide more surrounding context to make the match unique.",
        count, display_str
    ))])
}

/// Create an error result when validation fails and rollback occurred.
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
