# Architecture

Firm is organized as a Rust workspace with three crates, each with a specific responsibility.

## Crate overview

```
firm/
├── firm_core/     - Core data structures and graph operations
├── firm_lang/     - DSL parsing and generation
└── firm_cli/      - Command-line interface
```

## firm_core

Core data structures and graph operations.

**Responsibilities:**
- Entity data model
- Typed fields with references
- Relationship graph with query capabilities
- Entity schemas and validation

**Key types:**
- `Entity` - A business object with ID, type, and fields
- `EntityGraph` - Graph of entities with relationship edges
- `FieldValue` - Typed field values (String, Integer, Reference, etc.)
- `EntitySchema` - Schema definitions for validation

**Usage:**
```rust,no_run
use firm_core::{Entity, EntityGraph, EntityId, FieldId, FieldValue};

let entity = Entity::new(
        EntityId::new("john"),
        EntityType::new("person"))
    .with_field(
        FieldId::new("name"),
        FieldValue::String("John".to_string()));

let mut graph = EntityGraph::new();
graph.add_entity(entity)?;
graph.build();
```

## firm_lang

DSL parsing and generation.

**Responsibilities:**
- Tree-sitter-based DSL parser for `.firm` files
- Pest-based query language parser for the CLI
- Converting and generating DSL
- Workspace support for multi-file projects

**Key types:**
- `Workspace` - Multi-file workspace manager
- `parser::dsl` - DSL parsing
- `parser::query` - Query parsing
- `generate` - DSL generation

**Usage:**
```rust,no_run
use firm_lang::workspace::Workspace;

let mut workspace = Workspace::new();
workspace.load_directory("./my_workspace")?;
let build = workspace.build()?;
```

## firm_cli

Command-line interface, making the Firm workspace interactive.

**Responsibilities:**
- Interactive and non-interactive commands
- User-friendly output formatting
- Integration of core and lang crates

**Commands:**
- `firm init` - Initialize a workspace
- `firm add` - Add entities
- `firm list` - List entities by type
- `firm get` - Get a specific entity
- `firm related` - Find related entities
- `firm query` - Run custom queries
- `firm source` - Get the source file path for an entity

**Usage:**
```bash
firm init
firm add --type person --id john
firm query 'from person | where name == "John"'
```
