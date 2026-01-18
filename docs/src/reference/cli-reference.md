# CLI reference

Complete reference for the Firm command-line interface.

## Global options

These options apply to all commands:

### --workspace (-w)

Specify the workspace directory:

```bash
firm --workspace ./my_workspace list task
firm -w /absolute/path/to/workspace get person john_doe
```

Default: Current working directory

### --cached (-c)

Use the cached entity graph instead of rebuilding:

```bash
firm --cached list task
firm -c query 'from task | where is_completed == false'
```

Default: false (graph is rebuilt before each command)

### --verbose (-v)

Enable verbose logging output:

```bash
firm --verbose build
firm -v list task
```

### --format (-f)

Specify output format:

```bash
firm --format json list task
firm -f pretty get person john_doe
```

Options:
- `pretty` (default) - Human-readable formatted output
- `json` - JSON output for programmatic use

## Commands

### init

Initialize a new Firm workspace with default schemas and files.

```bash
firm init
```

This interactively gives you the options create:
- Default entity type schemas (person, organization, task, etc.)
- `.gitignore` file for graph files
- Starter entities (you and your organization)
- `AGENTS.md` file with AI assistant context

### build

Build the workspace and entity graph.

```bash
firm build
```

This:
- Parses all `.firm` files in the workspace
- Validates entities against their schemas
- Builds the entity graph with relationships
- Saves the graph to `current.firm.graph`

**Note:** Most commands automatically build the graph unless `--cached` is used.

### get

Get details of a specific entity.

```bash
firm get <entity_type> <entity_id>
```

**Arguments:**
- `entity_type` - The type of entity (e.g., `person`, `organization`, `task`)
- `entity_id` - The ID of the entity (e.g., `john_doe`)

**Examples:**

```bash
firm get person john_doe
firm get organization acme_corp
firm get task design_homepage
```

### list

List all entities of a specific type, or list all schemas.

```bash
firm list <entity_type>
```

**Arguments:**
- `entity_type` - The type to list, or `schema` to list all schemas

**Examples:**

```bash
# List all tasks
firm list task

# List all people
firm list person

# List all available schemas (special case)
firm list schema
```

**Note:** `firm list schema` is a special case that lists all schema definitions in your workspace, not entities of type "schema". This is useful for discovering what entity types are available and what fields they support.

### related

Get entities related to a specific entity.

```bash
firm related <entity_type> <entity_id> [--direction <dir>]
```

**Arguments:**
- `entity_type` - The type of entity
- `entity_id` - The ID of the entity

**Options:**
- `--direction` or `-d` - Filter by relationship direction
  - `to` - Only incoming relationships (entities referencing this one)
  - `from` - Only outgoing relationships (entities this one references)
  - No direction specified - Both incoming and outgoing

**Examples:**

```bash
# All related entities (both directions)
firm related organization acme_corp

# Only entities that reference this organization
firm related organization acme_corp --direction to

# Only entities this person references
firm related person john_doe --direction from
firm related person john_doe -d from
```

### add

Add a new entity to the workspace.

**Interactive mode** (prompts for input):

```bash
firm add
firm add path/to/file.firm
```

**Non-interactive mode** (all details provided):

```bash
firm add [to_file] --type <type> --id <id> [--field <name> <value>]...
```

**Options:**
- `to_file` - Optional path to the `.firm` file to write to
- `--type` - Entity type (required for non-interactive mode)
- `--id` - Entity ID (required for non-interactive mode)
- `--field <name> <value>` - Add a field (repeatable)
- `--list <name> <item_type>` - Declare a list field (repeatable)
- `--list-value <name> <value>` - Add an item to a list field (repeatable)

**Examples:**

```bash
# Interactive mode
firm add

# Non-interactive with fields
firm add --type person --id jane_smith \
  --field name "Jane Smith" \
  --field email "jane@example.com"

# Write to specific file
firm add people.firm --type person --id bob_jones \
  --field name "Bob Jones"

# With list fields
firm add --type person --id alice_wong \
  --field name "Alice Wong" \
  --list skills string \
  --list-value skills "rust" \
  --list-value skills "python"
```

### query

Query entities using the Firm query language.

```bash
firm query '<query_string>'
```

**Arguments:**
- `query_string` - A query in the Firm query language

**Examples:**

```bash
# Find incomplete tasks
firm query 'from task | where is_completed == false'

# Find high-value opportunities
firm query 'from opportunity | where value >= 10000.00 USD'

# Find tasks for active projects
firm query 'from project | where status == "active" | related task'

# Complex multi-hop query
firm query 'from organization | where industry == "tech" | related(2) task | where is_completed == false | limit 10'

# Sort and limit
firm query 'from task | order due_date desc | limit 5'
```

See the [Query reference](./query-reference.md) for complete query language documentation.

### source

Find the source file path where an entity or schema is defined.

```bash
firm source <target_type> <target_id>
```

**Arguments:**
- `target_type` - Entity type (e.g., `person`, `organization`) or `schema`
- `target_id` - Entity ID or schema name

**Examples:**

```bash
# Find where a person entity is defined
firm source person john_doe

# Find where an organization is defined
firm source organization acme_corp

# Find where a schema is defined
firm source schema project

# Output as JSON
firm --format json source person john_doe
```

**Output:**
Returns the absolute path to the `.firm` file containing the definition. This is useful for locating and editing entity or schema definitions.

## Exit codes

- `0` - Success
- `1` - Failure (error details printed to stderr)

## Examples

### Initialize and explore a workspace

```bash
# Create a new workspace
mkdir my_workspace && cd my_workspace
firm init

# List all schemas
firm list schema

# List all people
firm list person

# Get details of a person
firm get person me
```

### Add entities

```bash
# Add interactively
firm add

# Add non-interactively
firm add --type organization --id acme \
  --field name "Acme Corp" \
  --field email "contact@acme.com"

firm add --type contact --id john_at_acme \
  --field person_ref "person.john_doe" \
  --field organization_ref "organization.acme"
```

### Query and explore

```bash
# Find all incomplete tasks
firm query 'from task | where is_completed == false'

# Find organizations and their contacts
firm query 'from organization | related contact'

# Output as JSON for scripting
firm --format json query 'from task | limit 10' | jq '.[].id'
```
