use tree_sitter::Node;

/// Finds the first child node of a specific kind.
pub fn find_child_of_kind<'a>(node: &Node<'a>, kind: &str) -> Option<Node<'a>> {
    let mut cursor = node.walk();

    node.children(&mut cursor)
        .find(|child| child.kind() == kind)
}

/// Extracts the text content of a node from the source string.
pub fn get_node_text<'a>(node: &Node<'a>, source: &'a str) -> &'a str {
    &source[node.byte_range()]
}
