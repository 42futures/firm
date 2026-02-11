//! Diagnostic types and collection for Firm DSL files.
//!
//! Currently provides per-file syntax error detection. Workspace-level
//! diagnostics (references, schemas) will be added later.

mod syntax;

pub use syntax::collect_syntax_errors;

use std::path::{Path, PathBuf};

use tree_sitter::Node;

/// A source location span within a file.
#[derive(Debug, Clone, PartialEq)]
pub struct SourceSpan {
    /// Workspace-relative file path.
    pub file: PathBuf,
    /// Start line (0-indexed).
    pub start_line: u32,
    /// Start column (0-indexed).
    pub start_col: u32,
    /// End line (0-indexed).
    pub end_line: u32,
    /// End column (0-indexed).
    pub end_col: u32,
}

impl SourceSpan {
    /// Creates a SourceSpan from a tree-sitter node's position.
    pub fn from_node(node: &Node, file: &Path) -> Self {
        let start = node.start_position();
        let end = node.end_position();
        Self {
            file: file.to_path_buf(),
            start_line: start.row as u32,
            start_col: start.column as u32,
            end_line: end.row as u32,
            end_col: end.column as u32,
        }
    }
}

/// Severity level of a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
}

/// A diagnostic message with source location.
#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub message: String,
    pub severity: DiagnosticSeverity,
    pub span: SourceSpan,
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let severity = match self.severity {
            DiagnosticSeverity::Error => "error",
            DiagnosticSeverity::Warning => "warning",
        };
        write!(
            f,
            "{}:{}:{}: {}: {}",
            self.span.file.display(),
            self.span.start_line + 1,
            self.span.start_col + 1,
            severity,
            self.message,
        )
    }
}
