# Getting started with Rust

Beyond the CLI, you can integrate Firm's core logic directly into your own software using the `firm_core` and `firm_lang` Rust packages. This allows you to build more powerful automations and integrations on top of Firm.

## Adding dependencies

First, add the Firm crates to your `Cargo.toml`:

```toml
[dependencies]
firm_core = { git = "https://github.com/42futures/firm.git" }
firm_lang = { git = "https://github.com/42futures/firm.git" }
```

## Basic usage

Here's a simple example of loading a workspace and querying the entity graph:

```rust,no_run
use firm_lang::workspace::Workspace;
use firm_core::EntityGraph;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load workspace from a directory
    let mut workspace = Workspace::new();
    workspace.load_directory("./my_workspace")?;
    let build = workspace.build()?;

    // Build the graph from the workspace entities
    let mut graph = EntityGraph::new();
    graph.add_entities(build.entities)?;
    graph.build();

    // Query the graph for a specific entity
    let lead = graph.get_entity(&EntityId::new("lead.ai_validation_project"))?;
    println!("Found lead: {:?}", lead);

    Ok(())
}
```

## Working with entities

You can traverse relationships and access field values:

```rust,no_run
use firm_core::{EntityId, FieldId};

// Get an entity
let lead = graph.get_entity(&EntityId::new("lead.ai_validation_project"))?;

// Get a field value
let contact_ref = lead.get_field(FieldId::new("contact_ref"))?;

// Resolve the reference to another entity
let contact = contact_ref.resolve_entity_reference(&graph)?;

println!("Contact: {:?}", contact);
```

## Creating entities programmatically

You can create entities in code:

```rust,no_run
use firm_core::{Entity, EntityId, EntityType, FieldId, FieldValue};

let person = Entity::new(
    EntityId::new("john_doe"),
    EntityType::new("person")
)
.with_field(FieldId::new("name"), FieldValue::String("John Doe".to_string()))
.with_field(FieldId::new("email"), FieldValue::String("john@example.com".to_string()));

// Add to the graph
graph.add_entity(person)?;
```

## Generating DSL

You can also generate DSL from entities:

```rust,no_run
use firm_lang::generator::generate_dsl;

let dsl = generate_dsl(&entity)?;
println!("{}", dsl);
```

This outputs:

```firm
person john_doe {
    name = "John Doe"
    email = "john@example.com"
}
```


