# Automations and AI assistants

Firm is designed to interoperate with automation tools and AI assistants. Because your data is stored as plain text files in a structured format, it's easy for both scripts and AI to read, write, and query.

## Why Firm works well with automation

- **Plain text**: Everything is in `.firm` files that any tool can read
- **Version controlled**: Changes are tracked in git, making automation safe
- **Queryable**: The CLI provides programmatic access to your data
- **Structured**: The DSL is unambiguous and parseable

## Using Firm in scripts

You can call Firm commands from bash scripts or other automation tools:

```bash
#!/bin/bash

# Create a new task
firm add --type task \
  --id "daily_standup_$(date +%Y%m%d)" \
  --field name "Daily standup" \
  --field due_date "$(date +%Y-%m-%d) at 09:00 UTC"

# Query incomplete tasks
firm query 'from task | where is_completed == false'
```

## AI context files

When you run `firm init`, it creates an `AGENTS.md` file that helps AI assistants understand your workspace. This file provides:

- CLI command reference and best practices
- Query language syntax and examples
- Common workflows for working with entities
- JSON output options for programmatic use

This gives AI coding assistants like Claude, GitHub Copilot, or Cursor the context they need to help you work with Firm data.

## Working with AI assistants

AI assistants can help you:

- **Generate entities**: "Create a new meeting for Jane at Acme Corp"
- **Query data**: "Show me all incomplete tasks assigned to John"
- **Build relationships**: "Link this task to the website project"
- **Analyze patterns**: "How is the sales pipeline looking?"
- **Write scripts**: "Create a script that generates a weekly report"

Because Firm files are plain text, LLMs can read your workspace and provide context-aware suggestions.

## MCP server

Firm includes a built-in [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) server that allows AI assistants to interact with your workspace through a standardized interface.

### Running the MCP server

Start the server with:

```bash
firm mcp
```

The server runs over stdio and exposes tools for querying, listing, and modifying entities. It's designed to run locally alongside your workspace.

Most MCP-compatible clients (like Claude Desktop or other AI assistants) can be configured to connect to local MCP servers.

### Remote deployment

The MCP server is built for local use, but you can deploy it to your own backend if you need remote access. How you handle authentication, security, and hosting is up to you.

For an example of how you could go about it, see the [Firm Remote MCP repository](https://github.com/42futures/firm-remote-mcp).

## Programmatic access

For more complex automation, you can use Firm as a Rust library. See the [Rust library guide](../library/getting-started.md) for details.
