# Default entity types

When you run `firm init`, you get a set of default schemas for common business entities. These schemas reflect a design philosophy focused on flexibility, composability, and real-world business modeling.

## Design philosophy

### REA model foundation

The default entity types are built on the [REA model (Resources, Events, Agents)](https://en.wikipedia.org/wiki/Resources,_Events,_Agents), a proven accounting and business modeling framework.

Every entity maps to one of these categories:
- **Resource** - Things with value (projects, documents)
- **Event** - Things that happen (interactions, transactions)
- **Agent** - Things that act (people, organizations)

### Fundamental vs. contextual entities

We separate objective reality from business relationships:

**Fundamental entities** represent things that exist independently:
- `person` - An individual human being
- `organization` - A company or group
- `file_asset` - A file or artifact

**Contextual entities** represent your business relationships and processes:
- `contact` - Your business relationship with a person
- `lead` - A sales opportunity
- `project` - A body of work

The same `person` can be a `contact` at one organization, an `employee` at yours, and a `partner` in a joint venture—all simultaneously.

### Composition over inheritance

Entities reference each other rather than extending each other. This provides more flexibility:

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
}
```

One `person` entity, multiple relationship contexts.

## Included default types

### Core entities
- **person** - An individual
- **organization** - A company or group
- **industry** - A business sector or classification

### Customer relations
- **account** - Business relationship with an organization
- **channel** - Communication or marketing channel
- **lead** - Potential business lead
- **contact** - Person in a business relationship context
- **interaction** - Communication or meeting
- **opportunity** - Potential sale or business deal

### Work management
- **strategy** - High-level, long-term plan or goal
- **objective** - Specific, measurable goal contributing to a strategy
- **key_result** - Measurable outcome tracking an objective
- **project** - Planned initiative to achieve objectives
- **task** - Single, actionable unit of work
- **review** - Periodic review or meeting

### Resources
- **file_asset** - Digital file or document

## Customization

These default schemas are a starting point. You can:
- Modify them to fit your needs
- Add new custom entity types
- Remove types you don't use

See the [Creating schemas guide](../guide/creating-schemas.md) for details on customization.
