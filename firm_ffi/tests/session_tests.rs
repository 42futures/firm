use firm_ffi::{
    FirmDirection, FirmEntity, FirmError, FirmField, FirmFieldValue, FirmQueryResult, FirmSession,
};

const TASK_SCHEMA: &str = r#"
schema task {
    field {
        name = "title"
        type = "string"
        required = true
    }

    field {
        name = "is_completed"
        type = "boolean"
        required = false
    }

    field {
        name = "priority"
        type = "integer"
        required = false
    }

    field {
        name = "assignee"
        type = "reference"
        required = false
    }
}
"#;

const PERSON_SCHEMA: &str = r#"
schema person {
    field {
        name = "name"
        type = "string"
        required = true
    }

    field {
        name = "email"
        type = "string"
        required = false
    }
}
"#;

const TASK_ENTITIES: &str = r#"
task fix_bug {
    title = "Fix the login bug"
    is_completed = false
    priority = 1
}

task write_docs {
    title = "Write documentation"
    is_completed = true
    priority = 2
}
"#;

const PERSON_ENTITIES: &str = r#"
person alice {
    name = "Alice"
    email = "alice@example.com"
}
"#;

const TASK_WITH_REF: &str = r#"
task review_code {
    title = "Review pull request"
    is_completed = false
    priority = 1
    assignee = person.alice
}
"#;

fn create_session_with_tasks() -> FirmSession {
    let session = FirmSession::new();
    session
        .load_source(TASK_SCHEMA.to_string(), "schema.firm".to_string())
        .unwrap();
    session
        .load_source(TASK_ENTITIES.to_string(), "tasks.firm".to_string())
        .unwrap();
    session.build().unwrap();
    session
}

fn create_session_with_people_and_tasks() -> FirmSession {
    let session = FirmSession::new();
    session
        .load_source(TASK_SCHEMA.to_string(), "schema/task.firm".to_string())
        .unwrap();
    session
        .load_source(PERSON_SCHEMA.to_string(), "schema/person.firm".to_string())
        .unwrap();
    session
        .load_source(PERSON_ENTITIES.to_string(), "people.firm".to_string())
        .unwrap();
    session
        .load_source(TASK_WITH_REF.to_string(), "tasks.firm".to_string())
        .unwrap();
    session.build().unwrap();
    session
}

/// Helper to extract entities from a query result.
fn unwrap_entities(result: FirmQueryResult) -> Vec<FirmEntity> {
    match result {
        FirmQueryResult::Entities { entities } => entities,
        FirmQueryResult::Aggregation { value } => {
            panic!("Expected entities, got aggregation: {}", value)
        }
    }
}

/// Helper to find a field value by name on an entity.
fn get_field<'a>(entity: &'a FirmEntity, name: &str) -> Option<&'a FirmFieldValue> {
    entity
        .fields
        .iter()
        .find(|f| f.name == name)
        .map(|f| &f.value)
}

#[test]
fn test_create_and_build_empty() {
    let session = FirmSession::new();
    session.build().unwrap();

    let schemas = session.list_schemas().unwrap();
    assert!(schemas.is_empty());
}

#[test]
fn test_load_and_build() {
    let session = create_session_with_tasks();

    let entities = session.list_entities("task".to_string()).unwrap();
    assert_eq!(entities.len(), 2);
    assert!(entities.iter().any(|id| id.contains("fix_bug")));
    assert!(entities.iter().any(|id| id.contains("write_docs")));
}

#[test]
fn test_query() {
    let session = create_session_with_tasks();

    let result = session
        .query("from task | where is_completed == false".to_string())
        .unwrap();
    let entities = unwrap_entities(result);
    assert_eq!(entities.len(), 1);
    assert!(entities[0].id.contains("fix_bug"));
}

#[test]
fn test_query_returns_typed_fields() {
    let session = create_session_with_tasks();

    let result = session
        .query("from task | where priority == 1".to_string())
        .unwrap();
    let entities = unwrap_entities(result);
    assert_eq!(entities.len(), 1);

    let entity = &entities[0];
    assert_eq!(entity.entity_type, "task");

    // Verify typed field values
    match get_field(entity, "title") {
        Some(FirmFieldValue::String { value }) => {
            assert_eq!(value, "Fix the login bug");
        }
        other => panic!("Expected String field, got {:?}", other),
    }

    match get_field(entity, "is_completed") {
        Some(FirmFieldValue::Boolean { value }) => {
            assert!(!value);
        }
        other => panic!("Expected Boolean field, got {:?}", other),
    }

    match get_field(entity, "priority") {
        Some(FirmFieldValue::Integer { value }) => {
            assert_eq!(*value, 1);
        }
        other => panic!("Expected Integer field, got {:?}", other),
    }
}

#[test]
fn test_query_aggregation() {
    let session = create_session_with_tasks();

    let result = session
        .query("from task | count".to_string())
        .unwrap();
    match result {
        FirmQueryResult::Aggregation { value } => {
            assert_eq!(value, "2");
        }
        FirmQueryResult::Entities { .. } => panic!("Expected aggregation"),
    }
}

#[test]
fn test_get_entity() {
    let session = create_session_with_tasks();

    let entity = session
        .get_entity("task".to_string(), "fix_bug".to_string())
        .unwrap();
    assert!(entity.is_some());

    let entity = entity.unwrap();
    assert!(entity.id.contains("fix_bug"));
    assert_eq!(entity.entity_type, "task");

    match get_field(&entity, "title") {
        Some(FirmFieldValue::String { value }) => {
            assert_eq!(value, "Fix the login bug");
        }
        other => panic!("Expected String field, got {:?}", other),
    }
}

#[test]
fn test_get_entity_not_found() {
    let session = create_session_with_tasks();

    let result = session
        .get_entity("task".to_string(), "nonexistent".to_string())
        .unwrap();
    assert!(result.is_none());
}

#[test]
fn test_list_entities() {
    let session = create_session_with_tasks();

    let entities = session.list_entities("task".to_string()).unwrap();
    assert_eq!(entities.len(), 2);
}

#[test]
fn test_list_schemas() {
    let session = create_session_with_tasks();

    let schemas = session.list_schemas().unwrap();
    assert_eq!(schemas.len(), 1);
    assert!(schemas.contains(&"task".to_string()));
}

#[test]
fn test_generate_entity_dsl() {
    let session = create_session_with_tasks();

    let entity = FirmEntity {
        id: "task.new_task".to_string(),
        entity_type: "task".to_string(),
        fields: vec![
            FirmField {
                name: "title".to_string(),
                value: FirmFieldValue::String {
                    value: "New task".to_string(),
                },
            },
            FirmField {
                name: "is_completed".to_string(),
                value: FirmFieldValue::Boolean { value: false },
            },
            FirmField {
                name: "priority".to_string(),
                value: FirmFieldValue::Integer { value: 3 },
            },
        ],
    };

    let dsl = session.generate_entity_dsl(entity).unwrap();

    assert!(dsl.contains("task new_task"));
    assert!(dsl.contains("\"New task\""));
    assert!(dsl.contains("false"));
    assert!(dsl.contains("3"));
}

#[test]
fn test_find_entity_source() {
    let session = create_session_with_tasks();

    let path = session
        .find_entity_source("task".to_string(), "fix_bug".to_string())
        .unwrap();
    assert!(path.is_some());
    assert_eq!(path.unwrap(), "tasks.firm");
}

#[test]
fn test_find_entity_source_not_found() {
    let session = create_session_with_tasks();

    let path = session
        .find_entity_source("task".to_string(), "nonexistent".to_string())
        .unwrap();
    assert!(path.is_none());
}

#[test]
fn test_get_source() {
    let session = FirmSession::new();
    session
        .load_source(TASK_SCHEMA.to_string(), "schema.firm".to_string())
        .unwrap();

    let source = session.get_source("schema.firm".to_string()).unwrap();
    assert!(source.is_some());
    assert!(source.unwrap().contains("schema task"));
}

#[test]
fn test_get_source_not_found() {
    let session = FirmSession::new();

    let source = session.get_source("nonexistent.firm".to_string()).unwrap();
    assert!(source.is_none());
}

#[test]
fn test_get_related() {
    let session = create_session_with_people_and_tasks();

    let related = session
        .get_related("person".to_string(), "alice".to_string(), FirmDirection::Both)
        .unwrap();
    assert_eq!(related.len(), 1);
    assert!(related[0].id.contains("review_code"));
}

#[test]
fn test_get_related_entity_reference_field() {
    let session = create_session_with_people_and_tasks();

    let entity = session
        .get_entity("task".to_string(), "review_code".to_string())
        .unwrap()
        .unwrap();

    match get_field(&entity, "assignee") {
        Some(FirmFieldValue::EntityReference { entity_id }) => {
            assert!(entity_id.contains("alice"));
        }
        other => panic!("Expected EntityReference, got {:?}", other),
    }
}

#[test]
fn test_load_invalid_source() {
    let session = FirmSession::new();

    session
        .load_source(TASK_SCHEMA.to_string(), "schema.firm".to_string())
        .unwrap();
    session
        .load_source(
            "task broken {\n}\n".to_string(),
            "broken.firm".to_string(),
        )
        .unwrap();

    let result = session.build();
    assert!(result.is_err());
    match result.unwrap_err() {
        FirmError::BuildError { .. } => {}
        other => panic!("Expected BuildError, got {:?}", other),
    }
}

#[test]
fn test_query_before_build() {
    let session = FirmSession::new();
    session
        .load_source(TASK_SCHEMA.to_string(), "schema.firm".to_string())
        .unwrap();

    let result = session.query("from task".to_string());
    assert!(result.is_err());
    match result.unwrap_err() {
        FirmError::NotBuilt => {}
        other => panic!("Expected NotBuilt, got {:?}", other),
    }
}

#[test]
fn test_build_invalidates_on_new_source() {
    let session = FirmSession::new();
    session
        .load_source(TASK_SCHEMA.to_string(), "schema.firm".to_string())
        .unwrap();
    session
        .load_source(TASK_ENTITIES.to_string(), "tasks.firm".to_string())
        .unwrap();
    session.build().unwrap();

    // Loading new source should invalidate the build
    session
        .load_source(
            "task extra { title = \"Extra\" }\n".to_string(),
            "extra.firm".to_string(),
        )
        .unwrap();

    // Query should fail since build is invalidated
    let result = session.query("from task".to_string());
    assert!(result.is_err());
    match result.unwrap_err() {
        FirmError::NotBuilt => {}
        other => panic!("Expected NotBuilt, got {:?}", other),
    }

    // Rebuild should work
    session.build().unwrap();
    let entities = session.list_entities("task".to_string()).unwrap();
    assert_eq!(entities.len(), 3);
}
