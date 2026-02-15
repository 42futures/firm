//! Completion item generation for the Firm LSP.

use std::collections::HashMap;

use tower_lsp_server::lsp_types::*;

use firm_core::{Entity, EntityType};
use firm_core::schema::EntitySchema;

/// Generate field name completions for an entity type.
///
/// Returns completion items for schema fields not already present in the entity,
/// with required fields sorted before optional fields.
pub fn complete_field_names(
    entity_type: &str,
    existing_fields: &[&str],
    schemas: &HashMap<EntityType, EntitySchema>,
) -> Vec<CompletionItem> {
    let schema = match schemas.get(&EntityType::new(entity_type)) {
        Some(s) => s,
        None => return Vec::new(),
    };

    let mut items = Vec::new();

    for (field_id, field_schema) in schema.ordered_fields() {
        let name = field_id.as_str();

        // Skip fields already present
        if existing_fields.contains(&name) {
            continue;
        }

        let (sort_prefix, mode_label) = if field_schema.is_required() {
            ("0", "required")
        } else {
            ("1", "optional")
        };

        let detail = format!("{mode_label}, {}", field_schema.expected_type());

        items.push(CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::FIELD),
            detail: Some(detail),
            sort_text: Some(format!("{sort_prefix}_{:04}_{name}", field_schema.order)),
            insert_text: Some(format!("{name} = ")),
            ..Default::default()
        });
    }

    items
}

/// Generate reference completions from a typed prefix.
///
/// - No dot yet → suggest entity type names (from schema keys)
/// - `type.` → suggest all entity IDs of that type (inserts just the ID)
/// - `type.partial` → suggest matching entity IDs (inserts just the ID)
/// - `type.id.` → suggest field names from the matched entity
/// - `type.id.partial` → suggest matching field names
pub fn complete_references(
    prefix: &str,
    entities: &[Entity],
) -> Vec<CompletionItem> {
    // Count dots to determine context depth
    let dot_count = prefix.matches('.').count();

    if dot_count == 0 {
        // No dot yet — suggest entity types that match the prefix
        let mut seen = HashMap::new();
        for entity in entities {
            let et = entity.entity_type.as_str();
            if et.starts_with(prefix) && !seen.contains_key(et) {
                seen.insert(et, ());
            }
        }
        let mut items: Vec<CompletionItem> = seen
            .keys()
            .map(|et| CompletionItem {
                label: format!("{et}."),
                kind: Some(CompletionItemKind::MODULE),
                detail: Some("entity type".to_string()),
                insert_text: Some(format!("{et}.")),
                ..Default::default()
            })
            .collect();
        items.sort_by(|a, b| a.label.cmp(&b.label));
        items
    } else if dot_count == 1 {
        // One dot: `type.` or `type.partial` — suggest entity IDs
        let (entity_type, id_prefix) = prefix.split_once('.').unwrap();
        let mut items: Vec<CompletionItem> = entities
            .iter()
            .filter(|e| e.entity_type.as_str() == entity_type)
            .filter(|e| {
                let (_, eid) = firm_core::decompose_entity_id(e.id.as_str());
                eid.starts_with(id_prefix)
            })
            .map(|e| {
                let (_, eid) = firm_core::decompose_entity_id(e.id.as_str());
                CompletionItem {
                    label: eid.to_string(),
                    kind: Some(CompletionItemKind::REFERENCE),
                    detail: Some(entity_type.to_string()),
                    filter_text: Some(eid.to_string()),
                    ..Default::default()
                }
            })
            .collect();
        items.sort_by(|a, b| a.label.cmp(&b.label));
        items
    } else if dot_count == 2 {
        // Two dots: `type.id.` or `type.id.partial` — suggest field names
        let mut parts = prefix.splitn(3, '.');
        let entity_type = parts.next().unwrap_or("");
        let entity_id = parts.next().unwrap_or("");
        let field_prefix = parts.next().unwrap_or("");

        let composite_id = firm_core::compose_entity_id(entity_type, entity_id);

        // Find the entity and suggest its fields
        if let Some(entity) = entities.iter().find(|e| e.id == composite_id) {
            let mut items: Vec<CompletionItem> = entity
                .fields
                .iter()
                .filter(|(fid, _)| fid.as_str().starts_with(field_prefix))
                .map(|(fid, fval)| CompletionItem {
                    label: fid.as_str().to_string(),
                    kind: Some(CompletionItemKind::FIELD),
                    detail: Some(format!("{}", fval.get_type())),
                    filter_text: Some(fid.as_str().to_string()),
                    ..Default::default()
                })
                .collect();
            items.sort_by(|a, b| a.label.cmp(&b.label));
            items
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use firm_core::{Entity, EntityId, EntityType, FieldId, FieldType};
    use firm_core::schema::EntitySchema;

    fn test_schemas() -> HashMap<EntityType, EntitySchema> {
        let mut schemas = HashMap::new();
        let schema = EntitySchema::new(EntityType::new("contact"))
            .with_required_field(FieldId::new("name"), FieldType::String)
            .with_required_field(FieldId::new("email"), FieldType::String)
            .with_optional_field(FieldId::new("phone"), FieldType::String)
            .with_optional_field(FieldId::new("notes"), FieldType::String);
        schemas.insert(EntityType::new("contact"), schema);
        schemas
    }

    fn test_entities() -> Vec<Entity> {
        vec![
            Entity::new(EntityId::new("contact.jane_doe"), EntityType::new("contact"))
                .with_field(FieldId::new("name"), "Jane Doe"),
            Entity::new(EntityId::new("contact.john_doe"), EntityType::new("contact"))
                .with_field(FieldId::new("name"), "John Doe"),
            Entity::new(EntityId::new("project.website"), EntityType::new("project"))
                .with_field(FieldId::new("title"), "Website"),
        ]
    }

    #[test]
    fn test_field_completions_required_before_optional() {
        let schemas = test_schemas();
        let items = complete_field_names("contact", &[], &schemas);

        assert_eq!(items.len(), 4);
        // Required fields should sort before optional
        assert!(items[0].sort_text.as_ref().unwrap().starts_with("0_"));
        assert!(items[1].sort_text.as_ref().unwrap().starts_with("0_"));
        assert!(items[2].sort_text.as_ref().unwrap().starts_with("1_"));
        assert!(items[3].sort_text.as_ref().unwrap().starts_with("1_"));
    }

    #[test]
    fn test_field_completions_exclude_existing() {
        let schemas = test_schemas();
        let items = complete_field_names("contact", &["name", "email"], &schemas);

        assert_eq!(items.len(), 2);
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert!(labels.contains(&"phone"));
        assert!(labels.contains(&"notes"));
        assert!(!labels.contains(&"name"));
    }

    #[test]
    fn test_field_completions_insert_text_has_equals() {
        let schemas = test_schemas();
        let items = complete_field_names("contact", &[], &schemas);

        for item in &items {
            assert!(item.insert_text.as_ref().unwrap().ends_with(" = "));
        }
    }

    #[test]
    fn test_field_completions_detail_shows_mode_and_type() {
        let schemas = test_schemas();
        let items = complete_field_names("contact", &[], &schemas);

        let name_item = items.iter().find(|i| i.label == "name").unwrap();
        assert_eq!(name_item.detail.as_ref().unwrap(), "required, String");

        let phone_item = items.iter().find(|i| i.label == "phone").unwrap();
        assert_eq!(phone_item.detail.as_ref().unwrap(), "optional, String");
    }

    #[test]
    fn test_field_completions_unknown_schema_returns_empty() {
        let schemas = test_schemas();
        let items = complete_field_names("nonexistent", &[], &schemas);
        assert!(items.is_empty());
    }

    #[test]
    fn test_reference_completions_type_dot_returns_ids_only() {
        let entities = test_entities();
        let items = complete_references("contact.", &entities);

        assert_eq!(items.len(), 2);
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        // Labels should be just the ID, not type.id
        assert!(labels.contains(&"jane_doe"));
        assert!(labels.contains(&"john_doe"));
    }

    #[test]
    fn test_reference_completions_type_dot_partial_filters() {
        let entities = test_entities();
        let items = complete_references("contact.ja", &entities);

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "jane_doe");
    }

    #[test]
    fn test_reference_completions_empty_prefix_suggests_types() {
        let entities = test_entities();
        let items = complete_references("", &entities);

        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert!(labels.contains(&"contact."));
        assert!(labels.contains(&"project."));
    }

    #[test]
    fn test_reference_completions_partial_type_filters() {
        let entities = test_entities();
        let items = complete_references("con", &entities);

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "contact.");
    }

    #[test]
    fn test_reference_completions_unknown_type_returns_empty() {
        let entities = test_entities();
        let items = complete_references("unknown.", &entities);

        assert!(items.is_empty());
    }

    #[test]
    fn test_reference_completions_field_suggestions() {
        let entities = test_entities();
        let items = complete_references("contact.jane_doe.", &entities);

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "name");
        assert_eq!(items[0].detail.as_ref().unwrap(), "String");
    }

    #[test]
    fn test_reference_completions_field_partial_filters() {
        let entities = vec![
            Entity::new(EntityId::new("contact.jane_doe"), EntityType::new("contact"))
                .with_field(FieldId::new("name"), "Jane Doe")
                .with_field(FieldId::new("notes"), "Some notes"),
        ];
        let items = complete_references("contact.jane_doe.na", &entities);

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "name");
    }

    #[test]
    fn test_reference_completions_field_unknown_entity_returns_empty() {
        let entities = test_entities();
        let items = complete_references("contact.nobody.", &entities);

        assert!(items.is_empty());
    }
}
