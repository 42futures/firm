use std::path::PathBuf;
use tree_sitter::Node;

use super::{
    parsed_value::ParsedValue, parser_errors::ValueParseError, parser_utils::find_child_of_kind,
};

const FIELD_KIND: &str = "field";
const BLOCK_KIND: &str = "block";

/// A parsed schema field definition from a schema block.
///
/// Represents a nested field block like:
/// ```text
/// field {
///     name = "title"
///     type = "string"
///     required = true
/// }
/// ```
#[derive(Debug)]
pub struct ParsedSchemaField<'a> {
    node: Node<'a>,
    source: &'a str,
    path: &'a PathBuf,
}

impl<'a> ParsedSchemaField<'a> {
    /// Creates a new ParsedSchemaField from a tree-sitter node and source text.
    pub fn new(node: Node<'a>, source: &'a str, path: &'a PathBuf) -> Self {
        Self { node, source, path }
    }

    /// Gets the field name from the "name" field.
    pub fn name(&self) -> Result<String, ValueParseError> {
        let name_field = self
            .find_field_by_name("name")
            .ok_or(ValueParseError::MissingValue)?;

        match name_field.value()? {
            ParsedValue::String(s) => Ok(s),
            _ => Err(ValueParseError::UnknownValueKind),
        }
    }

    /// Gets the field type from the "type" field.
    pub fn field_type(&self) -> Result<String, ValueParseError> {
        let type_field = self
            .find_field_by_name("type")
            .ok_or(ValueParseError::MissingValue)?;

        match type_field.value()? {
            ParsedValue::String(s) => Ok(s),
            _ => Err(ValueParseError::UnknownValueKind),
        }
    }

    /// Checks whether the field is required or not.
    /// Defaults to false if not specified.
    pub fn required(&self) -> bool {
        if let Some(required_field) = self.find_field_by_name("required") {
            if let Ok(ParsedValue::Boolean(b)) = required_field.value() {
                return b;
            }
        }

        false // Default to false if not specified or invalid
    }

    /// Gets the allowed values for enum fields from the "values" field.
    /// Returns None if not specified or if it's not a list of strings.
    pub fn allowed_values(&self) -> Option<Vec<String>> {
        let values_field = self.find_field_by_name("allowed_values")?;

        match values_field.value() {
            Ok(ParsedValue::List(items)) => {
                let mut strings = Vec::new();
                for item in items {
                    if let ParsedValue::String(s) = item {
                        strings.push(s);
                    } else {
                        // If any item is not a string, enum is invalid
                        return None;
                    }
                }
                Some(strings)
            }
            _ => None,
        }
    }

    /// Helper method to find a field by name within this schema field block.
    fn find_field_by_name(&self, field_name: &str) -> Option<super::ParsedField<'_>> {
        // Find the block node within this field
        let block_node = find_child_of_kind(&self.node, BLOCK_KIND)?;
        let mut cursor = block_node.walk();

        // Look for field assignments within the block
        for child in block_node.children(&mut cursor) {
            if child.kind() == FIELD_KIND {
                let field = super::ParsedField::new(child, self.source, self.path);
                if let Some(id) = field.id() {
                    if id == field_name {
                        return Some(field);
                    }
                }
            }
        }

        None
    }
}
