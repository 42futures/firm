# Entities

Entities are the fundamental business objects in your workspace, like people, organizations, or projects. Each entity has a unique ID, a type, and a collection of fields.

## Defining entities

**In the DSL**, you define an entity with its type and ID, followed by its fields in a block:

```firm
person john_doe {
    name = "John Doe"
    email = "john@doe.com"
}
```

**In Rust**, this corresponds to an `Entity` struct:

```rust,no_run
let person = Entity::new(EntityId::new("john_doe"), EntityType::new("person"))
    .with_field(FieldId::new("name"), "John Doe")
    .with_field(FieldId::new("email"), "john@doe.com");
```

## Entity structure

Every entity has:

- **Type**: What kind of entity this is (e.g., `person`, `organization`, `task`)
- **ID**: A unique identifier within its type (e.g., `john_doe`)
- **Fields**: Key-value pairs containing the entity's data

## Composite entity identifier

When Firm works with entities, it assigns each one a composite identifier of its type and ID. This means you can have several entities with the same ID, as long as they have different types.

For example, you could have:
- `person.hello` - A person named Hello
- `organization.hello` - An organization called Hello

When referencing entities, Firm uses the format `type.id`:

- `person.john_doe`
- `organization.megacorp`
- `task.design_homepage`
