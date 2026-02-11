//! Syntax error detection by walking tree-sitter ERROR and MISSING nodes.
//!
//! Uses tree structure context to produce targeted error messages and
//! clamps large error spans to avoid underlining the entire file.

use std::path::Path;

use tree_sitter::Node;

use crate::parser::dsl::ParsedSource;

use super::{Diagnostic, DiagnosticSeverity, SourceSpan};

/// Collects syntax errors from a parsed source file.
pub fn collect_syntax_errors(parsed: &ParsedSource) -> Vec<Diagnostic> {
    let root = parsed.tree.root_node();
    let mut diagnostics = Vec::new();
    collect_errors_recursive(&root, &parsed.source, &parsed.path, &mut diagnostics);
    diagnostics
}

fn collect_errors_recursive(
    node: &Node,
    source: &str,
    file: &Path,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if node.is_error() {
        diagnostics.push(make_error_diagnostic(node, source, file));
        return;
    }

    if node.is_missing() {
        diagnostics.push(make_missing_diagnostic(node, file));
        return;
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.has_error() || child.is_error() || child.is_missing() {
            collect_errors_recursive(&child, source, file, diagnostics);
        }
    }
}

/// Produce a diagnostic for an ERROR node.
fn make_error_diagnostic(node: &Node, source: &str, file: &Path) -> Diagnostic {
    let span = clamped_span(node, source, file);

    // Detect missing value: tree-sitter swallows the next field's `name =`
    // as an ERROR child of the current field when a value is missing.
    if is_swallowed_field(node) {
        let diag_span = node
            .parent()
            .and_then(|p| find_child_of_kind(&p, "="))
            .map(|eq| SourceSpan::from_node(&eq, file))
            .unwrap_or(span);
        return Diagnostic {
            message: "Expected a value".to_string(),
            severity: DiagnosticSeverity::Error,
            span: diag_span,
        };
    }

    Diagnostic {
        message: "Syntax error".to_string(),
        severity: DiagnosticSeverity::Error,
        span,
    }
}

/// Produce a diagnostic for a MISSING node (expected but not found).
fn make_missing_diagnostic(node: &Node, file: &Path) -> Diagnostic {
    let message = match node.kind() {
        // Value alternatives — tree-sitter picks the first grammar alternative
        // (boolean > "true") but the user just needs to know a value is expected
        "true" | "false" | "boolean" => "Expected a value".to_string(),

        // Closing delimiters
        "}" => "Missing closing `}`".to_string(),
        "]" => "Missing closing `]`".to_string(),
        "\"" => "Unclosed string literal".to_string(),

        // Identifiers — use parent context
        "identifier" => missing_identifier_message(node),

        kind => format!("Expected {kind}"),
    };

    Diagnostic {
        message,
        severity: DiagnosticSeverity::Error,
        span: SourceSpan::from_node(node, file),
    }
}

/// Use the parent node to describe which identifier is missing.
fn missing_identifier_message(node: &Node) -> String {
    if let Some(parent) = node.parent() {
        match parent.kind() {
            "entity_id" => return "Missing entity ID".to_string(),
            "entity_type" => return "Missing entity type".to_string(),
            "schema_name" => return "Missing schema name".to_string(),
            "field_name" => return "Missing field name".to_string(),
            _ => {}
        }
    }
    "Missing identifier".to_string()
}

/// Clamp a span to at most the first line of the node.
/// Prevents large ERROR nodes from underlining the entire file.
fn clamped_span(node: &Node, source: &str, file: &Path) -> SourceSpan {
    let start = node.start_position();
    let end = node.end_position();

    if end.row > start.row {
        // Multi-line error: clamp to the full first line
        let line_end = source[node.start_byte()..]
            .find('\n')
            .map(|i| node.start_byte() + i)
            .unwrap_or(source.len());
        let end_col = line_end - (node.start_byte() - start.column);
        SourceSpan {
            file: file.to_path_buf(),
            start_line: start.row as u32,
            start_col: start.column as u32,
            end_line: start.row as u32,
            end_col: end_col as u32,
        }
    } else {
        SourceSpan::from_node(node, file)
    }
}

/// Check if an ERROR node looks like a swallowed next field assignment.
/// Pattern: parent is "field", ERROR contains identifier + "=".
/// This happens when `name =` is missing its value and tree-sitter
/// consumes `next_field = value` as a broken value.
fn is_swallowed_field(node: &Node) -> bool {
    let parent = match node.parent() {
        Some(p) if p.kind() == "field" => p,
        _ => return false,
    };
    // The parent field should have an "=" (the field we're inside)
    if find_child_of_kind(&parent, "=").is_none() {
        return false;
    }
    // And the ERROR itself should contain an identifier and "="
    // (the swallowed next field's name and assignment)
    has_child_kind(node, "identifier") && has_child_kind(node, "=")
}

fn find_child_of_kind<'a>(node: &Node<'a>, kind: &str) -> Option<Node<'a>> {
    let mut cursor = node.walk();
    node.children(&mut cursor).find(|c| c.kind() == kind)
}

fn has_child_kind(node: &Node, kind: &str) -> bool {
    find_child_of_kind(node, kind).is_some()
}

#[cfg(test)]
mod tests {
    use crate::parser::dsl::parse_source;

    use super::*;
    use std::path::PathBuf;

    fn diagnostics_for(source: &str) -> Vec<Diagnostic> {
        let parsed =
            parse_source(String::from(source), Some(PathBuf::from("test.firm"))).unwrap();
        collect_syntax_errors(&parsed)
    }

    #[test]
    fn test_no_errors_for_valid_source() {
        let diagnostics = diagnostics_for(
            r#"
            contact john_doe {
                name = "John Doe"
                age = 42
            }
        "#,
        );
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_unclosed_brace_at_eof() {
        let diagnostics = diagnostics_for("contact john {\n    name = \"John\"");
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message, "Missing closing `}`");
    }

    #[test]
    fn test_unclosed_brace_with_following_entity() {
        let diagnostics =
            diagnostics_for("contact john {\n    name = \"John\"\n\ncontact jane {\n    name = \"Jane\"\n}");
        // Should include a missing } diagnostic
        assert!(diagnostics.iter().any(|d| d.message.contains("}")));
    }

    #[test]
    fn test_unclosed_string() {
        let diagnostics =
            diagnostics_for("contact john {\n    name = \"John\n    age = 42\n}");
        assert!(!diagnostics.is_empty());
        // Should be clamped to first line, not span the whole file
        assert_eq!(diagnostics[0].span.start_line, diagnostics[0].span.end_line);
    }

    #[test]
    fn test_missing_value_at_end() {
        let diagnostics = diagnostics_for("contact john {\n    name =\n}");
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message, "Expected a value");
    }

    #[test]
    fn test_missing_value_with_following_field() {
        // Real editing: missing value with more fields after
        let diagnostics = diagnostics_for("contact john {\n    name =\n    age = 42\n}");
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message, "Expected a value");
    }

    #[test]
    fn test_missing_value_mid_block() {
        let diagnostics = diagnostics_for(
            "contact john {\n    title = \"CTO\"\n    name =\n    age = 42\n}",
        );
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message, "Expected a value");
    }

    #[test]
    fn test_missing_entity_id() {
        let diagnostics = diagnostics_for("contact {\n    name = \"Test\"\n}");
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message, "Missing entity ID");
    }

    #[test]
    fn test_missing_field_name() {
        let diagnostics = diagnostics_for("contact john {\n    = \"Test\"\n}");
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message, "Syntax error");
    }

    #[test]
    fn test_error_includes_file_path() {
        let diagnostics = diagnostics_for("contact {\n    name = \"Test\"\n}");
        assert_eq!(diagnostics[0].span.file, PathBuf::from("test.firm"));
    }

    #[test]
    fn test_unclosed_string_mid_entity() {
        let diagnostics = diagnostics_for(
            "contact john {\n    title = \"CTO\"\n    name = \"John\n    age = 42\n}",
        );
        assert!(!diagnostics.is_empty());
        // Clamped to one line
        assert_eq!(diagnostics[0].span.start_line, diagnostics[0].span.end_line);
    }

    #[test]
    fn test_unclosed_string_first_field() {
        let diagnostics = diagnostics_for("contact john {\n    name = \"John\n}");
        assert!(!diagnostics.is_empty());
        assert_eq!(diagnostics[0].span.start_line, diagnostics[0].span.end_line);
    }

    #[test]
    fn test_error_spans_full_line() {
        // Multi-line ERROR node should be clamped to the full first line
        let diagnostics =
            diagnostics_for("contact john {\n    name = \"John\n    age = 42\n}");
        assert!(!diagnostics.is_empty());
        let span = &diagnostics[0].span;
        assert_eq!(span.start_line, span.end_line);
        // Should span more than 1 character (the whole line content)
        assert!(span.end_col > span.start_col + 1);
    }
}
