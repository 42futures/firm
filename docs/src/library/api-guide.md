# Working with the API

This guide covers common patterns for working with Firm's Rust API.

## Loading workspaces

### From a directory

```rust,no_run
use firm_lang::workspace::Workspace;

let mut workspace = Workspace::new();
workspace.load_directory("./my_workspace")?;
let build = workspace.build()?;
```

### From a single file

```rust,no_run
let mut workspace = Workspace::new();
workspace.load_file("./entities.firm")?;
let build = workspace.build()?;
```

### From a string

```rust,no_run
let dsl = r#"
person john {
    name = "John Doe"
}
"#;

let mut workspace = Workspace::new();
workspace.load_string(dsl)?;
let build = workspace.build()?;
```

## Building the entity graph

```rust,no_run
use firm_core::EntityGraph;

let mut graph = EntityGraph::new();
graph.add_entities(build.entities)?;
graph.build(); // Resolves all references
```

## Querying entities

### Get by ID

```rust,no_run
use firm_core::EntityId;

let entity = graph.get_entity(&EntityId::new("person.john_doe"))?;
```

### Get all entities of a type

```rust,no_run
use firm_core::EntityType;

let people = graph.get_entities_by_type(&EntityType::new("person"));
```

### Filter entities

```rust,no_run
let active_tasks: Vec<&Entity> = graph
    .get_entities_by_type(&EntityType::new("task"))
    .iter()
    .filter(|e| {
        e.get_field(FieldId::new("completed"))
            .map(|v| v == &FieldValue::Boolean(false))
            .unwrap_or(false)
    })
    .collect();
```

## Working with fields

### Get field value

```rust,no_run
use firm_core::FieldId;

let name = entity.get_field(FieldId::new("name"))?;
```

### Set field value

```rust,no_run
use firm_core::FieldValue;

let mut entity = entity.clone();
entity.set_field(
    FieldId::new("email"),
    FieldValue::String("newemail@example.com".to_string())
);
```

### Check if field exists

```rust,no_run
if entity.has_field(FieldId::new("phone")) {
    println!("Has phone number");
}
```

## Traversing relationships

### Get related entities

```rust,no_run
let related = graph.get_related_entities(&EntityId::new("organization.acme"))?;
```

### Follow a reference

```rust,no_run
let assignee_ref = task.get_field(FieldId::new("assignee_ref"))?;
if let FieldValue::Reference(ref_id) = assignee_ref {
    let assignee = graph.get_entity(ref_id)?;
    println!("Assigned to: {:?}", assignee);
}
```

### Resolve reference helper

```rust,no_run
let contact_ref = lead.get_field(FieldId::new("contact_ref"))?;
let contact = contact_ref.resolve_entity_reference(&graph)?;
```

## Validating with schemas

```rust,no_run
use firm_core::{EntitySchema, FieldType};

let schema = EntitySchema::new(EntityType::new("task"))
    .with_required_field(FieldId::new("name"), FieldType::String)
    .with_optional_field(FieldId::new("due_date"), FieldType::DateTime);

schema.validate(&task_entity)?;
```

## Generating DSL

### Generate for a single entity

```rust,no_run
use firm_lang::generator::generate_dsl;

let dsl = generate_dsl(&entity)?;
println!("{}", dsl);
```

### Generate for multiple entities

```rust,no_run
for entity in &entities {
    let dsl = generate_dsl(entity)?;
    println!("{}\n", dsl);
}
```

## Error handling

Firm uses standard Rust `Result` types:

```rust,no_run
match workspace.load_directory("./workspace") {
    Ok(()) => println!("Loaded successfully"),
    Err(e) => eprintln!("Error loading workspace: {}", e),
}
```


