use convert_case::{Case, Casing};
use firm_core::graph::EntityGraph;
use firm_core::{Entity, EntitySchema, FieldId, FieldType, FieldValue, compose_entity_id};
use firm_lang::generate::generate_dsl;
use firm_lang::parser::ParsedValue;
use firm_lang::workspace::Workspace;
use inquire::{Confirm, Select, Text};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

use super::{
    build_workspace, field_prompt::prompt_for_field_value, load_workspace_files,
};
use crate::errors::CliError;
use crate::files::load_current_graph;
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

/// Add a new entity and generate DSL for it.
/// If type, id, or fields are provided, uses non-interactive mode.
pub fn add_entity(
    workspace_path: &PathBuf,
    to_file: Option<PathBuf>,
    entity_type: Option<String>,
    entity_id: Option<String>,
    fields: Vec<String>,
    lists: Vec<String>,
    list_values: Vec<String>,
    output_format: OutputFormat,
) -> Result<(), CliError> {
    // Check if we're in non-interactive mode
    let is_non_interactive = entity_type.is_some() || entity_id.is_some() || !fields.is_empty() || !lists.is_empty() || !list_values.is_empty();

    if is_non_interactive {
        // Validate that both type and id are provided
        if entity_type.is_none() || entity_id.is_none() {
            ui::error("Non-interactive mode requires both --type and --id arguments");
            return Err(CliError::InputError);
        }

        return add_entity_non_interactive(
            workspace_path,
            to_file,
            entity_type.unwrap(),
            entity_id.unwrap(),
            fields,
            lists,
            list_values,
            output_format,
        );
    }

    // Otherwise, use interactive mode
    add_entity_interactive(workspace_path, to_file, output_format)
}

/// Add a new entity non-interactively using CLI arguments.
fn add_entity_non_interactive(
    workspace_path: &PathBuf,
    to_file: Option<PathBuf>,
    entity_type: String,
    entity_id: String,
    fields: Vec<String>,
    lists: Vec<String>,
    list_values: Vec<String>,
    output_format: OutputFormat,
) -> Result<(), CliError> {
    // Load the pre-built graph and build workspace for schemas
    let graph = load_current_graph(&workspace_path)?;
    let mut workspace = Workspace::new();
    load_workspace_files(&workspace_path, &mut workspace).map_err(|_| CliError::BuildError)?;
    let build = build_workspace(workspace).map_err(|_| CliError::BuildError)?;

    // Find the schema for the given type
    let schema = build
        .schemas
        .iter()
        .find(|s| s.entity_type.to_string() == entity_type)
        .ok_or_else(|| {
            ui::error(&format!("Schema for '{}' not found in workspace", entity_type));
            CliError::InputError
        })?;

    // Check if the entity ID is unique
    let sanitized_id = sanitize_entity_id(entity_id.clone());
    let composite_id = compose_entity_id(&entity_type, &sanitized_id);
    if graph.get_entity(&composite_id).is_some() {
        ui::error(&format!(
            "An entity with ID '{}' already exists",
            composite_id
        ));
        return Err(CliError::InputError);
    }

    // Parse fields from CLI args
    let mut entity = Entity::new(composite_id.clone(), schema.entity_type.to_owned());

    // Parse list declarations (--list field_name item_type)
    let mut list_types: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for chunk in lists.chunks(2) {
        if chunk.len() == 2 {
            list_types.insert(chunk[0].to_string(), chunk[1].to_string());
        }
    }

    // Group list values by field name (--list-value field_name value)
    let mut list_value_groups: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    for chunk in list_values.chunks(2) {
        if chunk.len() == 2 {
            list_value_groups.entry(chunk[0].to_string()).or_insert_with(Vec::new).push(chunk[1].to_string());
        }
    }

    // Compute the generated file path early so we can use it for path parsing
    let generated_file_path = compute_dsl_path(workspace_path, to_file.clone(), entity_type.clone());

    // Process regular fields (--field field_name value)
    for chunk in fields.chunks(2) {
        if chunk.len() == 2 {
            let field_name = chunk[0].as_str();
            let field_value_str = chunk[1].as_str();
            let field_id = FieldId::new(field_name);

            // Find the field in the schema to get its expected type
            let schema_field = schema.fields.get(&field_id).ok_or_else(|| {
                ui::error(&format!("Field '{}' is not defined in schema '{}'", field_name, entity_type));
                ui::error("\nAvailable fields in this schema:");
                for (field_id, field_def) in &schema.fields {
                    let required_str = if field_def.is_required() { "required" } else { "optional" };
                    ui::error(&format!("  - {} ({}, {})", field_id.as_str(), field_def.expected_type(), required_str));
                }
                CliError::InputError
            })?;

            let expected_type = schema_field.expected_type();

            // Parse the field value
            let parsed_value = parse_field_value_from_string(field_value_str, expected_type, &generated_file_path)?;
            let field_value: FieldValue = parsed_value.try_into().map_err(|_| {
                ui::error(&format!("Failed to convert parsed value for field '{}'", field_name));
                CliError::InputError
            })?;

            entity = entity.with_field(field_id, field_value);
        }
    }

    // Process list fields (--list field_name item_type and --list-value field_name value)
    for (list_field_name, item_type_str) in &list_types {
        let field_id = FieldId::new(list_field_name);

        // Validate field exists in schema
        let schema_field = schema.fields.get(&field_id).ok_or_else(|| {
            ui::error(&format!("List field '{}' is not defined in schema '{}'", list_field_name, entity_type));
            ui::error("\nAvailable fields in this schema:");
            for (field_id, field_def) in &schema.fields {
                let required_str = if field_def.is_required() { "required" } else { "optional" };
                ui::error(&format!("  - {} ({}, {})", field_id.as_str(), field_def.expected_type(), required_str));
            }
            CliError::InputError
        })?;

        // Verify this field is actually a list type in the schema
        if !matches!(schema_field.expected_type(), FieldType::List) {
            ui::error(&format!("Field '{}' is not a list type in the schema", list_field_name));
            return Err(CliError::InputError);
        }

        // Parse the item type
        let item_field_type = parse_field_type(item_type_str)?;

        // Get the values for this list
        let values = list_value_groups.get(list_field_name).ok_or_else(|| {
            ui::error(&format!("No values provided for list '{}' (use --list-value)", list_field_name));
            CliError::InputError
        })?;

        // Parse each value using the declared item type
        let mut parsed_items = Vec::new();
        for value_str in values {
            let item = parse_field_value_from_string(value_str.as_str(), &item_field_type, &generated_file_path)?;
            parsed_items.push(item);
        }

        // Validate homogeneity and create list
        let parsed_list = ParsedValue::parse_list_from_vec(parsed_items).map_err(|e| {
            ui::error(&format!("Failed to create list for field '{}': {}", list_field_name, e));
            CliError::InputError
        })?;

        let field_value: FieldValue = parsed_list.try_into().map_err(|_| {
            ui::error(&format!("Failed to convert list for field '{}'", list_field_name));
            CliError::InputError
        })?;

        entity = entity.with_field(field_id, field_value);
    }

    // Validate entity against schema
    schema.validate(&entity).map_err(|errors| {
        ui::error("Entity validation failed:");
        for error in errors {
            ui::error(&format!("  - {}", error.message));
        }
        CliError::InputError
    })?;

    // Generate and write DSL
    let generated_dsl = generate_dsl(&[entity.clone()]);

    ui::info(&format!(
        "Writing generated DSL to file {}",
        generated_file_path.display()
    ));

    write_dsl(entity, generated_dsl, generated_file_path, output_format)
}

/// Interactively add a new entity and generate DSL for it.
fn add_entity_interactive(
    workspace_path: &PathBuf,
    to_file: Option<PathBuf>,
    output_format: OutputFormat,
) -> Result<(), CliError> {
    ui::header("Adding new entity");
    let graph = load_current_graph(&workspace_path)?;
    let mut workspace = Workspace::new();
    load_workspace_files(&workspace_path, &mut workspace).map_err(|_| CliError::BuildError)?;
    let build = build_workspace(workspace).map_err(|_| CliError::BuildError)?;

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
/// - Filters for only alphanumeric characters, underscores, dashes, and whitespace
/// - Converts to snake_case
pub fn sanitize_entity_id(input: String) -> String {
    input
        .chars()
        .filter(|&c| c == ' ' || c == '_' || c == '-' || c.is_alphanumeric())
        .collect::<String>()
        .to_case(Case::Snake)
}

/// Ensures uniqueness and conformity of a selected entity ID.
/// We do this by:
/// - Filtering for only alphanumeric characters, underscores, dashes, and whitespace
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

/// Parses a field type string into a FieldType enum.
fn parse_field_type(type_str: &str) -> Result<FieldType, CliError> {
    match type_str.to_lowercase().as_str() {
        "string" => Ok(FieldType::String),
        "integer" => Ok(FieldType::Integer),
        "float" => Ok(FieldType::Float),
        "boolean" => Ok(FieldType::Boolean),
        "currency" => Ok(FieldType::Currency),
        "reference" => Ok(FieldType::Reference),
        "datetime" => Ok(FieldType::DateTime),
        "path" => Ok(FieldType::Path),
        "enum" => Ok(FieldType::Enum),
        _ => {
            ui::error(&format!("Unknown field type '{}'. Valid types: string, integer, float, boolean, currency, reference, datetime, path, enum", type_str));
            Err(CliError::InputError)
        }
    }
}

/// Parses a field value from a string based on the expected type.
fn parse_field_value_from_string(
    value_str: &str,
    expected_type: &FieldType,
    source_path: &PathBuf,
) -> Result<ParsedValue, CliError> {
    match expected_type {
        FieldType::Boolean => ParsedValue::parse_boolean(value_str),
        FieldType::String => ParsedValue::parse_string(value_str),
        FieldType::Integer | FieldType::Float => ParsedValue::parse_number(value_str),
        FieldType::Currency => ParsedValue::parse_currency(value_str),
        FieldType::Reference => ParsedValue::parse_reference(value_str),
        FieldType::DateTime => {
            // Try parsing as datetime first, then as date
            ParsedValue::parse_datetime(value_str)
                .or_else(|_| ParsedValue::parse_date(value_str))
        }
        FieldType::Enum => ParsedValue::parse_enum(value_str),
        FieldType::Path => {
            // For paths in non-interactive mode, the user specifies them relative to CWD
            // But we need to store them relative to the generated .firm file
            // So we need to transform: CWD-relative -> absolute -> source-file-relative
            let user_path = PathBuf::from(value_str);
            let absolute_path = if user_path.is_absolute() {
                user_path
            } else {
                // Resolve relative to current working directory
                std::env::current_dir()
                    .map_err(|_| {
                        ui::error("Failed to get current working directory");
                        CliError::InputError
                    })?
                    .join(&user_path)
            };

            // Now make it relative to the source file's parent directory
            // Canonicalize both paths to ensure diff_paths works correctly
            let canonical_target = absolute_path.canonicalize().unwrap_or(absolute_path.clone());
            let source_dir = source_path.parent().unwrap_or(std::path::Path::new(""));
            let canonical_source_dir = source_dir.canonicalize().unwrap_or(source_dir.to_path_buf());

            let relative_to_source = pathdiff::diff_paths(&canonical_target, &canonical_source_dir)
                .unwrap_or(canonical_target.clone());

            Ok(ParsedValue::Path(relative_to_source))
        }
        FieldType::List => {
            ui::error("List fields must be specified using --list and --list-value flags");
            return Err(CliError::InputError);
        }
    }
    .map_err(|e| {
        ui::error(&format!("Failed to parse field value: {}", e));
        CliError::InputError
    })
}
