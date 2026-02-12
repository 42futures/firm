use std::path::PathBuf;

use tree_sitter::Node;

use super::{
    ParsedField,
    parser_utils::{find_child_of_kind, get_node_text},
};

const ENTITY_TYPE_KIND: &str = "entity_type";
const ENTITY_ID_KIND: &str = "entity_id";
const FIELD_KIND: &str = "field";

/// A parsed entity definition from Firm DSL.
///
/// Represents an entity block like `contact john_doe { ... }` with
/// access to the entity type, ID, and contained fields.
#[derive(Debug)]
pub struct ParsedEntity<'a> {
    node: Node<'a>,
    source: &'a str,
    path: &'a PathBuf,
}

impl<'a> ParsedEntity<'a> {
    /// Creates a new ParsedEntity from a tree-sitter node and source text.
    pub fn new(node: Node<'a>, source: &'a str, path: &'a PathBuf) -> Self {
        Self { node, source, path }
    }

    /// Returns the entity type (e.g., "contact", "role").
    pub fn entity_type(&self) -> Option<&str> {
        let type_node = find_child_of_kind(&self.node, ENTITY_TYPE_KIND)?;
        Some(get_node_text(&type_node, self.source))
    }

    /// Returns the entity ID (e.g., "john_doe", "cto").
    pub fn id(&self) -> Option<&str> {
        let id_node = find_child_of_kind(&self.node, ENTITY_ID_KIND)?;
        Some(get_node_text(&id_node, self.source))
    }

    /// Returns the underlying tree-sitter node.
    pub fn node(&self) -> Node<'a> {
        self.node
    }

    /// Returns the file path this entity was parsed from.
    pub fn path(&self) -> &PathBuf {
        self.path
    }

    /// Extracts all field definitions from the entity block.
    pub fn fields(&self) -> Vec<ParsedField<'_>> {
        let mut fields = Vec::new();
        let mut cursor = self.node.walk();

        // First find the block node
        if let Some(block_node) = self
            .node
            .children(&mut cursor)
            .find(|child| child.kind() == "block")
        {
            let mut block_cursor = block_node.walk();

            // Then find field nodes within the block
            for child in block_node.children(&mut block_cursor) {
                if child.kind() == FIELD_KIND {
                    fields.push(ParsedField::new(child, self.source, self.path));
                }
            }
        }

        fields
    }
}
