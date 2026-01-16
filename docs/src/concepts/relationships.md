# Relationships

The power of Firm comes from connecting entities using reference fields.

## Making relationships

When you add a reference field to an entity, you create a relationship:

```firm
contact john_at_acme {
    person_ref = person.john_doe
    organization_ref = organization.acme_corp
}
```

This creates two relationships:
- From `contact.john_at_acme` to `person.john_doe`
- From `contact.john_at_acme` to `organization.acme_corp`

## The entity graph

When Firm processes your workspace, it builds the **entity graph**: an in-memory data structure representing all your entities (as nodes) and their relationships (as directed edges).

The process:
1. Parse all `.firm` files in your workspace
2. Build entity objects with their fields
3. Read reference fields to identify relationships
4. Build directed edges between entities
5. Serialize and cache the graph for future queries

The entity graph is central to how Firm works. When you run queries, traverse relationships, or explore connections, you're working with this graph.

## Graph caching

When the graph is built, it's serialized and stored in your workspace. This way it can be used again for future queries if you don't wish to rebuild the graph on every interaction.

The `.gitignore` provided with `firm init` by default ignores the graph files.

When you rebuild the graph and it would overwrite the existing cached one, Firm backs it up. Your workspace will therefore usually have:
- `current.firm.graph` - The current graph
- `backup.firm.graph` - The previous graph

The contents of the serialized graph is JSON, so it can be used if you want to do more advanced queries and automations outside of the provided Rust crates.
