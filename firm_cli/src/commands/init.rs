use std::fs;
use std::io::Write;
use std::path::Path;

use firm_lang::defaults;
use firm_lang::generate::generate_schema_dsl;
use inquire::Confirm;

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

    ui::success("Workspace initialized!");

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
