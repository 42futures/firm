use std::path::PathBuf;

use tree_sitter::Node;

use super::{
    ParsedSchemaField,
    parser_utils::{find_child_of_kind, get_node_text},
};

const SCHEMA_NAME_KIND: &str = "schema_name";
const NESTED_BLOCK_KIND: &str = "nested_block";

/// A parsed schema definition from Firm DSL.
///
/// Represents a schema block like `schema project { ... }` with
/// access to the schema name and contained field definitions.
#[derive(Debug)]
pub struct ParsedSchema<'a> {
    node: Node<'a>,
    source: &'a str,
    path: &'a PathBuf,
}

impl<'a> ParsedSchema<'a> {
    /// Creates a new ParsedSchema from a tree-sitter node and source text.
    pub fn new(node: Node<'a>, source: &'a str, path: &'a PathBuf) -> Self {
        Self { node, source, path }
    }

    /// Gets the schema name (e.g., "project", "invoice").
    pub fn name(&self) -> Option<&str> {
        let name_node = find_child_of_kind(&self.node, SCHEMA_NAME_KIND)?;
        Some(get_node_text(&name_node, self.source))
    }

    /// Extracts all field definitions from the schema block.
    pub fn fields(&self) -> Vec<ParsedSchemaField<'_>> {
        let mut fields = Vec::new();
        let mut cursor = self.node.walk();

        // First find the block node
        if let Some(block_node) = self
            .node
            .children(&mut cursor)
            .find(|child| child.kind() == "block")
        {
            let mut block_cursor = block_node.walk();

            // Then find nested_block nodes within the block (these are the field definitions)
            for child in block_node.children(&mut block_cursor) {
                if child.kind() == NESTED_BLOCK_KIND {
                    // Verify this is a "field" block by checking the block_type
                    let mut nested_cursor = child.walk();
                    if let Some(block_type_node) = child
                        .children(&mut nested_cursor)
                        .find(|c| c.kind() == "block_type")
                    {
                        let block_type = get_node_text(&block_type_node, self.source);
                        if block_type == "field" {
                            fields.push(ParsedSchemaField::new(child, self.source, self.path));
                        }
                    }
                }
            }
        }

        fields
    }
}
