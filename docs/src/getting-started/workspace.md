# Your workspace

## What is a workspace?

Firm operates on a directory containing all your `.firm` DSL files. That's what we call your "workspace". Firm processes every file in this workspace to build a graph of your business.

You can interact with entities in your workspace either by using the Firm CLI or by writing Firm DSL yourself.

The CLI by default uses your current working directory as the root of the workspace. If you'd like to use a different workspace, you can specify it with `firm --workspace <path>`, where the path can be relative or absolute.

## Writing DSL

You can create `.firm` files and write the DSL yourself. These files are automatically included when they're in a Firm workspace.

DSL example (e.g., `organizations.firm`):

```firm
organization megacorp {
  name = "Megacorp Ltd."
  email = "mega@corp.com"
  urls = ["corp.com"]
}
```

See the [DSL reference](../reference/dsl-reference.md) for more.

## Organizing your files

You can organize your `.firm` files however you like:
- Single file with all entities
- One file per entity type (e.g., `people.firm`, `organizations.firm`)
- Directory structure by project or client
- Any combination that makes sense for your business

Firm will discover and process all `.firm` files in your workspace directory recursively.

## Ignoring files

You can exclude specific directories or files from being processed by creating a `.firmignore` file in your workspace root. This uses the same pattern syntax as `.gitignore`.

For example, to ignore git worktrees stored in `.worktrees/`:

```gitignore
# .firmignore
**/.worktrees/
```

Common use cases:
- Exclude git worktrees (e.g., `.worktrees/`, `.git/worktrees/`)
- Exclude backup directories
- Exclude specific files you don't want in your graph

If no `.firmignore` exists, Firm will fall back to reading `.gitignore` patterns. If neither file exists, all `.firm` files in the workspace are loaded.

## Version control

Since your workspace is just plain text files, you can (and should!) put it in version control:

```bash
git init
git add .
git commit -m "Initial workspace"
```

This gives you:
- Auditable history of changes
- Collaboration with teammates
- Backup and recovery
- Branch-based workflows for planning


