//! Static documentation content for the DSL reference tool.
//!
//! This content is adapted from the Firm documentation and provides
//! a comprehensive reference for the DSL syntax and query language.

/// DSL syntax reference documentation.
pub const DSL_REFERENCE: &str = r#"# Firm DSL Reference

The Firm DSL is used to define entities and schemas in `.firm` files.

## Entity Blocks

Define an entity with a type and ID:

```firm
person john_doe {
    name = "John Doe"
    email = "john@example.com"
}
```

Syntax: `<entity_type> <entity_id> { <fields> }`

## Schema Blocks

Define a schema for an entity type:

```firm
schema task {
    field {
        name = "name"
        type = "string"
        required = true
    }

    field {
        name = "is_completed"
        type = "boolean"
        required = false
    }
}
```

Syntax: `schema <schema_name> { <field_definitions> }`

## Field Types

### String
```firm
name = "John Doe"
description = """
Multiline string
with triple quotes.
"""
```

### Number
```firm
age = 30
height = 1.75
```

### Boolean
```firm
is_completed = true
is_active = false
```

### Currency
```firm
budget = 5000.00 USD
cost = 299.99 EUR
```

Syntax: `<amount> <CURRENCY_CODE>`

### Date
```firm
start_date = 2025-01-15
```

Syntax: `YYYY-MM-DD`

### DateTime
```firm
due_date = 2025-01-15 at 17:00
created = 2025-01-15 at 17:00 UTC+3
meeting = 2025-01-15 at 09:00 UTC
```

Syntax: `YYYY-MM-DD at HH:MM [UTC[+/-]Z]`

### Reference
```firm
// Entity reference
assignee_ref = person.john_doe

// Field reference
assignee_name = person.john_doe.name
```

Syntax: `<type>.<id>` or `<type>.<id>.<field>`

### List
```firm
tags = ["urgent", "frontend", "bug"]
urls = [
    "https://example.com",
    "https://github.com",
]
```

Lists are homogeneous (all items must be the same type). Trailing commas are allowed.

### Path
```firm
contract = path"./contracts/acme.pdf"
```

Syntax: `path"<path>"`

### Enum
```firm
status = enum"active"
priority = enum"high"
```

Syntax: `enum"<value>"`

## Comments

```firm
// Single-line comment
person john_doe {
    name = "John Doe" // Inline comment
}

/*
Multi-line comment
spanning multiple lines.
*/
```

## Identifiers

Entity types, IDs, field names, and schema names must:
- Start with a letter or underscore
- Contain only letters, numbers, and underscores
- Use snake_case convention

Valid: `person`, `john_doe`, `my_organization`, `_private`
Invalid: `123abc`, `my-entity`, `my.field`
"#;

/// Query language reference documentation.
pub const QUERY_REFERENCE: &str = r#"# Firm Query Language Reference

## Basic Syntax

```
from <entity_selector> | <operation> | <operation> | ...
```

## Entity Selector

```bash
from task    # Select entities of a specific type
from *       # Select all entities (wildcard)
```

## Operations

### where - Filter entities

```bash
from task | where is_completed == false
from * | where @type == "task"
from task | where is_completed == false | where priority > 5
```

**Operators:** `==`, `!=`, `>`, `<`, `>=`, `<=`, `contains`, `startswith`, `endswith`, `in`

**Metadata fields:** `@type`, `@id`

**Value types in queries:**
- String: `"John Doe"` or `'active'`
- Number: `30`, `99.99`
- Boolean: `true`, `false`
- Currency: `5000.00 USD`
- Date: `2025-01-15`
- DateTime: `2025-01-15 at 09:00 UTC`
- Reference: `person.john_doe`
- Enum: `enum"active"`
- Path: `path"./file.txt"`

### related - Traverse relationships

```bash
from organization | related              # All related (1 degree)
from organization | related task         # Related tasks (1 degree)
from organization | related(2)           # All related (2 degrees)
from organization | related(2) task      # Related tasks (2 degrees)
```

### order - Sort results

```bash
from task | order due_date           # Ascending (default)
from task | order due_date desc      # Descending
from task | order priority asc       # Ascending (explicit)
from * | order @type                 # Sort by metadata
```

### limit - Limit results

```bash
from task | limit 10
from task | where priority > 8 | order priority desc | limit 5
```

## Example Queries

```bash
# Find incomplete tasks
from task | where is_completed == false

# Find tasks assigned to a person
from task | where assignee_ref == person.john_doe

# Find high-value opportunities
from opportunity | where value >= 10000.00 USD | order value desc

# Find tasks for active projects
from project | where status == "active" | related task

# Complex multi-hop query
from organization | where industry == "tech" | related(2) task | where is_completed == false | order due_date | limit 10
```

## Query Execution

Queries execute left to right, each operation transforming the result set:

```
from task          → [all tasks]
| where status     → [filtered tasks]
| related project  → [related projects]
| order name       → [sorted projects]
| limit 5          → [top 5 projects]
```
"#;
