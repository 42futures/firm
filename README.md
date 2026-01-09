# Firm: Business-as-code
A text-based work management system for technologists.

![Firm CLI demo](media/demo.gif)

## Why?
Modern businesses are natively digital, but lack a unified view. Your data is scattered across SaaS tools you don't control, so you piece together answers by jumping between platforms.

Your business is a graph: customers link to projects, projects link to tasks, people link to organizations. Firm lets you define these relationships in plain text files (you own!).

Version controlled, locally stored and structured as code with the Firm DSL. This structured representation of your work, *business-as-code*, makes your business readable to yourself and to the robots that help you run it.

### Features
- **Everything in one place:** Organizations, contacts, projects, and how they relate.
- **Own your data:** Plain text files and tooling that runs on your machine.
- **Open data model:** Tailor to your business with custom schemas.
- **Automate anything:** Search, report, integrate, whatever. It's just code.
- **AI-ready:** LLMs can read, write, and query your business structure.

## Getting started

### Initialize your workspace

The easiest way to start is by running `firm init` in an empty directory:

```bash
mkdir my_workspace
cd my_workspace
firm init
```

This interactive command will help you set up your workspace by:
- Creating default schemas for common entity types (Person, Organization, Task, etc.)
- Adding a `.gitignore` file
- Creating starter entities (you and your organization)
- Adding AI context documentation for AI coding assistants

Once initialized, your workspace is ready to use!

### Working with your workspace

Firm operates on a "workspace": a directory containing all your `.firm` DSL files. The Firm CLI processes every file in this workspace to build a unified, queryable graph of your business.

You can add entities to your workspace either by using the CLI or by writing the DSL yourself.

### Add entities with the CLI
Use `firm add` to generate new entities. Out of the box, Firm supports a set of pre-built entity schemas for org mapping, customer relations and work management. The CLI will prompt you for the necessary info and generate corresponding DSL.

```bash
$ firm add
```
```
Adding new entity

> Type: organization
> ID: megacorp
> Name: Megacorp Ltd.
> Email: mega@corp.com
> Urls: ["corp.com"]

Writing generated DSL to file my_workspace/generated/organization.firm
```

You can also use `firm add` noninteractively by providing its type, ID and fields:

`$ firm add --type organization --id megacorp --field name "Megacorp Ltd."`

### Write DSL manually
Alternatively, you can create a `.firm` file and write the DSL yourself.

```firm
organization megacorp {
  name = "Megacorp Ltd."
  email = "mega@corp.com"
  urls = ["corp.com"]
}
```

Both of these methods achieve the same result: a new entity defined in your Firm workspace.

### Querying the workspace
Once you have entities in your workspace, you can query them using the CLI.

#### Listing entities
Use `firm list` to see all entities of a specific type.

```bash
$ firm list task
```
```
Found 7 entities with type 'task'

ID: task.design_homepage
Name: Design new homepage
Is completed: false
Assignee ref: person.jane_doe

...
```

#### Getting an entity
To view the full details of a single entity, use `firm get` followed by the entity's type and ID.

```bash
$ firm get person john_doe
```
```
Found 'person' entity with ID 'john_doe'

ID: person.john_doe
Name: John Doe
Email: john@doe.com
```

#### Exploring relationships
The power of Firm lies in its ability to travel a graph of your business. Use `firm related` to explore connections to/from any entity.

```bash
$ firm related contact john_doe
```
```
Found 1 relationships for 'contact' entity with ID 'john_doe'

ID: interaction.megacorp_intro
Type: Call
Subject: Initial discussion about Project X
Interaction date: 2025-09-30 09:45:00 +02:00
Initiator ref: person.jane_smith
Primary contact ref: contact.john_doe
```

#### Custom queries

For deeper insights, use `firm query` with supports a SQL-like query language. This allows you to filter, traverse relationships, sort, and limit results all in one expression.

**Basic syntax:**
```
from <type> | <operation> | <operation> | ...
```

**Available operations:**
- `where <field> <operator> <value>` - Filter entities by field values
- `related([degrees]) [<type>]` - Traverse relationships
- `order <field> [asc|desc]` - Sort results
- `limit <n>` - Limit the number of results

**Example:** Find recent incomplete tasks related to active projects, sorted by due date:

```bash
$ firm query 'from project | where status == "in progress" | related(2) task | where is_completed == false | where due_date > 2025-01-01 | order due_date | limit 10'
```

You can filter by any field or metadata (`@type`, `@id`), traverse relationships multiple degrees deep, and compose operations to build the exact query you need.

## Installation
The Firm CLI is available to download via [Github Releases](https://github.com/42futures/firm/releases/). Install scripts are provided to make the process easy.

### Linux and macOS

```bash
curl -fsSL https://raw.githubusercontent.com/42futures/firm/main/install.sh | sudo bash
```

If you don't feel confident running it with `sudo`, you can:

1. **Download the release**
   - Go to [Github Releases](https://github.com/42futures/firm/releases/)
   - Download the appropriate archive for your operating system and architecture. You can run `uname -m` in your terminal if you're not sure which one to pick.

2. **Extract the archive**
```bash
tar -xzf firm-[OS]-[ARCH].tar.gz
```

3. **Navigate to the extracted directory**
```bash
cd firm-[OS]-[ARCH]
```

4. **Run the application**

**Option A:** Run from current directory
```bash
./firm
```

**Option B:** Install globally (recommended)
```bash
# Make executable (if needed)
chmod +x firm

# Move to system PATH
sudo mv firm /usr/local/bin/

# Now you can run firm from anywhere
firm
```

### Windows
```bash
irm https://raw.githubusercontent.com/42futures/firm/main/install.ps1 | iex
```

## Using Firm as a library
Beyond the CLI, you can integrate Firm's core logic directly into your own software using the `firm_core` and `firm_lang` Rust packages. This allows you to build more powerful automations and integrations on top of Firm.

First, add the Firm crates to your `Cargo.toml`:

```toml
[dependencies]
firm_core = { git = "https://github.com/42futures/firm.git" }
firm_lang = { git = "https://github.com/42futures/firm.git" }
```

You can then load a workspace, build the entity graph, and query it programmatically:

```rust
use firm_lang::workspace::Workspace;
use firm_core::EntityGraph;

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

// Traverse a relationship to another entity
let contact_ref = lead.get_field(FieldId::new("contact_ref"))?;
let contact = contact_ref.resolve_entity_reference(&graph)?;
```

This gives you full access to the underlying data structures, providing a foundation for building custom business automations.

## Architecture

Firm is organized as a Rust workspace with three crates:

### `firm_core`
Core data structures and graph operations.

- Entity data model
- Typed fields with references
- Relationship graph with query capabilities
- Entity schemas and validation

### `firm_lang`
DSL parsing and generation.

- Tree-sitter-based parser for `.firm` files
- Conversion between DSL and entities
- Workspace support for multi-file projects
- DSL generation from entities

Grammar is defined in [tree-sitter-firm](https://github.com/42futures/tree-sitter-firm).

### `firm_cli`

Command-line interface, making the Firm workspace interactive.

## Core concepts
Firm's data model is built on a few key concepts. Each concept is accessible declaratively through the `.firm` DSL for human-readable definitions, and programmatically through the Rust packages for building your own automations.

### Entities
Entities are the fundamental business objects in your workspace, like people, organizations, or projects. Each entity has a unique ID, a type, and a collection of fields.

**In the DSL**, you define an entity with its type and ID, followed by its fields in a block:

```firm
person john_doe {
    name = "John Doe"
    email = "john@doe.com"
}
```

**In Rust**, this corresponds to an `Entity` struct:

```rust
let person = Entity::new(EntityId::new("john_doe"), EntityType::new("person"))
    .with_field(FieldId::new("name"), "John Doe")
    .with_field(FieldId::new("email"), "john@doe.com");
```

### Fields
Fields are typed key-value pairs attached to an entity. Firm supports a rich set of types:

- `String`
- `Integer`
- `Float`
- `Boolean`
- `Currency`
- `DateTime`
- `List` of other values
- `Reference` to other fields or entities
- `Path` to a local file
- `Enum` for standard values

**In the DSL**, the syntax maps directly to these types:

```firm
my_task design_homepage {
    title = "Design new homepage"        // String
    priority = 1                         // Integer
    completed = false                    // Boolean
    budget = 5000.00 USD                 // Currency
    due_date = 2024-12-01 at 17:00 UTC   // DateTime
    tags = ["ui", "ux"]                  // List
    assignee = person.jane_doe           // Reference
    deliverable = path"./homepage.zip"   // Path
    status = enum"implementing"          // Enum
}
```

**In Rust**, these are represented by the `FieldValue` enum:

```rust
let value = FieldValue::Integer(42);
```

### Relationships and the entity graph
The power of Firm comes from connecting entities. You create relationships using `Reference` fields.

When Firm processes your workspace, it builds the *entity graph* representing of all your entities (as nodes) and their relationships (as directed edges). This graph is what allows for traversal and querying.

**In the DSL**, creating a relationship is as simple as referencing another entity's ID.

```firm
contact john_at_acme {
    person_ref = person.john_doe
    organization_ref = organization.acme_corp
}
```

**In Rust**, you build the graph by loading entities and calling the `.build()` method, which resolves all references into queryable links.

```rust
let mut graph = EntityGraph::new();
graph.add_entities(workspace.build()?.entities)?;
graph.build(); // Builds relationships from references

// Now you can traverse the graph
let contact = graph.get_entity(&EntityId::new("contact.john_at_acme"))?;
let person_ref = contact.get_field(FieldId::new("person_ref"))?;
let person = person_ref.resolve_entity_reference(&graph)?;
```

### Schemas

Schemas allow you to define and enforce a structure for your entities, ensuring data consistency. You can specify which fields are required or optional and what their types should be.

**In the DSL**, you can define a schema that other entities can adhere to:

```firm
schema custom_project {
    field {
        name = "title"
        type = "string"
        required = true
    }
    field {
        name = "budget"
        type = "currency"
        required = false
    }
    field {
        name = "status"
        type = "enum"
        required = false
        allowed_values = ["planned", "in-progress", "completed"]
    }
}

custom_project my_project {
    title  = "My custom project"
    budget = 42000 EUR
    status = enum"planned"
}
```

**In Rust**, you can define schemas programmatically to validate entities.

```rust
let schema = EntitySchema::new(EntityType::new("project"))
    .with_required_field(FieldId::new("title"), FieldType::String)
    .with_optional_field(FieldId::new("budget"), FieldType::Currency);

schema.validate(&some_project_entity)?;
```

## Built-in entities

Firm includes schemas for a range of built-in entities like Person, Organization, and Industry.

Firm's entity taxonomy is built on the [REA model (Resources, Events, Agents)](https://en.wikipedia.org/wiki/Resources,_Events,_Agents) with inspiration from [Schema.org](https://schema.org/Person), designed for flexible composition and efficient queries.

Every entity maps to a Resource (thing with value), an Event (thing that happens), or an Agent (thing that acts).

We separate objective reality from business relationships:

- **Fundamental entities** represent things that exist independently (`Person`, `Organization`, `Document`)
- **Contextual entities** represent your business relationships and processes (`Contact`, `Lead`, `Project`)

Entities reference each other rather than extending. One `Person` can be referenced by multiple `Contact`, `Employee`, and `Partner` entities simultaneously.

When the entity graph is built, all `Reference` values automatically create directed edges between entities. This enables traversal queries like "find all Tasks for Opportunities whose Contacts work at Organization X" without complex joins.
