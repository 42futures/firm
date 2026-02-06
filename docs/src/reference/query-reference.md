# Query reference

The Firm query language provides a SQL-like, compact syntax for querying your entity graph from the command line.

## Design philosophy

The query language is inspired by Microsoft's Kusto Query Language (KQL), using pipe operators to chain operations together. It's designed to be:

- **SQL-like and familiar** - Easy to learn if you know SQL
- **Compact for CLI use** - Minimal syntax for quick queries
- **Composable** - Build complex queries by piping operations together

The query grammar is defined using [Pest](https://pest.rs/) and can be found in `firm_lang/src/parser/query/grammar.pest`.

### Bag of entities model

Firm queries always operate on a "bag of entities". At every stage in query execution, you're processing complete, unmodified entities - we only read, filter, and traverse them, but never modify them or extract individual fields.

The `from` clause selects the initial set of entities, and every subsequent operation filters, expands, limits, or orders that entity set. This keeps the query language simple and focused on navigating the entity graph.

Optionally, a final **aggregation** clause can be added at the end of a query to compute a summary value (like a count or sum) or extract specific fields from the final entity set. Aggregations are the only operation that transforms the result from entities into a different shape.

## Basic syntax

All queries follow this structure:

```
from <entity_selector> | <operation> | <operation> | ... | <aggregation>
```

Start with a `from` clause, then chain operations using the pipe symbol `|`. Optionally end with an aggregation clause.

## Entity selector

Select which entities to start with:

```bash
# Select entities of a specific type
from task

# Select all entities (wildcard)
from *
```

## Operations

### where

Filter entities by field values or metadata:

```bash
# Filter by field value
from task | where is_completed == false

# Filter by metadata
from * | where @type == "task"
```

**Compound conditions:**

Combine multiple conditions in a single `where` clause using `and` or `or`:

```bash
# Match any of multiple values (OR)
from invoice | where status == "draft" or status == "sent"

# Require all conditions (AND)
from task | where is_completed == false and priority > 5

# Multiple OR conditions
from opportunity | where status == enum"open" or status == enum"negotiation" or status == enum"proposal"
```

You cannot mix `and` and `or` in the same `where` clause. Use separate `where` clauses to combine them:

```bash
# (status is draft OR sent) AND (amount > 1000)
from invoice | where status == "draft" or status == "sent" | where amount > 1000
```

**Chaining where clauses:**

Multiple `where` clauses joined by pipes act as implicit AND:

```bash
# These are equivalent:
from task | where is_completed == false | where priority > 5
from task | where is_completed == false and priority > 5
```

**Supported operators:**

- `==` - Equal to
- `!=` - Not equal to
- `>` - Greater than
- `<` - Less than
- `>=` - Greater than or equal to
- `<=` - Less than or equal to
- `contains` - String/list contains value
- `startswith` - String starts with value
- `endswith` - String ends with value
- `in` - Value in list

**Field references:**

- Regular fields: `field_name`
- Metadata fields: `@type`, `@id`

**Value types:**

```bash
# String (quoted)
where name == "John Doe"
where status == 'active'

# Number
where age > 30
where price <= 99.99

# Boolean
where is_completed == true
where active != false

# Currency
where budget >= 5000.00 USD

# Date/DateTime
where due_date > 2025-01-15
where created_at >= 2025-01-15 at 09:00 UTC

# Reference
where assignee_ref == person.john_doe

# Enum
where status == enum"active"

# Path
where file == path"./contracts/acme.pdf"

# List
where tags contains "urgent"
```

### related

Traverse relationships to find connected entities:

```bash
# Find all related entities (1 degree)
from organization | related

# Find related entities of a specific type
from organization | related task

# Traverse multiple degrees of separation
from organization | related(2)

# Combine degrees and type filter
from organization | related(2) task
```

**Syntax:**
- `related` - All related entities (1 degree)
- `related <type>` - Related entities of a specific type (1 degree)
- `related(<n>)` - All related entities (n degrees)
- `related(<n>) <type>` - Related entities of a specific type (n degrees)

### order

Sort results by a field:

```bash
# Sort ascending (default)
from task | order due_date

# Sort descending
from task | order due_date desc

# Sort ascending (explicit)
from task | order priority asc

# Sort by metadata
from * | order @type
```

**Syntax:**
- `order <field>` - Sort ascending
- `order <field> asc` - Sort ascending (explicit)
- `order <field> desc` - Sort descending

### limit

Limit the number of results:

```bash
# Get first 10 results
from task | limit 10

# Get top 5 high-priority tasks
from task | where priority > 8 | order priority desc | limit 5
```

**Syntax:** `limit <number>`

## Aggregations

Aggregations are optional clauses that go at the end of a query. They transform the entity set into a summary value or extracted fields. Only one aggregation can be used per query.

### select

Extract specific field values from entities:

```bash
# Select a single field
from person | select name

# Select multiple fields
from task | select name, status, due_date

# Include metadata fields
from task | where is_completed == false | select @id, name, due_date
```

**Syntax:** `select <field>, <field>, ...`

Fields can be regular field names or metadata fields (`@id`, `@type`). Missing fields appear as empty values.

### count

Count entities, optionally filtering by field presence:

```bash
# Count all matching entities
from task | where is_completed == false | count

# Count entities that have a specific field
from person | count email
```

**Syntax:**
- `count` - Count all entities in the result set
- `count <field>` - Count entities that have the specified field

### sum

Sum numeric field values across entities:

```bash
# Sum integer or float fields
from line_item | sum quantity

# Sum currency fields
from invoice | where status == "sent" | sum amount
```

**Syntax:** `sum <field>`

Works with integer, float, and currency fields. Entities missing the field are skipped. Currency values must all share the same currency code — mixed currencies produce an error.

### average

Compute the mean of a numeric field:

```bash
from task | average estimated_hours
```

**Syntax:** `average <field>`

Works with integer, float, and currency fields. Entities missing the field are skipped. Returns an error if no entities have the field.

### median

Compute the median of a numeric field:

```bash
from task | median estimated_hours
```

**Syntax:** `median <field>`

Works with integer, float, and currency fields. Entities missing the field are skipped. For an even number of values, returns the average of the two middle values. Returns an error if no entities have the field.

## Examples

### Find incomplete tasks

```bash
from task | where is_completed == false
```

### Find tasks assigned to a person

```bash
from task | where assignee_ref == person.john_doe
```

### Find high-value opportunities

```bash
from opportunity | where value >= 10000.00 USD | order value desc
```

### Find tasks for active projects

```bash
from project | where status == "active" | related task
```

### Count incomplete tasks

```bash
from task | where is_completed == false | count
```

### Total invoice amount

```bash
from invoice | where status == "sent" | sum amount
```

### Task summary with select

```bash
from task | where is_completed == false | order due_date | select @id, name, due_date
```

### Complex multi-hop query

```bash
from organization | where industry == "tech" | related(2) task | where is_completed == false | order due_date | limit 10
```

This query:
1. Starts with tech organizations
2. Finds entities within 2 degrees of separation that are tasks
3. Filters to incomplete tasks
4. Orders by due date
5. Limits to 10 results

## Query execution

Queries are executed left to right, with each operation transforming the result set:

```
from task                            → [all tasks]
| where is_completed == false        → [filtered tasks]
| related project                    → [related projects]
| order name                         → [sorted projects]
| limit 5                            → [top 5 projects]
```

Each operation receives the output of the previous operation and produces a new result set.

If an aggregation is present, it runs last and transforms the entity set into a result value:

```
from invoice                         → [all invoices]
| where status == "sent"             → [filtered invoices]
| sum amount                         → 15000.00 USD
```
