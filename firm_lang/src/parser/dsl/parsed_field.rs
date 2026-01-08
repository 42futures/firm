use std::path::PathBuf;

use tree_sitter::Node;

use super::{
    parsed_value::ParsedValue, parser_errors::ValueParseError, parser_utils::find_child_of_kind,
    parser_utils::get_node_text,
};

const FIELD_ID_KIND: &str = "field_name";
const VALUE_KIND: &str = "value";

/// A parsed field definition from an entity block.
///
/// Represents a field assignment like `name = "John Doe"` with
/// access to the field name and parsed value.
#[derive(Debug)]
pub struct ParsedField<'a> {
    node: Node<'a>,
    source: &'a str,
    path: &'a PathBuf,
}

impl<'a> ParsedField<'a> {
    /// Creates a new ParsedField from a tree-sitter node and source text.
    pub fn new(node: Node<'a>, source: &'a str, path: &'a PathBuf) -> Self {
        Self { node, source, path }
    }

    /// Gets the field name (e.g., "name", "age").
    pub fn id(&self) -> Option<&str> {
        let id_node = find_child_of_kind(&self.node, FIELD_ID_KIND)?;
        Some(get_node_text(&id_node, self.source))
    }

    /// Parses and gets the field's value with full type information.
    pub fn value(&self) -> Result<ParsedValue, ValueParseError> {
        let value_node =
            find_child_of_kind(&self.node, VALUE_KIND).ok_or(ValueParseError::MissingValue)?;

        ParsedValue::from_node(value_node, self.source, self.path)
    }
}
