use std::fs;
use std::io::Write;
use std::path::Path;

use firm_core::{Entity, EntityId, EntityType, FieldId, FieldValue};
use firm_lang::defaults;
use firm_lang::generate::{generate_dsl, generate_schema_dsl};
use inquire::{Confirm, Text};

use super::add::sanitize_entity_id;
use crate::errors::CliError;
use crate::ui;

/// Initialize a new Firm workspace with default schemas and files.
pub fn init_workspace(workspace_path: &Path) -> Result<(), CliError> {
    ui::header(&format!(
        "Initializing Firm workspace at {}",
        workspace_path.display()
    ));

    // Prompt for default schemas
    let include_schemas = Confirm::new("Include default schemas?")
        .with_default(true)
        .prompt()
        .map_err(|_| CliError::InputError)?;

    if include_schemas {
        create_default_schemas(workspace_path)?;
    }

    // Prompt for .gitignore
    create_or_update_gitignore(workspace_path)?;

    // Prompt for default entities
    let create_entities = Confirm::new("Add default entities (you and your organization)?")
        .with_default(true)
        .prompt()
        .map_err(|_| CliError::InputError)?;

    if create_entities {
        create_default_entities(workspace_path)?;
    }

    // Prompt for AI context
    let add_ai_context = Confirm::new("Add AI context (AGENTS.md)?")
        .with_default(true)
        .prompt()
        .map_err(|_| CliError::InputError)?;

    if add_ai_context {
        create_ai_context(workspace_path)?;
    }

    ui::success("Workspace initialized!");

    Ok(())
}

/// Create AI context file (AGENTS.md).
fn create_ai_context(workspace_path: &Path) -> Result<(), CliError> {
    let agents_md_path = workspace_path.join("AGENTS.md");

    // Check if AGENTS.md already exists
    if agents_md_path.exists() {
        let overwrite = Confirm::new("AGENTS.md already exists. Overwrite?")
            .with_default(false)
            .prompt()
            .map_err(|_| CliError::InputError)?;

        if !overwrite {
            ui::info("Skipped AI context creation");
            return Ok(());
        }
    }

    // Load AGENTS.md template from embedded file
    let agents_md_content = include_str!("../../AGENTS.md.template");

    fs::write(&agents_md_path, agents_md_content).map_err(|_| CliError::FileError)?;
    ui::success("Created AGENTS.md");

    Ok(())
}

/// Create default entities (person and organization) in main.firm file.
fn create_default_entities(workspace_path: &Path) -> Result<(), CliError> {
    let main_file_path = workspace_path.join("main.firm");

    // Check if main.firm already exists
    if main_file_path.exists() {
        let overwrite = Confirm::new("main.firm already exists. Overwrite?")
            .with_default(false)
            .prompt()
            .map_err(|_| CliError::InputError)?;

        if !overwrite {
            ui::info("Skipped entity creation");
            return Ok(());
        }
    }

    ui::info("Let's set up your core entities");

    // Prompt for person name
    let person_name = Text::new("Your name:")
        .prompt()
        .map_err(|_| CliError::InputError)?;

    // Prompt for organization name
    let org_name = Text::new("Your organization name:")
        .prompt()
        .map_err(|_| CliError::InputError)?;

    // Create person entity with sanitized ID (filters numbers, converts to snake_case)
    let person_id = sanitize_entity_id(person_name.clone());

    let person_entity = Entity::new(
        EntityId(format!("person.{}", person_id)),
        EntityType::new("person"),
    )
    .with_field(FieldId::new("name"), FieldValue::String(person_name));

    // Create organization entity with sanitized ID (filters numbers, converts to snake_case)
    let org_id = sanitize_entity_id(org_name.clone());

    let org_entity = Entity::new(
        EntityId(format!("organization.{}", org_id)),
        EntityType::new("organization"),
    )
    .with_field(FieldId::new("name"), FieldValue::String(org_name));

    // Generate DSL
    let entities = vec![person_entity, org_entity];
    let dsl_content = generate_dsl(&entities);

    // Write to main.firm
    fs::write(&main_file_path, dsl_content).map_err(|_| CliError::FileError)?;

    ui::success("Created main.firm with your person and organization");

    Ok(())
}

/// Create default schema files in the schemas/ directory.
fn create_default_schemas(workspace_path: &Path) -> Result<(), CliError> {
    let schemas_dir = workspace_path.join("schemas");
    let schemas = defaults::all_default_schemas();

    // Check which schema files already exist
    let existing_files: Vec<String> = schemas
        .iter()
        .map(|schema| format!("{}.firm", schema.entity_type))
        .filter(|filename| schemas_dir.join(filename).exists())
        .collect();

    // If files exist, ask for confirmation to overwrite
    if !existing_files.is_empty() {
        ui::warning(&format!(
            "{} schema file(s) already exist:",
            existing_files.len()
        ));
        for filename in &existing_files {
            ui::info(&format!("  - schemas/{}", filename));
        }

        let overwrite = Confirm::new("Overwrite existing schema files?")
            .with_default(false)
            .prompt()
            .map_err(|_| CliError::InputError)?;

        if !overwrite {
            ui::info("Skipped schema creation");
            return Ok(());
        }
    }

    // Create schemas directory
    fs::create_dir_all(&schemas_dir).map_err(|_| CliError::FileError)?;

    let spinner = ui::spinner(&format!("Creating {} default schemas", schemas.len()));

    for schema in &schemas {
        let schema_name = schema.entity_type.to_string();
        let file_path = schemas_dir.join(format!("{}.firm", schema_name));
        let dsl_content = generate_schema_dsl(&schema);

        let mut file = fs::File::create(&file_path).map_err(|_| CliError::FileError)?;
        file.write_all(dsl_content.as_bytes())
            .map_err(|_| CliError::FileError)?;
    }

    spinner.finish_with_message(format!(
        "Created {} schema files in schemas/",
        schemas.len()
    ));

    Ok(())
}

/// Create or update .gitignore file with Firm-specific entries.
fn create_or_update_gitignore(workspace_path: &Path) -> Result<(), CliError> {
    let gitignore_path = workspace_path.join(".gitignore");
    let gitignore_entries = "**/*.firm.graph\n";

    if gitignore_path.exists() {
        // File exists, ask if they want to update it
        let update = Confirm::new("Update existing .gitignore with Firm entries?")
            .with_default(true)
            .prompt()
            .map_err(|_| CliError::InputError)?;

        if !update {
            ui::info("Skipped .gitignore update");
            return Ok(());
        }

        // Read existing content
        let existing_content =
            fs::read_to_string(&gitignore_path).map_err(|_| CliError::FileError)?;

        // Check if entries already exist
        if existing_content.contains(".DS_Store") && existing_content.contains("*.firm.graph") {
            ui::info(".gitignore already contains Firm entries");
            return Ok(());
        }

        // Append entries
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(&gitignore_path)
            .map_err(|_| CliError::FileError)?;

        // Add a newline before our entries if file doesn't end with one
        let prefix = if existing_content.ends_with('\n') {
            ""
        } else {
            "\n"
        };
        file.write_all(format!("{}{}", prefix, gitignore_entries).as_bytes())
            .map_err(|_| CliError::FileError)?;

        ui::success("Updated .gitignore with Firm entries");
    } else {
        // File doesn't exist, ask if they want to create it
        let create = Confirm::new("Create .gitignore file?")
            .with_default(true)
            .prompt()
            .map_err(|_| CliError::InputError)?;

        if !create {
            ui::info("Skipped .gitignore creation");
            return Ok(());
        }

        // Create new file
        fs::write(&gitignore_path, gitignore_entries).map_err(|_| CliError::FileError)?;

        ui::success("Created .gitignore");
    }

    Ok(())
}
