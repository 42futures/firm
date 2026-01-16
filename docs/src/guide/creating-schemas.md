# Creating schemas

Firm works on whatever schemas are in your workspace. If you've run `firm init`, you already have some default schemas with sensible defaults for common entity types like Person, Organization, Task, and Project.

You can customize these schemas any way you want, or create entirely new ones.

Schemas can be included in any `.firm` file and you can put them anywhere in your workspace. They will get discovered and included automatically.

## What schemas define

Each schema defines:
- Which fields are available for that entity type
- Which fields are required vs optional
- The data type of each field

This defines your data model.

## Example: Simple schema

Here's a basic schema for a task entity:

```firm
schema task {
    field {
        name = "name"
        type = "string"
        required = true
    }

    field {
        name = "description"
        type = "string"
        required = false
    }

    field {
        name = "completed"
        type = "boolean"
        required = true
    }
}
```

Now you can define entities that adhere to this schema:

```firm
task design_homepage {
    name = "Design new homepage"
    description = "Create mockups for the new homepage design"
    completed = false
}
```

## Available field types

When creating or customizing schemas, you can use these field types:

- **boolean** - True/false values
- **string** - Text values
- **integer** - Integer numbers
- **float** - Decimal numbers
- **currency** - Monetary values with currency codes
- **reference** - Links to other entities
- **list** - Lists of values
- **datetime** - Date and time values
- **path** - Local file paths
- **enum** - Enumerated values with allowed options

See the [Fields reference](../concepts/fields.md) for more details on how each field type works.

## Enum fields and allowed values

For enum fields in schemas, you must provide a set of allowed values. The enum type is intended to be a static set of options that constrains the field to only those specific values.

Here's an example of a schema withwith an enum field:

```firm
schema project {
    field {
        name = "name"
        type = "string"
        required = true
    }

    field {
        name = "status"
        type = "enum"
        allowed_values = ["planning", "active", "completed"]
        required = true
    }

    field {
        name = "budget"
        type = "float"
        required = false
    }
}
```
