# Relationships and the entity graph

The power of Firm comes from connecting entities. You create relationships using `Reference` fields.

## Creating relationships

When you add a reference field to an entity, you create a directed edge in the entity graph:

```firm
contact john_at_acme {
    person_ref = person.john_doe
    organization_ref = organization.acme_corp
}
```

This creates:
- An edge from `contact.john_at_acme` to `person.john_doe`
- An edge from `contact.john_at_acme` to `organization.acme_corp`

## The entity graph

When Firm processes your workspace, it builds the **entity graph**: a representation of all your entities (as nodes) and their relationships (as directed edges).

This graph is what enables:
- Traversal queries (e.g., "find all tasks for this project")
- Relationship exploration (e.g., "who works at this organization?")
- Multi-hop queries (e.g., "find tasks assigned to people at this organization")

## Traversing relationships

Use `firm related` to explore connections:

```bash
$ firm related organization acme_corp
```

This shows all entities that reference `organization.acme_corp`, or that are referenced by it.

### Multi-degree traversal

With the query language, you can traverse multiple degrees:

```bash
$ firm query 'from organization | where @id == "acme_corp" | related(2)'
```

This finds entities within 2 hops of the organization.

### Type-filtered traversal

You can filter by entity type during traversal:

```bash
$ firm query 'from organization | where @id == "acme_corp" | related task'
```

This finds only tasks related to the organization.

## Graph queries

The query language enables complex graph traversal:

```bash
$ firm query 'from organization | where industry == "tech" | related contact | related person | where skills contains "rust"'
```

This finds:
1. Organizations in the tech industry
2. Contacts at those organizations
3. People linked to those contacts
4. Filtered to those with "rust" in their skills

## Working with relationships in Rust

**In Rust**, you build the graph and traverse it programmatically:

```rust,no_run
// Build the graph
let mut graph = EntityGraph::new();
graph.add_entities(workspace.build()?.entities)?;
graph.build(); // Resolves all references

// Get an entity
let contact = graph.get_entity(&EntityId::new("contact.john_at_acme"))?;

// Follow a reference
let person_ref = contact.get_field(FieldId::new("person_ref"))?;
let person = person_ref.resolve_entity_reference(&graph)?;

// Find related entities
let related = graph.get_related_entities(&EntityId::new("organization.acme_corp"))?;
```

## Bidirectional relationships

Relationships in Firm are directional, but you can query in both directions:

```firm
task design {
    assignee = person.jane
}
```

You can:
- Start from the task and find the assignee
- Start from the person and find all tasks assigned to them

The graph supports both forward and backward traversal.

## Next steps

- Learn about [schemas](./schemas.md) for defining relationship constraints
- Explore [built-in entities](./built-in-entities.md) and their relationships
- See the [querying guide](../guide/querying.md) for more query examples
