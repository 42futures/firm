# DSL reference

The Firm DSL (Domain-Specific Language) is used to define entities and schemas in plain text `.firm` files.

## Design philosophy

The Firm language is inspired by HashiCorp Configuration Language (HCL) but simplified for business entity modeling. Firm keeps HCL's clean block syntax and nested blocks while reducing complexity with a more restricted grammar and focused type system.

The syntax is intentionally simple, making it:
- Easy to write by hand
- Straightforward to parse programmatically
- Simple to generate from tooling

This makes it suited for both human authoring and machine generation in business workflows.

The grammar is defined in the [tree-sitter-firm](https://github.com/42futures/tree-sitter-firm) repository, which is a submodule of this project. Using Tree-sitter also enables editor integrations like syntax highlighting and code navigation in editors such as Zed.

## Blocks

Blocks are the fundamental structural elements in Firm DSL, enclosed in curly braces `{ }`.

### Entity blocks

Define an entity with a type and ID:

```firm
person john_doe {
    name = "John Doe"
    email = "john@example.com"
}
```

Syntax: `<entity_type> <entity_id> { <fields> }`

### Schema blocks

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

### Nested blocks

Schemas use nested blocks for field definitions:

```firm
schema project {
    field {
        name = "status"
        type = "enum"
        allowed_values = ["planning", "active", "completed"]
        required = true
    }
}
```

## Fields

Fields are key-value pairs defined with the assignment operator `=`.

Syntax: `<field_name> = <value>`

## Field types

### String

Single-line strings:

```firm
name = "John Doe"
```

Multiline strings with triple quotes:

```firm
description = """
This is a multiline string.
It can span multiple lines.
"""
```

### Number

Integers and floats:

```firm
age = 30
height = 1.75
```

### Boolean

True or false values:

```firm
is_completed = true
is_active = false
```

### Currency

Monetary values with ISO 4217 currency codes:

```firm
budget = 5000.00 USD
cost = 299.99 EUR
```

Syntax: `<amount> <CURRENCY_CODE>`

### Date

ISO 8601 date format:

```firm
start_date = 2025-01-15
```

Syntax: `YYYY-MM-DD`

### DateTime

Date with time and optional timezone:

```firm
due_date = 2025-01-15 at 17:00
created = 2025-01-15 at 17:00 UTC+3
meeting = 2025-01-15 at 09:00 UTC
```

Syntax: `YYYY-MM-DD at HH:MM [UTC[+/-]Z]`

### Reference

Entity references:

```firm
assignee_ref = person.john_doe
```

Field references:

```firm
assignee_name = person.john_doe.name
```

Syntax: `<type>.<id>` or `<type>.<id>.<field>`

### List

Homogeneous lists (all items must be the same type):

```firm
tags = ["urgent", "frontend", "bug"]
urls = ["https://example.com", "https://github.com"]
```

Trailing commas are allowed:

```firm
tags = [
    "urgent",
    "frontend",
    "bug",
]
```

### Path

File path literals:

```firm
contract = path"./contracts/acme.pdf"
deliverable = path"/Users/john/Documents/report.pdf"
```

Syntax: `path"<path>"`

### Enum

Enumerated values:

```firm
status = enum"active"
priority = enum"high"
```

Syntax: `enum"<value>"`

## Comments

Single-line comments:

```firm
// This is a single-line comment
person john_doe {
    name = "John Doe" // Inline comment
}
```

Multi-line comments:

```firm
/*
This is a multi-line comment.
It can span multiple lines.
*/
person john_doe {
    name = "John Doe"
}
```

## Identifiers

Identifiers (entity types, entity IDs, field names, schema names) must:
- Start with a letter or underscore
- Contain only letters, numbers, and underscores
- Use snake_case convention

Valid identifiers: `person`, `john_doe`, `my_organization`, `_private`

Invalid identifiers: `123abc`, `my-entity`, `my.field`

## Why not YAML or JSON?

The Firm language is optimized for readability and compactness while retaining rich typing information:

- **More scannable than YAML** - Block syntax makes entity boundaries clear at a glance
- **Less verbose than JSON** - No need for extensive quoting and bracket nesting
- **Native support for business concepts** - Built-in support for currency, dates, and references
- **Schema definitions** - First-class support for defining custom entity types

The result is a format that's both human-friendly for manual editing and machine-friendly for programmatic generation.
