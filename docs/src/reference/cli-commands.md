# CLI commands

Complete reference for all Firm CLI commands.

## firm init

Initialize a new workspace.

```bash
firm init
```

**What it does:**
- Creates default schemas for common entity types
- Adds a `.gitignore` file
- Creates starter entities (you and your organization)
- Adds AI context documentation

**Options:**
- Interactive prompts guide you through setup

## firm add

Add a new entity to the workspace.

**Interactive mode:**
```bash
firm add
```

Prompts you for entity type, ID, and field values.

**Non-interactive mode:**
```bash
firm add --type <type> --id <id> [--field <name> <value>]...
```

**Examples:**
```bash
# Interactive
firm add

# Non-interactive
firm add --type person --id john_doe --field name "John Doe" --field email "john@example.com"

# Multiple fields
firm add --type task --id design_homepage --field title "Design homepage" --field priority 1
```

## firm list

List all entities of a specific type.

```bash
firm list <type>
```

**Examples:**
```bash
firm list person
firm list organization
firm list task
```

## firm get

Get details of a specific entity.

```bash
firm get <type> <id>
```

**Examples:**
```bash
firm get person john_doe
firm get organization acme_corp
firm get task design_homepage
```

## firm related

Find entities related to a specific entity.

```bash
firm related <type> <id>
```

**Examples:**
```bash
firm related organization acme_corp
firm related person john_doe
```

Shows all entities that reference the specified entity, or are referenced by it.

## firm query

Run custom queries using the query language.

```bash
firm query '<query>'
```

**Query syntax:**
```
from <type> | <operation> | <operation> | ...
```

**Operations:**
- `where <field> <operator> <value>` - Filter by field value
- `related([degrees]) [<type>]` - Traverse relationships
- `order <field> [asc|desc]` - Sort results
- `limit <n>` - Limit number of results

**Operators:**
- `==` - Equal
- `!=` - Not equal
- `>` - Greater than
- `<` - Less than
- `>=` - Greater than or equal
- `<=` - Less than or equal
- `contains` - List contains value

**Examples:**

Find incomplete tasks:
```bash
firm query 'from task | where is_completed == false'
```

Find tasks for a specific person:
```bash
firm query 'from task | where assignee_ref == person.john_doe'
```

Find organizations in tech industry:
```bash
firm query 'from organization | where industry == "tech"'
```

Multi-hop query:
```bash
firm query 'from project | where status == "active" | related(2) task | where is_completed == false'
```

Sort and limit:
```bash
firm query 'from task | where due_date > 2025-01-01 | order due_date asc | limit 10'
```

## firm --version

Show the installed version of Firm.

```bash
firm --version
```

## firm --help

Show help information.

```bash
firm --help
```

For command-specific help:
```bash
firm <command> --help
```

## Next steps

- Learn more about [querying](../guide/querying.md)
- Explore the [query language syntax](../guide/querying.md#custom-queries)
