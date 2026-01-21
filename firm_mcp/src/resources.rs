//! Resource handling for the Firm MCP server.
//!
//! Resources expose .firm source files to MCP clients:
//! - `firm://source` - lists all .firm file paths in the workspace
//! - `firm://source/{path}` - reads the contents of a specific .firm file

use std::fs;
use std::path::{Path, PathBuf};

use rmcp::model::{AnnotateAble, RawResource, Resource};

/// The URI scheme for Firm resources.
pub const SCHEME: &str = "firm";

/// The resource type for source files.
pub const SOURCE_TYPE: &str = "source";

/// Creates a URI for a specific source file.
pub fn source_file_uri(relative_path: &str) -> String {
    format!("{}://{}/{}", SCHEME, SOURCE_TYPE, relative_path)
}

/// Parses a source file URI and returns the relative path.
///
/// Returns `None` if the URI doesn't match the expected format.
pub fn parse_source_uri(uri: &str) -> Option<String> {
    let prefix = format!("{}://{}/", SCHEME, SOURCE_TYPE);
    if uri.starts_with(&prefix) {
        Some(uri[prefix.len()..].to_string())
    } else {
        None
    }
}

/// Creates a Resource for a specific source file.
pub fn source_file_resource(relative_path: &str) -> Resource {
    RawResource {
        uri: source_file_uri(relative_path),
        name: relative_path.to_string(),
        title: None,
        description: Some(format!("Firm source file: {}", relative_path)),
        mime_type: Some("text/plain".to_string()),
        size: None,
        icons: None,
        meta: None,
    }
    .no_annotation()
}

/// Converts an absolute path to a relative path within the workspace.
pub fn to_relative_path(workspace_path: &Path, absolute_path: &Path) -> Option<String> {
    absolute_path
        .strip_prefix(workspace_path)
        .ok()
        .map(|p| p.to_string_lossy().to_string())
}

/// Converts a relative path to an absolute path within the workspace.
///
/// Returns `None` if the resulting path would be outside the workspace
/// (e.g., if the relative path contains `..` that escapes).
pub fn to_absolute_path(workspace_path: &Path, relative_path: &str) -> Option<PathBuf> {
    let absolute = workspace_path.join(relative_path);

    // Canonicalize to resolve any `..` components, then verify it's still in workspace
    // Note: The file must exist for canonicalize to work
    if absolute.exists() {
        match absolute.canonicalize() {
            Ok(canonical) => {
                let workspace_canonical = workspace_path.canonicalize().ok()?;
                if canonical.starts_with(&workspace_canonical) {
                    Some(canonical)
                } else {
                    None // Path escapes workspace
                }
            }
            Err(_) => None,
        }
    } else {
        // For non-existent files, do a simple check (less secure but allows write_source for new files)
        if relative_path.contains("..") {
            None
        } else {
            Some(absolute)
        }
    }
}

/// Reads the contents of a source file.
pub fn read_source_file(workspace_path: &Path, relative_path: &str) -> Result<String, String> {
    let absolute_path = to_absolute_path(workspace_path, relative_path)
        .ok_or_else(|| format!("Invalid path: {}", relative_path))?;

    if !absolute_path.exists() {
        return Err(format!("File not found: {}", relative_path));
    }

    if absolute_path.extension().is_none_or(|ext| ext != "firm") {
        return Err(format!("Not a .firm file: {}", relative_path));
    }

    fs::read_to_string(&absolute_path).map_err(|e| format!("Failed to read file: {}", e))
}

/// Writes content to a source file.
///
/// Creates parent directories if they don't exist.
/// Only allows writing to .firm files within the workspace.
pub fn write_source_file(
    workspace_path: &Path,
    relative_path: &str,
    content: &str,
) -> Result<(), String> {
    // Validate the path
    if !relative_path.ends_with(".firm") {
        return Err(format!(
            "Path must end with .firm extension: {}",
            relative_path
        ));
    }

    let absolute_path = to_absolute_path(workspace_path, relative_path)
        .ok_or_else(|| format!("Invalid path (must be within workspace): {}", relative_path))?;

    // Create parent directories if needed
    if let Some(parent) = absolute_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create parent directories: {}", e))?;
    }

    fs::write(&absolute_path, content).map_err(|e| format!("Failed to write file: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_file_uri() {
        assert_eq!(
            source_file_uri("core/people.firm"),
            "firm://source/core/people.firm"
        );
    }

    #[test]
    fn test_parse_source_uri() {
        assert_eq!(
            parse_source_uri("firm://source/core/people.firm"),
            Some("core/people.firm".to_string())
        );
        assert_eq!(parse_source_uri("firm://source"), None);
        assert_eq!(parse_source_uri("other://source/file.firm"), None);
    }

    #[test]
    fn test_to_relative_path() {
        let workspace = Path::new("/workspace");
        let absolute = Path::new("/workspace/core/people.firm");
        assert_eq!(
            to_relative_path(workspace, absolute),
            Some("core/people.firm".to_string())
        );
    }
}
