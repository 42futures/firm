# Querying data

Once you have entities in your workspace, you can query them using the CLI.

## Getting an entity

To view the full details of a single entity, use `firm get` followed by the entity's type and ID.

```bash
$ firm get person john_doe
```
```
Found 'person' entity with ID 'john_doe'

ID: person.john_doe
Name: John Doe
Email: john@doe.com
```

## Listing entities

Use `firm list` to see all entities of a specific type.

```bash
$ firm list task
```
```
Found 7 entities with type 'task'

ID: task.design_homepage
Name: Design new homepage
Is completed: false
Assignee ref: person.jane_doe

...
```

## Custom queries

For deeper insights, use `firm query` which supports a SQL-like query language. This allows you to filter, traverse relationships, sort, and limit results in one expression.

### Query syntax

```
from <type> | <operation> | <operation> | ...
```

### Available operations

- `from <type>` - Selects the initial entity set
- `where <field> <operator> <value>` - Filter entities by field values
- `related([degrees]) [<type>]` - Traverse relationships
- `order <field> [asc|desc]` - Sort results
- `limit <n>` - Limit the number of results

### Examples

**Find all incomplete tasks:**
```bash
$ firm query 'from task | where is_completed == false'
```

**Find tasks assigned to a specific person:**
```bash
$ firm query 'from task | where assignee_ref == person.john_doe'
```

**Find invoices that are draft or sent:**
```bash
$ firm query 'from invoice | where status == "draft" or status == "sent"'
```

**Find recent incomplete tasks related to active projects, sorted by due date:**
```bash
$ firm query 'from project | where status == "in progress" | related(2) task | where is_completed == false | where due_date > 2025-01-01 | order due_date | limit 10'
```

### Query operators

You can filter by any field or metadata (`@type`, `@id`), traverse relationships multiple degrees deep, and compose operations to build the exact query you need.

**Comparison operators:**
- `==` - Equal
- `!=` - Not equal
- `>` - Greater than
- `<` - Less than
- `>=` - Greater than or equal
- `<=` - Less than or equal

For more details, see the [Query reference](../reference/query-reference.md).
