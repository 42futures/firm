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

### Integer

Whole numbers:

```firm
task design {
    priority = 1
    estimated_hours = 40
}
```

### Float

Decimal numbers:

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

Supported currency codes include USD, EUR, GBP, and many others.

### DateTime

Dates and times with timezone:

```firm
task design {
    due_date = 2024-12-01 at 17:00 UTC
    created = 2024-01-15 at 09:30 EST
}
```

### List

Collections of values:

```firm
person john {
    tags = ["developer", "manager", "consultant"]
    skills = ["rust", "python", "javascript"]
}
```

Lists can contain any type of value, including other lists.

### Reference

Links to other entities:

```firm
task design {
    assignee = person.jane_doe
    project = project.website_redesign
}
```

References create relationships in the entity graph. See [Relationships](./relationships.md) for more details.

### Path

Local file paths:

```firm
project website {
    deliverable = path"./deliverables/website.zip"
    contract = path"./contracts/megacorp_contract.pdf"
}
```

Paths are relative to your workspace directory.

### Enum

Predefined values:

```firm
task design {
    status = enum"in_progress"
    priority = enum"high"
}
```

Enums are useful when combined with [schemas](./schemas.md) that define allowed values.

## Working with fields in Rust

**In Rust**, fields are represented by the `FieldValue` enum:

```rust,no_run
let name = FieldValue::String("John Doe".to_string());
let age = FieldValue::Integer(30);
let active = FieldValue::Boolean(true);
let budget = FieldValue::Currency(Money::new(5000.00, "USD"));
```

You can get and set fields on entities:

```rust,no_run
let person = entity.get_field(FieldId::new("name"))?;
entity.set_field(FieldId::new("email"), FieldValue::String("john@example.com".to_string()));
```


