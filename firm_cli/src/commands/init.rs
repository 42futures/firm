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

    // Create schemas directory
    fs::create_dir_all(&schemas_dir).map_err(|_| CliError::FileError)?;

    let schemas = defaults::all_default_schemas();
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
