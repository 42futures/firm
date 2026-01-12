# Adding entities

There are two ways to add entities to your workspace: using the CLI or writing DSL manually.

## Using the CLI

The `firm add` command provides an interactive way to create entities:

```bash
$ firm add
```

The CLI will prompt you for:
1. Entity type (e.g., person, organization, task)
2. Entity ID (unique identifier)
3. Field values based on the entity type's schema

### Example session

```bash
$ firm add
```
```
Adding new entity

> Type: person
> ID: john_doe
> Name: John Doe
> Email: john@example.com

Writing generated DSL to file my_workspace/generated/person.firm
```

### Non-interactive mode

For scripting and automation, you can provide all values as arguments:

```bash
$ firm add --type person --id john_doe --field name "John Doe" --field email "john@example.com"
```

## Writing DSL manually

You can create entities by writing `.firm` files directly. This gives you more control and is often faster for experienced users.

### Basic syntax

```firm
<type> <id> {
  field_name = value
  another_field = value
}
```

### Example

Create a file `people.firm`:

```firm
person john_doe {
  name = "John Doe"
  email = "john@example.com"
  phone = "+1-555-0100"
}

person jane_smith {
  name = "Jane Smith"
  email = "jane@example.com"
}
```

### Field types

Firm supports multiple field types:

```firm
person example {
  name = "Example Person"              // String
  age = 30                              // Integer
  height = 1.75                         // Float
  active = true                         // Boolean
  balance = 1000.00 USD                 // Currency
  created = 2024-01-01 at 12:00 UTC     // DateTime
  tags = ["developer", "manager"]       // List
  manager_ref = person.jane_smith       // Reference
  resume = path"./resumes/example.pdf"  // Path
  status = enum"active"                 // Enum
}
```

See [Fields](../concepts/fields.md) for detailed information about field types.

## When to use each method

**Use the CLI when:**
- You're just getting started
- You want validation and guidance
- You're adding a single entity

**Write DSL manually when:**
- You're comfortable with the syntax
- You're adding multiple entities at once
- You want full control over file organization
- You're editing existing entities

## Next steps

- Learn how to [query your entities](./querying.md)
- Understand [entity schemas](../concepts/schemas.md)
