use std::path::PathBuf;

use tree_sitter::{Language, Parser};

use super::LanguageError;
use super::ParsedSource;

/// Gets the tree-sitter language for Firm DSL.
fn language() -> Language {
    tree_sitter_firm::LANGUAGE.into()
}

/// Parses Firm DSL source code into a structured representation.
///
/// This is the main entry point for parsing Firm DSL. It initializes
/// a tree-sitter parser and returns ParsedSource for further processing.
pub fn parse_source(source: String, path: Option<PathBuf>) -> Result<ParsedSource, LanguageError> {
    let mut parser = Parser::new();
    parser
        .set_language(&language())
        .map_err(|_| LanguageError::IncompatibleLanguageVersion)?;

    match parser.parse(&source, None) {
        Some(tree) => Ok(ParsedSource::new(source, tree, path.unwrap_or_default())),
        None => Err(LanguageError::LanguageNotInitialized),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_valid() {
        let source = r#"
            contact john_doe {
                name = "John Doe"
                email = "john@example.com"
                age = 42
            }
        "#;

        let result = parse_source(String::from(source), None);
        assert!(result.is_ok());
    }
}
