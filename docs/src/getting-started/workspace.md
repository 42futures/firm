# Working with your workspace

## What is a workspace?

Firm operates on a "workspace": a directory containing all your `.firm` DSL files. The Firm CLI processes every file in this workspace to build a unified, queryable graph of your business.

You can add entities to your workspace either by using the CLI or by writing the DSL yourself.

## Adding entities with the CLI

Use `firm add` to generate new entities. The CLI will prompt you for the necessary information and generate the corresponding DSL.

```bash
$ firm add
```

The interactive prompt will guide you through selecting a type and providing field values.

### Non-interactive mode

You can also use `firm add` in non-interactive mode:

```bash
$ firm add --type organization --id megacorp --field name "Megacorp Ltd."
```

This is useful for scripting and automation.

## Writing DSL manually

Alternatively, you can create a `.firm` file and write the DSL yourself.

Create a new file (e.g., `organizations.firm`):

```firm
organization megacorp {
  name = "Megacorp Ltd."
  email = "mega@corp.com"
  urls = ["corp.com"]
}
```

Both methods achieve the same result: a new entity defined in your Firm workspace.

## Organizing your files

You can organize your `.firm` files however you like:
- Single file with all entities
- One file per entity type (e.g., `people.firm`, `organizations.firm`)
- Directory structure by project or client
- Any combination that makes sense for your business

Firm will discover and process all `.firm` files in your workspace directory recursively.

## Version control

Since your workspace is just plain text files, you can (and should!) put it in version control:

```bash
git init
git add .
git commit -m "Initial workspace"
```

This gives you:
- Full history of changes
- Collaboration with teammates
- Backup and recovery
- Branch-based workflows for planning

## Next steps

- Learn about [adding entities](../guide/adding-entities.md)
- Explore [querying your data](../guide/querying.md)
