//! Workspace-level diagnostics: value parse errors, schema validation,
//! missing schemas, and broken references.
//!
//! These diagnostics require the full workspace context (all files loaded)
//! and use firm_core's validation, mapping results back to source positions
//! via entity_id + field_id lookups.

use std::collections::HashMap;

use firm_core::{Entity, EntitySchema, EntityType, FieldId, compose_entity_id};

use crate::parser::dsl::{ParsedEntity, ParsedField, ParsedSource, ParsedValue};
use crate::workspace::Workspace;

use super::{Diagnostic, DiagnosticSeverity, SourceSpan};

/// Collects workspace-level diagnostics from all loaded files.
///
/// Performs three passes:
/// 1. Build schemas from parsed schema definitions
/// 2. Check entity field values, run schema validation, collect references
/// 3. Validate references against known entity IDs
pub fn collect_workspace_diagnostics(workspace: &Workspace) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let parsed_sources = workspace.parsed_sources();

    // Pass 1: Build schemas
    let schemas = collect_schemas(&parsed_sources, &mut diagnostics);

    // Pass 2: Process entities — value checks, schema validation, collect references
    let mut built_entities: HashMap<String, Entity> = HashMap::new();
    let mut pending_references: Vec<PendingReference> = Vec::new();

    for parsed_source in &parsed_sources {
        for parsed_entity in parsed_source.entities() {
            process_entity(
                &parsed_entity,
                &schemas,
                &mut built_entities,
                &mut pending_references,
                &mut diagnostics,
            );
        }
    }

    // Pass 3: Validate references
    check_references(&pending_references, &built_entities, &mut diagnostics);

    diagnostics
}

/// A reference found during entity processing, pending validation.
struct PendingReference {
    /// The composite ID of the referenced entity (e.g. "contact.john_doe").
    target_entity_id: String,
    /// For field references, the field ID being referenced.
    target_field_id: Option<String>,
    /// Source span pointing at the reference value node.
    span: SourceSpan,
}

/// Pass 1: Convert all ParsedSchemas to EntitySchemas, emitting diagnostics for failures.
fn collect_schemas(
    parsed_sources: &[&ParsedSource],
    diagnostics: &mut Vec<Diagnostic>,
) -> HashMap<EntityType, EntitySchema> {
    let mut schemas = HashMap::new();

    for parsed_source in parsed_sources {
        for parsed_schema in parsed_source.schemas() {
            match EntitySchema::try_from(&parsed_schema) {
                Ok(schema) => {
                    schemas.insert(schema.entity_type.clone(), schema);
                }
                Err(err) => {
                    // Point at the schema node if available
                    let span = SourceSpan::from_node(&parsed_schema.node(), &parsed_source.path);
                    diagnostics.push(Diagnostic {
                        message: format!("Invalid schema: {}", err),
                        severity: DiagnosticSeverity::Error,
                        span,
                    });
                }
            }
        }
    }

    schemas
}

/// Pass 2: Process a single entity — check values, validate against schema, collect references.
fn process_entity(
    parsed_entity: &ParsedEntity,
    schemas: &HashMap<EntityType, EntitySchema>,
    built_entities: &mut HashMap<String, Entity>,
    pending_references: &mut Vec<PendingReference>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let entity_type_str = match parsed_entity.entity_type() {
        Some(t) => t,
        None => return, // Syntax error, handled by syntax diagnostics
    };

    if parsed_entity.id().is_none() {
        return; // Syntax error
    }

    // Check field values and collect references
    let fields = parsed_entity.fields();
    let mut has_value_errors = false;

    for field in &fields {
        match check_field_value(field, pending_references) {
            Ok(()) => {}
            Err(diag) => {
                has_value_errors = true;
                diagnostics.push(diag);
            }
        }
    }

    // Skip schema validation if any field values failed to parse — the entity
    // can't be converted to Entity. Fix value errors first, then schema errors appear.
    if has_value_errors {
        return;
    }

    // Try to convert the ParsedEntity to an Entity for schema validation
    let entity = match Entity::try_from(parsed_entity) {
        Ok(entity) => entity,
        Err(_) => return, // Conversion failed for other reasons
    };

    // Store the built entity for field reference checking
    built_entities.insert(entity.id.to_string(), entity.clone());

    // Check for missing schema
    let entity_type = EntityType::new(entity_type_str);
    let schema = match schemas.get(&entity_type) {
        Some(s) => s,
        None => {
            diagnostics.push(Diagnostic {
                message: format!("No schema defined for entity type '{}'", entity_type_str),
                severity: DiagnosticSeverity::Error,
                span: SourceSpan::from_node(&parsed_entity.node(), parsed_entity.path()),
            });
            return;
        }
    };

    // Run schema validation
    if let Err(validation_errors) = schema.validate(&entity) {
        for error in validation_errors {
            let span = match &error.field {
                Some(field_id) => {
                    // Try to find the matching ParsedField for this field
                    match find_parsed_field(&fields, field_id.as_str()) {
                        Some(parsed_field) => {
                            // For MismatchedFieldType or InvalidEnumValue, point at the value
                            match &error.error_type {
                                firm_core::schema::ValidationErrorType::MismatchedFieldType {
                                    ..
                                }
                                | firm_core::schema::ValidationErrorType::InvalidEnumValue {
                                    ..
                                } => parsed_field
                                    .value_node()
                                    .map(|n| SourceSpan::from_node(&n, parsed_entity.path()))
                                    .unwrap_or_else(|| {
                                        SourceSpan::from_node(
                                            &parsed_field.node(),
                                            parsed_entity.path(),
                                        )
                                    }),
                                // For MissingRequiredField — the field exists but shouldn't reach here
                                _ => SourceSpan::from_node(
                                    &parsed_field.node(),
                                    parsed_entity.path(),
                                ),
                            }
                        }
                        None => {
                            // Field not found in parsed entity (e.g. MissingRequiredField)
                            // Point at the entity node
                            SourceSpan::from_node(&parsed_entity.node(), parsed_entity.path())
                        }
                    }
                }
                None => {
                    // No field specified — point at the entity node
                    SourceSpan::from_node(&parsed_entity.node(), parsed_entity.path())
                }
            };

            diagnostics.push(Diagnostic {
                message: error.message,
                severity: DiagnosticSeverity::Error,
                span,
            });
        }
    }
}

/// Check a single field's value. Returns Ok if the value parses successfully,
/// or Err with a diagnostic if it fails. Also collects reference values for later checking.
fn check_field_value(
    field: &ParsedField,
    pending_references: &mut Vec<PendingReference>,
) -> Result<(), Diagnostic> {
    let value = match field.value() {
        Ok(v) => v,
        Err(err) => {
            // Point at the value node if available, otherwise at the field node
            let span = field
                .value_node()
                .map(|n| SourceSpan::from_node(&n, field.path()))
                .unwrap_or_else(|| SourceSpan::from_node(&field.node(), field.path()));

            return Err(Diagnostic {
                message: err.to_string(),
                severity: DiagnosticSeverity::Error,
                span,
            });
        }
    };

    // Collect references for later checking
    collect_references_from_value(&value, field, pending_references);

    Ok(())
}

/// Extract reference values from a parsed value and register them for checking.
fn collect_references_from_value(
    value: &ParsedValue,
    field: &ParsedField,
    pending_references: &mut Vec<PendingReference>,
) {
    match value {
        ParsedValue::EntityReference {
            entity_type,
            entity_id,
        } => {
            let composite_id = compose_entity_id(entity_type, entity_id);
            let span = field
                .value_node()
                .map(|n| SourceSpan::from_node(&n, field.path()))
                .unwrap_or_else(|| SourceSpan::from_node(&field.node(), field.path()));

            pending_references.push(PendingReference {
                target_entity_id: composite_id.to_string(),
                target_field_id: None,
                span,
            });
        }
        ParsedValue::FieldReference {
            entity_type,
            entity_id,
            field_id,
        } => {
            let composite_id = compose_entity_id(entity_type, entity_id);
            let span = field
                .value_node()
                .map(|n| SourceSpan::from_node(&n, field.path()))
                .unwrap_or_else(|| SourceSpan::from_node(&field.node(), field.path()));

            pending_references.push(PendingReference {
                target_entity_id: composite_id.to_string(),
                target_field_id: Some(field_id.clone()),
                span,
            });
        }
        ParsedValue::List(items) => {
            for item in items {
                collect_references_from_value(item, field, pending_references);
            }
        }
        _ => {}
    }
}

/// Pass 3: Check all collected references against known entities and their fields.
fn check_references(
    pending_references: &[PendingReference],
    built_entities: &HashMap<String, Entity>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for reference in pending_references {
        match built_entities.get(&reference.target_entity_id) {
            None => {
                diagnostics.push(Diagnostic {
                    message: format!(
                        "Reference to unknown entity '{}'",
                        reference.target_entity_id
                    ),
                    severity: DiagnosticSeverity::Error,
                    span: reference.span.clone(),
                });
            }
            Some(entity) => {
                // Entity exists — if this is a field reference, check the field too
                if let Some(field_id) = &reference.target_field_id {
                    if entity.get_field(&FieldId::from(field_id.as_str())).is_none() {
                        diagnostics.push(Diagnostic {
                            message: format!(
                                "Entity '{}' has no field '{}'",
                                reference.target_entity_id, field_id
                            ),
                            severity: DiagnosticSeverity::Error,
                            span: reference.span.clone(),
                        });
                    }
                }
            }
        }
    }
}

/// Find a ParsedField within a list of fields by its field name.
fn find_parsed_field<'a>(
    fields: &'a [ParsedField<'a>],
    field_name: &str,
) -> Option<&'a ParsedField<'a>> {
    fields.iter().find(|f| f.id() == Some(field_name))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::parser::dsl::parse_source;
    use crate::workspace::Workspace;

    use super::*;

    /// Helper: create a workspace from multiple (path, source) pairs.
    fn workspace_from_sources(files: &[(&str, &str)]) -> Workspace {
        let mut workspace = Workspace::new();
        for (path, source) in files {
            let parsed =
                parse_source(String::from(*source), Some(PathBuf::from(path))).unwrap();
            workspace.add_parsed_source(parsed);
        }
        workspace
    }

    #[test]
    fn test_valid_workspace_no_diagnostics() {
        let workspace = workspace_from_sources(&[
            (
                "schemas.firm",
                r#"
                schema contact {
                    field {
                        name = "name"
                        type = "string"
                        required = true
                    }
                }
                "#,
            ),
            (
                "contacts.firm",
                r#"
                contact john_doe {
                    name = "John Doe"
                }
                "#,
            ),
        ]);

        let diagnostics = collect_workspace_diagnostics(&workspace);
        assert!(diagnostics.is_empty(), "Expected no diagnostics, got: {:?}", diagnostics);
    }

    #[test]
    fn test_invalid_currency_code() {
        let workspace = workspace_from_sources(&[
            (
                "schemas.firm",
                r#"
                schema invoice {
                    field {
                        name = "amount"
                        type = "currency"
                        required = true
                    }
                }
                "#,
            ),
            (
                "invoices.firm",
                r#"
                invoice inv_001 {
                    amount = 100.50 XYZ
                }
                "#,
            ),
        ]);

        let diagnostics = collect_workspace_diagnostics(&workspace);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("Currency code"));
        assert_eq!(diagnostics[0].span.file, PathBuf::from("invoices.firm"));
    }

    #[test]
    fn test_missing_required_field() {
        let workspace = workspace_from_sources(&[
            (
                "schemas.firm",
                r#"
                schema contact {
                    field {
                        name = "name"
                        type = "string"
                        required = true
                    }
                    field {
                        name = "email"
                        type = "string"
                        required = true
                    }
                }
                "#,
            ),
            (
                "contacts.firm",
                r#"
                contact john_doe {
                    name = "John Doe"
                }
                "#,
            ),
        ]);

        let diagnostics = collect_workspace_diagnostics(&workspace);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("Missing required field"));
        assert!(diagnostics[0].message.contains("email"));
        // Missing field points at the entity node
        assert_eq!(diagnostics[0].span.file, PathBuf::from("contacts.firm"));
    }

    #[test]
    fn test_mismatched_field_type() {
        let workspace = workspace_from_sources(&[
            (
                "schemas.firm",
                r#"
                schema contact {
                    field {
                        name = "age"
                        type = "integer"
                        required = true
                    }
                }
                "#,
            ),
            (
                "contacts.firm",
                r#"
                contact john_doe {
                    age = "not a number"
                }
                "#,
            ),
        ]);

        let diagnostics = collect_workspace_diagnostics(&workspace);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("type"));
        assert_eq!(diagnostics[0].span.file, PathBuf::from("contacts.firm"));
    }

    #[test]
    fn test_invalid_enum_value() {
        let workspace = workspace_from_sources(&[
            (
                "schemas.firm",
                r#"
                schema account {
                    field {
                        name = "status"
                        type = "enum"
                        required = true
                        allowed_values = ["prospect", "customer", "partner"]
                    }
                }
                "#,
            ),
            (
                "accounts.firm",
                r#"
                account acme {
                    status = enum"invalid_status"
                }
                "#,
            ),
        ]);

        let diagnostics = collect_workspace_diagnostics(&workspace);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("Invalid value"));
        assert!(diagnostics[0].message.contains("invalid_status"));
    }

    #[test]
    fn test_broken_entity_reference() {
        let workspace = workspace_from_sources(&[
            (
                "schemas.firm",
                r#"
                schema contact {
                    field {
                        name = "name"
                        type = "string"
                        required = true
                    }
                    field {
                        name = "manager"
                        type = "reference"
                    }
                }
                "#,
            ),
            (
                "contacts.firm",
                r#"
                contact john_doe {
                    name = "John Doe"
                    manager = contact.nonexistent_person
                }
                "#,
            ),
        ]);

        let diagnostics = collect_workspace_diagnostics(&workspace);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("unknown entity"));
        assert!(diagnostics[0].message.contains("contact.nonexistent_person"));
    }

    #[test]
    fn test_missing_schema() {
        let workspace = workspace_from_sources(&[(
            "contacts.firm",
            r#"
            contact john_doe {
                name = "John Doe"
            }
            "#,
        )]);

        let diagnostics = collect_workspace_diagnostics(&workspace);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("No schema defined"));
        assert!(diagnostics[0].message.contains("contact"));
    }

    #[test]
    fn test_valid_reference_no_diagnostic() {
        let workspace = workspace_from_sources(&[
            (
                "schemas.firm",
                r#"
                schema contact {
                    field {
                        name = "name"
                        type = "string"
                        required = true
                    }
                    field {
                        name = "manager"
                        type = "reference"
                    }
                }
                "#,
            ),
            (
                "contacts.firm",
                r#"
                contact jane_doe {
                    name = "Jane Doe"
                }

                contact john_doe {
                    name = "John Doe"
                    manager = contact.jane_doe
                }
                "#,
            ),
        ]);

        let diagnostics = collect_workspace_diagnostics(&workspace);
        assert!(diagnostics.is_empty(), "Expected no diagnostics, got: {:?}", diagnostics);
    }

    #[test]
    fn test_value_error_skips_schema_validation() {
        // Entity with a value parse error should not also report schema validation errors.
        // Fix value errors first, then schema errors appear.
        let workspace = workspace_from_sources(&[
            (
                "schemas.firm",
                r#"
                schema invoice {
                    field {
                        name = "amount"
                        type = "currency"
                        required = true
                    }
                    field {
                        name = "description"
                        type = "string"
                        required = true
                    }
                }
                "#,
            ),
            (
                "invoices.firm",
                r#"
                invoice inv_001 {
                    amount = 100.50 INVALID
                }
                "#,
            ),
        ]);

        let diagnostics = collect_workspace_diagnostics(&workspace);
        // Should only have the value parse error, not the missing "description" field error
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("Currency code"));
    }

    #[test]
    fn test_broken_field_reference() {
        let workspace = workspace_from_sources(&[
            (
                "schemas.firm",
                r#"
                schema contact {
                    field {
                        name = "name"
                        type = "string"
                        required = true
                    }
                    field {
                        name = "nickname"
                        type = "reference"
                    }
                }
                "#,
            ),
            (
                "contacts.firm",
                r#"
                contact jane_doe {
                    name = "Jane Doe"
                }

                contact john_doe {
                    name = "John Doe"
                    nickname = contact.jane_doe.nonexistent_field
                }
                "#,
            ),
        ]);

        let diagnostics = collect_workspace_diagnostics(&workspace);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("has no field"));
        assert!(diagnostics[0].message.contains("nonexistent_field"));
    }

    #[test]
    fn test_valid_field_reference_no_diagnostic() {
        let workspace = workspace_from_sources(&[
            (
                "schemas.firm",
                r#"
                schema contact {
                    field {
                        name = "name"
                        type = "string"
                        required = true
                    }
                    field {
                        name = "nickname"
                        type = "reference"
                    }
                }
                "#,
            ),
            (
                "contacts.firm",
                r#"
                contact jane_doe {
                    name = "Jane Doe"
                }

                contact john_doe {
                    name = "John Doe"
                    nickname = contact.jane_doe.name
                }
                "#,
            ),
        ]);

        let diagnostics = collect_workspace_diagnostics(&workspace);
        assert!(diagnostics.is_empty(), "Expected no diagnostics, got: {:?}", diagnostics);
    }
}
