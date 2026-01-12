# API documentation

This page provides links to the full API documentation for Firm's Rust crates.

## Generating API docs

You can generate the full API documentation locally using `cargo doc`:

```bash
cd /path/to/firm
cargo doc --no-deps --open
```

This will generate and open the documentation in your browser.

## Key modules

### firm_core

**Entity management:**
- `Entity` - Represents a business object
- `EntityId` - Unique identifier for an entity
- `EntityType` - Type of entity (e.g., "person", "organization")

**Fields:**
- `FieldId` - Identifier for a field
- `FieldValue` - Typed field values
- `FieldType` - Field type definitions

**Graph:**
- `EntityGraph` - Graph of entities and relationships
- Graph query methods

**Schemas:**
- `EntitySchema` - Schema definitions
- Validation methods

### firm_lang

**Workspace:**
- `Workspace` - Multi-file workspace manager
- Loading methods (directory, file, string)
- Build methods

**Parser:**
- DSL parsing integration
- Error handling

**Generator:**
- `generate_dsl()` - Generate DSL from entities
- Formatting options

### firm_cli

The CLI crate is primarily for the binary. Most library users will only need `firm_core` and `firm_lang`.

## Online documentation

For the latest API documentation, you can also check:

- [GitHub repository](https://github.com/42futures/firm)
- Generated docs (when published to crates.io)

## Examples

For code examples, see:
- [Getting started with Rust](../library/getting-started.md)
- [API guide](../library/api-guide.md)
- The `examples/` directory in the repository

## Next steps

- Browse the [API guide](../library/api-guide.md) for common patterns
- Check the [architecture overview](./architecture.md) to understand the crate structure
