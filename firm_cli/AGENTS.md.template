# Firm Workspace AI Guide

This workspace uses [Firm](https://github.com/42futures/firm), a text-based work management system for defining business entities and their relationships as code.

## How to interact with this workspace

**Prefer using the `firm` CLI for querying data.** The CLI provides validated, structured access to the workspace. You can edit `.firm` files directly when needed, but always run `firm build` afterward to validate your changes.

### Quick Overview Commands

Start with these commands to understand the workspace:

```bash
# See what entity types are available
firm list schema

# List all entities of a specific type
firm list <entity_type>

# Get details for a specific entity
firm get <entity_type> <entity_id>

# Explore relationships for an entity
firm related <entity_type> <entity_id>

# Query the workspace
firm query <query_string>
```

### Best Practices

1. **Prefer the CLI for queries**: Use `firm get`, `firm list`, and `firm related` rather than parsing `.firm` files manually
2. **Use non-interactive add**: Consider `firm add --type --id --field` for programmatic entity creation instead of creating files directly
3. **Validate after changes**: Always run `firm build` after changing a firm workspace to ensure your changes are valid
4. **Check schemas first**: Use `firm list schema` to see available entity types and their required fields
5. **Explore relationships**: Use `firm related` to understand connections between entities
6. **Use JSON output for automation**: Add `--format json` to any command for structured output

### CLI Reference

#### `firm build`
Validate the workspace and build the entity graph.

#### `firm get <target_type> <target_id>`
Get details for a specific entity or schema.

Examples:
```bash
# Get an entity
firm get contact john_at_acme

# Get a schema
firm get schema project
```

#### `firm list <target_type>`
List all entities of a type. Use `firm list schema` to see available schemas.

Example: `firm list task`

#### `firm related <entity_type> <entity_id>`
Show all entities related to the specified entity.

Example: `firm related contact john_at_acme`

#### `firm source <target_type> <target_id>`
Find the source file path where an entity or schema is defined. Useful when you've found an entity via `get`, `list`, or `query` and want to edit the source file.

Examples:
```bash
# Find where a person entity is defined
firm source person john_doe

# Find where a schema is defined
firm source schema project
```

Returns the absolute path to the `.firm` file containing the definition.

#### `firm add`
Add a new entity. Can be used interactively (without flags) or non-interactively for automation.

**Non-interactive mode** (recommended for AI agents):
```bash
# Add an entity with fields
firm add --type person --id john_doe \
  --field name "John Doe" \
  --field email "john@example.com"

# Add an entity with lists
firm add --type person --id jane_smith \
  --field name "Jane Smith" \
  --list urls string \
  --list-value urls "https://github.com/janesmith" \
  --list-value urls "https://linkedin.com/in/janesmith"
```

Arguments:
- `--type <entity_type>`: The entity type (required for non-interactive mode)
- `--id <entity_id>`: The entity ID (required for non-interactive mode)
- `--field <field_name> <value>`: Add a field (repeatable)
- `--list <field_name> <item_type>`: Declare a list field with its item type
- `--list-value <field_name> <value>`: Add an item to a list (repeatable)

The command validates against schemas and provides error messages for self-correction.
In cases where you're adding new entities, this is preferable to first editing the DSL and then building the workspace to verify.

### Querying with the Query Language

For advanced queries, use `firm query` with the Firm Query Language. This provides filtering, relationship traversal, sorting, and limiting capabilities without needing external tools like `jq`.

**Basic syntax:**
```
from <type or wildcard> | <operation> | <operation> | ...
```

**Available operations:**

1. **`where <field> <operator> <value>`** - Filter entities
   - Operators: `==`, `!=`, `>`, `<`, `>=`, `<=`, `contains`, `startswith`, `endswith`, `in`
   - Works with all field types and metadata fields (`@type`, `@id`)
   - Examples:
     ```bash
     firm query 'from task | where is_completed == false'
     firm query 'from person | where email contains "@acme.com"'
     firm query 'from * | where @type == "task"'
     ```

2. **`related([degrees]) [type]`** - Traverse relationships
   - Default: 1 degree of separation
   - Optional type filter to only return specific entity types
   - Examples:
     ```bash
     firm query 'from person | related task'
     firm query 'from project | related(2)'
     firm query 'from contact | related(2) interaction'
     ```

3. **`order <field> [asc|desc]`** - Sort results
   - Default: ascending
   - Works with regular fields and metadata (`@type`, `@id`)
   - Examples:
     ```bash
     firm query 'from task | order due_date'
     firm query 'from task | order due_date desc'
     firm query 'from * | order @type'
     ```

4. **`limit <n>`** - Limit number of results
   - Example: `firm query 'from task | limit 10'`

**Complex query example:**
```bash
# Find incomplete tasks due soon from active projects
firm query 'from project | where status == "in progress" | related(2) task | where is_completed == false | where due_date > 2025-01-01 | order due_date | limit 10'
```

**Composable operations:**
Operations can be chained in any order. The query engine processes them left-to-right, with each operation transforming the result set for the next operation.

**Field types in queries:**
- Strings: `"value"` or `'value'`
- Numbers: `42` or `3.14`
- Booleans: `true` or `false`
- Currency: `5000.50 USD`
- DateTime: `2025-01-15` or `2025-01-15 at 14:00` or `2025-01-15 at 14:00 UTC` `2025-01-15 at 14:00 UTC+3`
- References: `person.john_doe`
- Enums: `enum"value"` or `"value"`
- Paths: `path"./file.txt"`  or `"./file.txt"`

### Manual Workflows
Sometimes you might want to read or edit firm DSL directly.
This is always a possibility if the CLI doesn't provide the necessary functionality, but you should always remember to build the workspace with the CLI afterwards to verify that the changes were valid.

#### Adding a new contact and interaction

When adding contacts via DSL, create a `.firm` file with the contact, their organization, and any interactions:

```firm
organization acme {
    name = "Acme Corp"
}

person john_doe {
    name = "John Doe"
}

contact john_at_acme {
    person_ref = person.john_doe
    account_ref = account.acme
    role = "CEO"
}

interaction intro_call {
    type = "Call"
    subject = "Introduction call"
    interaction_date = 2025-12-30 at 14:00 UTC
    initiator_ref = person.me
    primary_contact_ref = contact.john_at_acme
}
```

#### Creating a task

Tasks can reference other entities as their source:

```firm
task follow_up {
    name = "Follow up with John"
    source_ref = interaction.intro_call
    assignee_ref = person.me
    due_date = 2026-01-05
    is_completed = false
}
```

### Output Formats

For structured output that's easy to parse programmatically, use `--format json`:

```bash
firm list task --format json
firm query 'from task | where is_completed == false' --format json

# Combine with jq to extract specific data
firm query 'from task | order due_date asc | limit 5' --format json | jq '.[].id'
```

### Notes for AI Agents

- The workspace structure is validated by schemas defined in the `schemas/` directory
- Entity IDs should be unique and follow snake_case naming
- References between entities create a queryable graph structure
- Always validate changes with `firm build` before committing
