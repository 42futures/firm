# Fields

Fields are typed key-value pairs attached to an entity. Firm supports a rich set of types to represent your business data.

## Field types

### String

Text values:

```firm
person john {
    name = "John Doe"
    bio = "Software engineer and entrepreneur"
}
```

For multiline strings, use triple quotes.

```firm
project website {
    description = """
        # Complete redesign
        Includes new homepage, about page, and contact form.
    """
}
```

Common indentation across the multiline string is removed when parsed.

### Integer

Numbers without a decimal place:

```firm
task design {
    priority = 1
    estimated_hours = 40
}
```

### Float

Numbers with a decimal place:

```firm
person john {
    height = 1.75
    weight = 70.5
}
```

### Boolean

True or false values:

```firm
task design {
    completed = false
    billable = true
}
```

### Currency

Monetary values with currency codes:

```firm
project website {
    budget = 5000.00 USD
    spent = 2500.00 USD
}
```

Firm supports ISO 4217 currency codes (USD, EUR, GBP, JPY, etc.).

### DateTime

Dates and times support three variants:

```firm
task design {
    # Date only (YYYY-MM-DD)
    start_date = 2025-01-15

    # Date and time (YYYY-MM-DD at HH:MM)
    due_date = 2025-01-15 at 17:00

    # Date and time with UTC offset (YYYY-MM-DD at HH:MM UTC+Z)
    created = 2025-01-15 at 17:00 UTC+3
}
```

**Timezone handling:**
- When you specify just a date (like `2025-01-15`), Firm assumes midnight (00:00) in your local timezone
- When you specify date and time without a timezone (like `2025-01-15 at 17:00`), Firm uses your local timezone
- When you specify a UTC offset (like `UTC+3` or `UTC-5`), Firm uses that timezone
- If you write `UTC` with no offset, it uses UTC+0
- Only `UTC` timezone offsets are supported (`EST`, `CET`, etc. are not)

### List

Collections of values. Lists are required to have homogeneous types (all items must be the same type):

```firm
person john {
    tags = ["developer", "manager", "consultant"]
    skills = ["rust", "python", "javascript"]
}
```

### Reference

Links to other entities:

```firm
task design {
    assignee_ref = person.jane_doe
    project_ref = project.website_redesign
}
```

References create relationships in the entity graph. See [Relationships](./relationships.md) for more details.

### Path

Local file paths:

```firm
project website {
    deliverable = path"./deliverables/website.zip"
    contract = path"/Users/john/Documents/contracts/megacorp_contract.pdf"
}
```

Paths are specified relative to the `.firm` source file. When parsed, they are transformed to be relative to the workspace root. Absolute paths are left unchanged.

### Enum

Predefined values:

```firm
task design {
    status = enum"in_progress"
    priority = enum"high"
}
```

Enums are useful when combined with [schemas](./schemas.md) that define allowed values.
