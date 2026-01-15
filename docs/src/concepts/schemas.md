# Schemas

Schemas allow you to define and enforce a structure for your entities, ensuring data consistency. You can specify which fields are required or optional and what their types should be.

## Defining schemas

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
```

## Using schemas

Once defined, entities of that type will be validated against the schema:

```firm
custom_project my_project {
    title  = "My custom project"
    budget = 42000 EUR
    status = enum"planned"
}
```

If you omit a required field or use an invalid value, Firm will report an error.

## Field definitions

Each field in a schema can specify:

- **name**: The field identifier
- **type**: The field type (string, integer, float, boolean, currency, datetime, reference, path, enum, list)
- **required**: Whether the field must be present (true/false)
- **allowed_values**: For enum fields, the valid values

## Built-in schemas

Firm includes schemas for common entity types:

- `person` - Individuals
- `organization` - Companies and groups
- `contact` - Business relationships
- `project` - Bodies of work
- `task` - Units of work
- `lead` - Sales opportunities
- `interaction` - Communications

See [Built-in entities](./built-in-entities.md) for complete definitions.

## Custom schemas

You can define your own schemas to model your specific business domain:

```firm
schema customer {
    field {
        name = "company_name"
        type = "string"
        required = true
    }
    field {
        name = "annual_revenue"
        type = "currency"
        required = false
    }
    field {
        name = "segment"
        type = "enum"
        required = true
        allowed_values = ["enterprise", "mid-market", "smb"]
    }
}

customer acme {
    company_name = "Acme Corp"
    annual_revenue = 10000000 USD
    segment = enum"enterprise"
}
```

## Schemas in Rust

**In Rust**, you can define schemas programmatically to validate entities:

```rust,no_run
let schema = EntitySchema::new(EntityType::new("project"))
    .with_required_field(FieldId::new("title"), FieldType::String)
    .with_optional_field(FieldId::new("budget"), FieldType::Currency);

schema.validate(&some_project_entity)?;
```

This allows you to enforce constraints when building tools and automations.

## Schema inheritance

Currently, schemas don't support inheritance, but you can compose entities with references:

```firm
employee emp_001 {
    person_ref = person.john_doe
    organization_ref = organization.acme_corp
    role = "Senior Engineer"
    start_date = 2024-01-15 at 00:00 UTC
}
```

One `person` can be referenced by multiple `employee`, `contact`, and `partner` entities simultaneously.


