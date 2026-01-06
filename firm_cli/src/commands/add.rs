use convert_case::{Case, Casing};
use firm_core::graph::EntityGraph;
use firm_core::{Entity, EntitySchema, compose_entity_id};
use firm_lang::generate::generate_dsl;
use firm_lang::workspace::Workspace;
use inquire::{Confirm, Select, Text};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

use super::{
    build_graph, build_workspace, field_prompt::prompt_for_field_value, load_workspace_files,
};
use crate::errors::CliError;
use crate::ui::{self, OutputFormat};

pub const GENERATED_DIR_NAME: &str = "generated";
pub const FIRM_EXTENSION: &str = "firm";

/// Wrapper for EntitySchema that customizes Display for Inquire prompts.
struct InquireSchema<'a>(&'a EntitySchema);
impl<'a> std::fmt::Display for InquireSchema<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.entity_type)
    }
}

/// Interactively add a new entity and generate DSL for it.
pub fn add_entity(
    workspace_path: &PathBuf,
    to_file: Option<PathBuf>,
    output_format: OutputFormat,
) -> Result<(), CliError> {
    ui::header("Adding new entity");
    let mut workspace = Workspace::new();
    load_workspace_files(&workspace_path, &mut workspace).map_err(|_| CliError::BuildError)?;
    let build = build_workspace(workspace).map_err(|_| CliError::BuildError)?;
    let graph = build_graph(&build)?;

    // Let user choose entity type from built-in and custom schemas
    let mut sorted_schemas = build.schemas.clone();
    sorted_schemas.sort_by_key(|schema| schema.entity_type.to_string());
    let schema_options: Vec<_> = sorted_schemas.iter().map(InquireSchema).collect();
    let chosen_option = Select::new("Type:", schema_options)
        .prompt()
        .map_err(|_| CliError::InputError)?;

    let chosen_schema = chosen_option.0.clone();
    let chosen_type_str = format!("{}", &chosen_schema.entity_type);
    let chosen_id = Text::new("ID:")
        .prompt()
        .map_err(|_| CliError::InputError)?;

    // Make a unique ID for the entity based on the name
    let entity_id = compute_unique_entity_id(&graph, &chosen_type_str, chosen_id);

    // Create initial entity and collect required fields
    let mut entity = Entity::new(entity_id.into(), chosen_schema.entity_type.to_owned());
    let arc_graph = Arc::new(graph.clone());
    let generated_file_path = compute_dsl_path(workspace_path, to_file, chosen_type_str);
    entity = prompt_required_fields(
        &chosen_schema,
        entity.clone(),
        &arc_graph,
        &generated_file_path,
        workspace_path,
    )?;

    // If user chooses to add optionals, prompt for each optional field
    let add_optional = Confirm::new("Add optional fields?")
        .with_default(false)
        .prompt()
        .map_err(|_| CliError::InputError)?;

    if add_optional {
        entity = prompt_optional_fields(
            chosen_schema.clone(),
            entity.clone(),
            arc_graph,
            &generated_file_path,
            workspace_path,
        )?;
    }

    // Generate and write the resulting DSL
    let generated_dsl = generate_dsl(&[entity.clone()]);

    ui::info(&format!(
        "Writing generated DSL to file {}",
        generated_file_path.display()
    ));

    write_dsl(entity, generated_dsl, generated_file_path, output_format)
}

/// Prompts for each required field in an entity schema and writes it to the entity.
fn prompt_required_fields(
    chosen_schema: &EntitySchema,
    mut entity: Entity,
    arc_graph: &Arc<EntityGraph>,
    source_path: &PathBuf,
    workspace_path: &PathBuf,
) -> Result<Entity, CliError> {
    let mut required_fields: Vec<_> = chosen_schema
        .fields
        .iter()
        .filter(|(_, f)| f.is_required())
        .collect();

    required_fields.sort_by_key(|(field_id, _)| field_id.as_str());
    for (field_id, field) in required_fields {
        match prompt_for_field_value(
            field_id,
            field.expected_type(),
            field.is_required(),
            field.allowed_values(),
            Arc::clone(arc_graph),
            source_path,
            workspace_path,
        )? {
            Some(value) => {
                entity = entity.with_field(field_id.clone(), value);
            }
            None => {}
        }
    }

    Ok(entity)
}

/// Prompts for each optional field in an entity schema and writes it to the entity.
fn prompt_optional_fields(
    chosen_schema: EntitySchema,
    mut entity: Entity,
    graph: Arc<EntityGraph>,
    source_path: &PathBuf,
    workspace_path: &PathBuf,
) -> Result<Entity, CliError> {
    let mut optional_fields: Vec<_> = chosen_schema
        .fields
        .iter()
        .filter(|(_, f)| !f.is_required())
        .collect();

    optional_fields.sort_by_key(|(field_id, _)| field_id.as_str());
    for (field_id, field) in optional_fields {
        match prompt_for_field_value(
            field_id,
            field.expected_type(),
            field.is_required(),
            field.allowed_values(),
            Arc::clone(&graph),
            source_path,
            workspace_path,
        )? {
            Some(value) => {
                entity = entity.with_field(field_id.clone(), value);
            }
            None => {}
        }
    }

    Ok(entity)
}

/// Sanitize a string to be a valid entity ID.
/// - Filters for only alphabetic characters and whitespace
/// - Converts to snake_case
pub fn sanitize_entity_id(input: String) -> String {
    input
        .chars()
        .filter(|&c| c == ' ' || c.is_alphabetic())
        .collect::<String>()
        .to_case(Case::Snake)
}

/// Ensures uniqueness and comformity of a selected entity ID.
/// We do this by:
/// - Filtering for only alphabetic characters and whitespace
/// - Convert ID to snake_case
/// - Add a number at the end if ID is not unique
/// - Keep increasing the number (within reason) until it's unique
fn compute_unique_entity_id(
    graph: &EntityGraph,
    chosen_type_str: &String,
    chosen_id: String,
) -> String {
    let sanitized_id = sanitize_entity_id(chosen_id);

    let mut entity_id = sanitized_id.clone();
    let mut id_counter = 1;
    while graph
        .get_entity(&compose_entity_id(chosen_type_str, &entity_id))
        .is_some()
        && id_counter < 1000
    {
        entity_id = format!("{}_{}", sanitized_id, id_counter);
        id_counter += 1;
    }

    entity_id
}

/// Get the target path to write DSL to by:
/// - Using a custom path, if provided
/// - Generating a path from default settings
fn compute_dsl_path(
    workspace_path: &PathBuf,
    to_file: Option<PathBuf>,
    chosen_type_str: String,
) -> PathBuf {
    let dsl_path = match to_file {
        Some(file_path) => workspace_path
            .join(file_path)
            .with_extension(FIRM_EXTENSION),
        None => workspace_path
            .join(GENERATED_DIR_NAME)
            .join(&chosen_type_str)
            .with_extension(FIRM_EXTENSION),
    };

    dsl_path
}

/// Writes the DSL to a file and outputs the generated entity.
fn write_dsl(
    entity: Entity,
    generated_dsl: String,
    target_path: PathBuf,
    output_format: OutputFormat,
) -> Result<(), CliError> {
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent).map_err(|_| CliError::FileError)?;
    }

    match File::options().create(true).append(true).open(target_path) {
        Ok(mut file) => match file.write_all(&generated_dsl.into_bytes()) {
            Ok(_) => {
                ui::success(&format!("Generated DSL for '{}'", &entity.id));

                match output_format {
                    OutputFormat::Pretty => ui::pretty_output_entity_single(&entity),
                    OutputFormat::Json => ui::json_output(&entity),
                }
                Ok(())
            }
            Err(e) => {
                ui::error_with_details("Couldn't write to file", &e.to_string());
                Err(CliError::FileError)
            }
        },
        Err(e) => {
            ui::error_with_details("Couldn't open file", &e.to_string());
            Err(CliError::FileError)
        }
    }
}
