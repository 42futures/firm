# Changelog

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2026-01-10

### Added

- **Query Engine**: SQL-like query language for filtering, traversing, sorting, and limiting entities
  - New `firm query` command with composable operations: `where`, `related`, `order`, `limit`
  - Supports all field types and operators (==, !=, >, <, >=, <=, contains, startswith, endswith, in)
  - Example: `firm query 'from project | where status == "in progress" | related(2) task | limit 10'`
- **Workspace Initialization**: New `firm init` command for setting up workspaces
  - Creates default schemas as editable DSL files in workspace
  - Generates .gitignore configuration for graph files
  - Optional starter entities for first-time setup
  - Creates AGENTS.md file for AI context
- **Enum Field Type**: New field type for constrained string values
  - Case-insensitive enum matching with interactive picker in CLI
  - Build-time validation with helpful error messages
  - Default schemas updated with sensible enum defaults
- **Non-Interactive Add**: Automation support for `firm add` command
  - Use `--type`, `--id`, and `--field` arguments to add entities programmatically
  - List support with `--list <name> <type>` and `--list-value <name> <value>`
  - Validation against schemas with helpful error messages
- **Field Ordering**: Field schemas now preserve insertion order
  - Explicit `order` attribute on field schemas
  - Entity fields retain DSL ordering when displayed
- Path autocomplete in CLI add command
- GitHub issue templates and pull request workflow

### Fixed

- Path fields now correctly resolve relative to source files
- Both interactive and non-interactive add modes now use pre-built graph

### Changed

- **BREAKING**: Built-in schemas removed from core package. Existing workspaces must run `firm init` to create schema files.
- **BREAKING**: Entity field access changed from direct field ID indexing to `get_field()` method or entity field index.
- **BREAKING**: Module reorganization - DSL parsing moved from `parser::` to `parser::dsl::` namespace in firm_lang crate.
- Entity fields now stored in vector instead of hash map
- Updated README with query language documentation

## [0.3.0] - 2025-10-13

### Added

- Tree-sitter grammar repo as a root-level submodule.
- A new README which unifies concepts across core, language and CLI.
- A shared workspace example.
- Pretty output support.
- Inline documentation for most features.
- Github CI pipeline for building and releasing binaries.

### Fixed

- Cargo configs for crates in the workspace.
- Broken test referencing the workspace example.

### Changed

- Migrated separate crate repo to a single Rust workspace.
- CLI add action now also outputs the generated entity.
- Refactoring and documentation cleanup.
