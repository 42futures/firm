//! Syntax error detection by walking tree-sitter ERROR and MISSING nodes.

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
        let text = &source[node.byte_range()];
        let preview = truncate(text, 30);
        diagnostics.push(Diagnostic {
            message: format!("Syntax error: unexpected `{preview}`"),
            severity: DiagnosticSeverity::Error,
            span: SourceSpan::from_node(node, file),
        });
        return; // Don't recurse into ERROR nodes
    }

    if node.is_missing() {
        let kind = node.kind();
        diagnostics.push(Diagnostic {
            message: format!("Syntax error: missing {kind}"),
            severity: DiagnosticSeverity::Error,
            span: SourceSpan::from_node(node, file),
        });
        return;
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.has_error() || child.is_error() || child.is_missing() {
            collect_errors_recursive(&child, source, file, diagnostics);
        }
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    let trimmed = s.trim();
    if trimmed.len() <= max_len {
        trimmed.to_string()
    } else {
        format!("{}...", &trimmed[..max_len])
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::dsl::parse_source;

    use super::*;
    use std::path::PathBuf;

    fn diagnostics_for(source: &str) -> Vec<Diagnostic> {
        let parsed = parse_source(String::from(source), Some(PathBuf::from("test.firm"))).unwrap();
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
    fn test_error_for_missing_closing_brace() {
        let diagnostics = diagnostics_for(
            r#"
            contact john_doe {
                name = "John Doe"
        "#,
        );
        assert!(!diagnostics.is_empty());
        assert!(diagnostics.iter().all(|d| d.severity == DiagnosticSeverity::Error));
    }

    #[test]
    fn test_error_for_missing_value() {
        let diagnostics = diagnostics_for(
            r#"
            contact test {
                name =
            }
        "#,
        );
        assert!(!diagnostics.is_empty());
    }

    #[test]
    fn test_error_for_missing_entity_id() {
        let diagnostics = diagnostics_for(
            r#"
            contact {
                name = "Test"
            }
        "#,
        );
        assert!(!diagnostics.is_empty());
    }

    #[test]
    fn test_error_includes_file_path() {
        let diagnostics = diagnostics_for(
            r#"
            contact {
                name = "Test"
            }
        "#,
        );
        assert!(!diagnostics.is_empty());
        assert_eq!(diagnostics[0].span.file, PathBuf::from("test.firm"));
    }

    #[test]
    fn test_error_has_line_info() {
        let diagnostics = diagnostics_for(
            r#"contact test {
    name =
}"#,
        );
        assert!(!diagnostics.is_empty());
        // The missing value is on line 1 (0-indexed)
        assert!(diagnostics[0].span.start_line >= 1);
    }
}
