# Schemas

Schemas define the structure and validation rules for entity types in your workspace. They provide consistency and ensure data integrity across your business data.

## What schemas do

Schemas specify:
- Which fields are available for an entity type
- Which fields are required vs optional
- The expected data type for each field

## Validation and flexibility

When Firm builds the entity graph, it validates each entity against its schema. The validation rules are:

- **Fields in the schema must match the defined types** - A field marked as `boolean` cannot contain a number
- **Required fields must be present** - If a field is marked `required = true`, the entity must have it
- **Entities can have fields not defined in their schema** - Schemas define minimum requirements, not maximum constraints

This gives you structure where you need it, while allowing flexibility for additional custom data.

## Example

```firm
schema task {
    field {
        name = "name"
        type = "string"
        required = true
    }

    field {
        name = "completed"
        type = "boolean"
        required = true
    }
}

task design_homepage {
    name = "Design new homepage"
    completed = false
    custom_priority = "high"  # Not in schema, but allowed
}
```

This entity is valid because:
- It has all required fields (`name` and `completed`)
- Those fields have the correct types
- The extra `custom_priority` field is allowed

## Default schemas

When you run `firm init`, you get default schemas for common entity types like `person`, `organization`, `task`, and `project`. See the [Quick start guide](../getting-started/quick-start.md) to learn more about initializing a workspace.
