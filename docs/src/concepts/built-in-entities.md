# Built-in entities

Firm includes schemas for a range of built-in entities like Person, Organization, and Industry.

## Entity taxonomy

Firm's entity taxonomy is built on the [REA model (Resources, Events, Agents)](https://en.wikipedia.org/wiki/Resources,_Events,_Agents) with inspiration from [Schema.org](https://schema.org/Person), designed for flexible composition and efficient queries.

Every entity maps to:
- **Resource** - Thing with value
- **Event** - Thing that happens
- **Agent** - Thing that acts

## Fundamental vs. contextual entities

We separate objective reality from business relationships:

**Fundamental entities** represent things that exist independently:
- `person` - An individual
- `organization` - A company or group
- `document` - A file or artifact

**Contextual entities** represent your business relationships and processes:
- `contact` - Your business relationship with a person
- `lead` - A sales opportunity
- `project` - A body of work
- `employee` - An employment relationship
- `partner` - A partnership relationship

## Composition over inheritance

Entities reference each other rather than extending. One `person` can be referenced by multiple `contact`, `employee`, and `partner` entities simultaneously.

```firm
person john_doe {
    name = "John Doe"
    email = "john@example.com"
}

contact john_at_acme {
    person_ref = person.john_doe
    organization_ref = organization.acme_corp
    role = "CTO"
}

employee john_employee {
    person_ref = person.john_doe
    organization_ref = organization.my_company
    role = "Consultant"
    start_date = 2024-01-15 at 00:00 UTC
}
```

## Entity graph

When the entity graph is built, all `Reference` values automatically create directed edges between entities. This enables traversal queries like "find all Tasks for Opportunities whose Contacts work at Organization X" without complex joins.

## Common entity types

### Agents

**person**
- Represents an individual
- Fields: name, email, phone, urls, etc.

**organization**
- Represents a company or group
- Fields: name, email, phone, urls, industry, etc.

### Resources

**project**
- A body of work
- Fields: name, description, status, budget, etc.

**task**
- A unit of work
- Fields: title, description, assignee, due_date, completed, etc.

**document**
- A file or artifact
- Fields: title, path, type, etc.

### Events

**interaction**
- A meeting, call, email, or other communication
- Fields: subject, type, date, participants, etc.

### Contextual

**contact**
- Your business relationship with a person at an organization
- Fields: person_ref, organization_ref, role, etc.

**lead**
- A sales opportunity
- Fields: name, contact_ref, value, status, etc.

**employee**
- An employment relationship
- Fields: person_ref, organization_ref, role, start_date, etc.

**partner**
- A partnership relationship
- Fields: organization_ref, type, start_date, etc.

## Viewing schemas

To see the full schema for any built-in entity type, you can:

1. Run `firm init` which creates schema definitions in your workspace
2. Check the [source code](https://github.com/42futures/firm) for schema definitions

## Custom entities

You can define your own entity types with [schemas](./schemas.md) to model your specific business domain.

## Next steps

- Learn how to [define custom schemas](./schemas.md)
- Explore [relationships between entities](./relationships.md)
- See [querying examples](../guide/querying.md)
