# Working with relationships

The power of Firm comes from connecting entities together to build a graph of your business. You create relationships using reference fields.

## Types of references

Firm supports two kinds of references:

### Entity references

Entity references point to an entire entity using the format `type.id`. This is the primary way to connect entities in Firm:

```firm
task design_homepage {
    assignee_ref = person.john_doe
}
```

### Field references

Field references point to a specific field on an entity using the format `type.id.field`:

```firm
task design_homepage {
    assignee_name = person.john_doe.name
}
```

Currently, Firm primarily uses entity references.

## Creating a reference

A reference field links one entity to another. Here's a simple example:

```firm
person john_doe {
    name = "John Doe"
    email = "john@example.com"
}

task design_homepage {
    name = "Design new homepage"
    assignee_ref = person.john_doe
    completed = false
}
```

The `assignee_ref` field contains a reference to `person.john_doe`, creating a connection from the task to the person.

## Multiple references

An entity can reference multiple other entities:

```firm
contact john_at_acme {
    person_ref = person.john_doe
    organization_ref = organization.acme_corp
    role = "CTO"
}
```

This creates two relationships:
- From `contact.john_at_acme` to `person.john_doe`
- From `contact.john_at_acme` to `organization.acme_corp`

## Building a connected graph

By connecting entities with references, you build a graph that represents your business relationships:

```firm
organization acme_corp {
    name = "Acme Corp"
}

person jane_smith {
    name = "Jane Smith"
}

contact jane_at_acme {
    person_ref = person.jane_smith
    organization_ref = organization.acme_corp
    role = "CEO"
}

project website_redesign {
    name = "Website Redesign"
    organization_ref = organization.acme_corp
}

task design_mockups {
    name = "Design mockups"
    project_ref = project.website_redesign
    assignee_ref = person.jane_smith
}
```

Now you can explore these relationships:
- Find all projects for Acme Corp
- Find all tasks assigned to Jane
- Find all contacts at Acme Corp
- Find all tasks for projects at Acme Corp

## Querying relationships

Use `firm related` to explore connections:

```bash
firm related organization acme_corp
```

This shows all entities connected to the organization.

For more complex queries, use `firm query`:

```bash
# Find all tasks for projects at Acme Corp
firm query 'from organization | where name contains "Acme" | related project | related task'
```

See the [Querying data](./querying.md) guide for more examples.
