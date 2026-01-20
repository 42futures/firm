use std::path::PathBuf;

use tree_sitter::Tree;

use super::{ParsedEntity, ParsedSchema};

const ENTITY_BLOCK_KIND: &str = "entity_block";
const SCHEMA_BLOCK_KIND: &str = "schema_block";

/// A parsed Firm DSL source document.
///
/// Contains the original source text and the tree-sitter parse tree,
/// providing access to entities and syntax error detection.
#[derive(Debug)]
pub struct ParsedSource {
    /// The plain text source file.
    pub source: String,

    /// The parsed syntax tree.
    pub tree: Tree,

    /// The workspace-relative path to this source file.
    pub path: PathBuf,
}

impl ParsedSource {
    /// Creates a new ParsedSource from source text and parse tree.
    pub fn new(source: String, tree: Tree, path: PathBuf) -> Self {
        Self { source, tree, path }
    }

    /// Check if the source contains syntax errors.
    pub fn has_error(&self) -> bool {
        self.tree.root_node().has_error()
    }

    /// Extracts all entity definitions from the parsed source.
    pub fn entities(&self) -> Vec<ParsedEntity<'_>> {
        let mut entities = Vec::new();
        let root = self.tree.root_node();
        let mut cursor = root.walk();

        for child in root.children(&mut cursor) {
            if child.kind() == ENTITY_BLOCK_KIND {
                entities.push(ParsedEntity::new(child, &self.source, &self.path));
            }
        }

        entities
    }

    /// Extracts all schema definitions from the parsed source.
    pub fn schemas(&self) -> Vec<ParsedSchema<'_>> {
        let mut schemas = Vec::new();
        let root = self.tree.root_node();
        let mut cursor = root.walk();

        for child in root.children(&mut cursor) {
            if child.kind() == SCHEMA_BLOCK_KIND {
                schemas.push(ParsedSchema::new(child, &self.source, &self.path));
            }
        }

        schemas
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::parser::dsl::parse_source;

    #[test]
    fn test_has_entities_for_valid_source() {
        let source = r#"
            role cto {
                name = "CTO"
                executive = true
            }

            contact john_doe {
                name = "John Doe"
                role = role.cto
            }
        "#;

        let parsed = parse_source(String::from(source), None).unwrap();

        assert!(!parsed.has_error());

        let entities = parsed.entities();
        assert!(entities.len() == 2);
    }

    #[test]
    fn test_has_schemas_for_valid_source() {
        let source = r#"
            schema project {
                field {
                    name = "title"
                    type = "string"
                    required = true
                }

                field {
                    name = "priority"
                    type = "integer"
                    required = false
                }
            }

            contact john_doe {
                name = "John Doe"
            }
        "#;

        let parsed = parse_source(String::from(source), None).unwrap();

        assert!(!parsed.has_error());

        let schemas = parsed.schemas();
        assert_eq!(schemas.len(), 1);

        let schema = &schemas[0];
        assert_eq!(schema.name(), Some("project"));

        let fields = schema.fields();
        assert_eq!(fields.len(), 2);

        // Test first field
        let field1 = &fields[0];
        assert_eq!(field1.name().unwrap(), "title");
        assert_eq!(field1.field_type().unwrap(), "string");
        assert_eq!(field1.required(), true);

        // Test second field
        let field2 = &fields[1];
        assert_eq!(field2.name().unwrap(), "priority");
        assert_eq!(field2.field_type().unwrap(), "integer");
        assert_eq!(field2.required(), false);
    }

    #[test]
    fn test_no_error_for_valid_source() {
        let source = r#"
            // Some sort of tech boss?
            role cto {
                name = "CTO"
                executive = true
            }

            // A person I know
            contact john_doe {
                name = "John Doe"
                role = role.cto
            }
        "#;

        let parsed = parse_source(String::from(source), None).unwrap();
        assert!(!parsed.has_error());
    }

    #[test]
    fn test_error_for_incomplete_entity_block() {
        let source = r#"
            role cto {
                name = "CTO"
                executive = true

                // Entity block is incomplete...
        "#;

        let parsed = parse_source(String::from(source), None).unwrap();
        assert!(parsed.has_error());
    }

    #[test]
    fn test_error_for_malformed_reference() {
        let source = r#"
            contact test {
                bad_ref = contact.too.many.parts.here
            }
        "#;

        let parsed = parse_source(String::from(source), None).unwrap();
        assert!(parsed.has_error());
    }

    #[test]
    fn test_error_for_malformed_number() {
        let source = r#"
            contact test {
                bad_number = 42.3.4
            }
        "#;

        let parsed = parse_source(String::from(source), None).unwrap();
        assert!(parsed.has_error());
    }

    #[test]
    fn test_error_for_missing_field_value() {
        let source = r#"
            contact test {
                name =
            }
        "#;

        let parsed = parse_source(String::from(source), None).unwrap();
        assert!(parsed.has_error());
    }

    #[test]
    fn test_error_for_missing_entity_id() {
        let source = r#"
            contact {
                name = "Test"
            }
        "#;

        let parsed = parse_source(String::from(source), None).unwrap();
        assert!(parsed.has_error());
    }

    #[test]
    fn test_error_for_unclosed_string() {
        let source = r#"
            contact test {
                name = "Unclosed string
            }
        "#;

        let parsed = parse_source(String::from(source), None).unwrap();
        assert!(parsed.has_error());
    }

    #[test]
    fn test_schema_with_complex_fields() {
        let source = r#"
            schema project {
                field {
                    name = "title"
                    type = "string"
                    required = true
                }

                field {
                    name = "priority"
                    type = "integer"
                    required = false
                }

                field {
                    name = "budget"
                    type = "currency"
                    required = false
                }
            }

            schema invoice {
                field {
                    name = "amount"
                    type = "currency"
                    required = true
                }

                field {
                    name = "description"
                    type = "string"
                }
            }
        "#;

        let parsed = parse_source(String::from(source), None).unwrap();
        assert!(!parsed.has_error());

        let schemas = parsed.schemas();
        assert_eq!(schemas.len(), 2);

        // Test first schema
        let project_schema = &schemas[0];
        assert_eq!(project_schema.name(), Some("project"));
        let project_fields = project_schema.fields();
        assert_eq!(project_fields.len(), 3);

        // Test required field
        let title_field = &project_fields[0];
        assert_eq!(title_field.name().unwrap(), "title");
        assert_eq!(title_field.field_type().unwrap(), "string");
        assert_eq!(title_field.required(), true);

        // Test optional field
        let priority_field = &project_fields[1];
        assert_eq!(priority_field.name().unwrap(), "priority");
        assert_eq!(priority_field.field_type().unwrap(), "integer");
        assert_eq!(priority_field.required(), false);

        // Test second schema
        let invoice_schema = &schemas[1];
        assert_eq!(invoice_schema.name(), Some("invoice"));
        let invoice_fields = invoice_schema.fields();
        assert_eq!(invoice_fields.len(), 2);

        // Test field without explicit required (should default to false)
        let description_field = &invoice_fields[1];
        assert_eq!(description_field.name().unwrap(), "description");
        assert_eq!(description_field.field_type().unwrap(), "string");
        assert_eq!(description_field.required(), false);
    }

    #[test]
    fn test_passes_default_path() {
        let parsed = parse_source(String::new(), None).unwrap();
        assert!(parsed.path == PathBuf::new());
    }

    #[test]
    fn test_passes_custom_path() {
        let path = PathBuf::from("./subdirectory/file.firm");
        let parsed = parse_source(String::new(), Some(path.clone())).unwrap();
        assert_eq!(parsed.path, path);
    }
}
