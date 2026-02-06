//! Select aggregation: extract specific field values from entities

use super::super::filter::{FieldRef, MetadataField};
use super::super::types::AggregationResult;
use super::super::QueryError;
use crate::{Entity, FieldValue};

pub fn execute(
    fields: &[FieldRef],
    entities: &[&Entity],
) -> Result<AggregationResult, QueryError> {
    let columns: Vec<String> = fields
        .iter()
        .map(|f| match f {
            FieldRef::Metadata(MetadataField::Id) => "@id".to_string(),
            FieldRef::Metadata(MetadataField::Type) => "@type".to_string(),
            FieldRef::Regular(field_id) => field_id.as_str().to_string(),
        })
        .collect();

    let rows: Vec<Vec<Option<FieldValue>>> = entities
        .iter()
        .map(|entity| {
            fields
                .iter()
                .map(|field| match field {
                    FieldRef::Metadata(MetadataField::Id) => {
                        Some(FieldValue::String(entity.id.to_string()))
                    }
                    FieldRef::Metadata(MetadataField::Type) => {
                        Some(FieldValue::String(entity.entity_type.to_string()))
                    }
                    FieldRef::Regular(field_id) => entity.get_field(field_id).cloned(),
                })
                .collect()
        })
        .collect();

    Ok(AggregationResult::Select { columns, rows })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Entity, EntityId, EntityType, FieldId, FieldValue};

    fn make_entities() -> Vec<Entity> {
        vec![
            Entity::new(EntityId::new("p1"), EntityType::new("person"))
                .with_field(FieldId::new("name"), "Alice")
                .with_field(FieldId::new("age"), FieldValue::Integer(30)),
            Entity::new(EntityId::new("p2"), EntityType::new("person"))
                .with_field(FieldId::new("name"), "Bob"),
        ]
    }

    #[test]
    fn test_select_single_field() {
        let entities = make_entities();
        let refs: Vec<&Entity> = entities.iter().collect();
        let fields = vec![FieldRef::Regular(FieldId::new("name"))];
        let result = execute(&fields, &refs).unwrap();
        if let AggregationResult::Select { columns, rows } = result {
            assert_eq!(columns, vec!["name"]);
            assert_eq!(rows.len(), 2);
            assert_eq!(rows[0][0], Some(FieldValue::String("Alice".to_string())));
            assert_eq!(rows[1][0], Some(FieldValue::String("Bob".to_string())));
        } else {
            panic!("Expected Select result");
        }
    }

    #[test]
    fn test_select_multiple_fields() {
        let entities = make_entities();
        let refs: Vec<&Entity> = entities.iter().collect();
        let fields = vec![
            FieldRef::Regular(FieldId::new("name")),
            FieldRef::Regular(FieldId::new("age")),
        ];
        let result = execute(&fields, &refs).unwrap();
        if let AggregationResult::Select { columns, rows } = result {
            assert_eq!(columns, vec!["name", "age"]);
            assert_eq!(rows.len(), 2);
            // p1 has both fields
            assert_eq!(rows[0][0], Some(FieldValue::String("Alice".to_string())));
            assert_eq!(rows[0][1], Some(FieldValue::Integer(30)));
            // p2 has name but no age
            assert_eq!(rows[1][0], Some(FieldValue::String("Bob".to_string())));
            assert_eq!(rows[1][1], None);
        } else {
            panic!("Expected Select result");
        }
    }

    #[test]
    fn test_select_metadata_id() {
        let entities = make_entities();
        let refs: Vec<&Entity> = entities.iter().collect();
        let fields = vec![FieldRef::Metadata(MetadataField::Id)];
        let result = execute(&fields, &refs).unwrap();
        if let AggregationResult::Select { columns, rows } = result {
            assert_eq!(columns, vec!["@id"]);
            // EntityId converts to snake_case, so "p1" becomes "p_1"
            assert_eq!(rows[0][0], Some(FieldValue::String("p_1".to_string())));
            assert_eq!(rows[1][0], Some(FieldValue::String("p_2".to_string())));
        } else {
            panic!("Expected Select result");
        }
    }

    #[test]
    fn test_select_metadata_type() {
        let entities = make_entities();
        let refs: Vec<&Entity> = entities.iter().collect();
        let fields = vec![FieldRef::Metadata(MetadataField::Type)];
        let result = execute(&fields, &refs).unwrap();
        if let AggregationResult::Select { columns, rows } = result {
            assert_eq!(columns, vec!["@type"]);
            assert!(rows.iter().all(|r| r[0] == Some(FieldValue::String("person".to_string()))));
        } else {
            panic!("Expected Select result");
        }
    }

    #[test]
    fn test_select_missing_field_returns_none() {
        let entities = make_entities();
        let refs: Vec<&Entity> = entities.iter().collect();
        let fields = vec![FieldRef::Regular(FieldId::new("nonexistent"))];
        let result = execute(&fields, &refs).unwrap();
        if let AggregationResult::Select { rows, .. } = result {
            assert!(rows.iter().all(|r| r[0].is_none()));
        } else {
            panic!("Expected Select result");
        }
    }

    #[test]
    fn test_select_empty_entities() {
        let refs: Vec<&Entity> = vec![];
        let fields = vec![FieldRef::Regular(FieldId::new("name"))];
        let result = execute(&fields, &refs).unwrap();
        if let AggregationResult::Select { columns, rows } = result {
            assert_eq!(columns, vec!["name"]);
            assert!(rows.is_empty());
        } else {
            panic!("Expected Select result");
        }
    }
}
