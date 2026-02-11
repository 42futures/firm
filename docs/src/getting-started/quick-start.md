# Quick start

## Initialize your workspace

The easiest way to start is by running `firm init` in an empty directory:

```bash
mkdir my_workspace
cd my_workspace
firm init
```

This interactive command will help you set up your workspace by:
- Creating default schemas for common entity types (Person, Organization, Task, etc.)
- Adding a `.gitignore` file for Firm's graph files
- Creating starter entities (you and your organization)
- Adding AI context documentation for AI coding assistants

Once initialized, your workspace is ready to use!

## Add your first entity

Use `firm add` to generate new entities. The CLI will prompt you for the necessary info and generate corresponding DSL.

```bash
$ firm add
```
```
Adding new entity

> Type: organization
> ID: megacorp
> Name: Megacorp Ltd.
> Email: mega@corp.com
> Urls: ["corp.com"]

Writing generated DSL to file my_workspace/generated/organization.firm
```

You can also use `firm add` non-interactively by providing its type, ID and fields:

```bash
$ firm add --type organization --id megacorp --field name "Megacorp Ltd."
```

## View your entities

Use `firm list` to see all entities of a specific type:

```bash
$ firm list organization
```
```
Found 1 entities with type 'organization'

ID: organization.megacorp
Name: Megacorp Ltd.
Email: mega@corp.com
Urls: ["corp.com"]
```

## Query your data

Once you have a few entities, you can search and filter them using Firm's query language:

```bash
$ firm query 'from organization | where name contains "Mega"'
```
