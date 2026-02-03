use chrono::{FixedOffset, Local, NaiveTime, TimeZone, Timelike};
use console::style;
use convert_case::{Case, Casing};
use firm_core::{
    FieldId, FieldType, FieldValue, ReferenceValue, compose_entity_id, graph::EntityGraph,
};
use inquire::{Confirm, CustomType, DateSelect, Select, Text, validator::Validation};
use iso_currency::{Currency, IntoEnumIterator};
use pathdiff::diff_paths;
use rust_decimal::Decimal;
use std::{
    error::Error,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::errors::CliError;

pub const SKIP_PROMPT_FRAGMENT: &str = " (esc to skip)";

/// Interactive prompt for a field value, applying relevant prompt configurations depending on the field type.
pub fn prompt_for_field_value(
    field_id: &FieldId,
    field_type: &FieldType,
    is_required: bool,
    allowed_values: Option<&Vec<String>>,
    entity_graph: Arc<EntityGraph>,
    source_path: &PathBuf,
    workspace_dir: &PathBuf,
) -> Result<Option<FieldValue>, CliError> {
    let skippable = !is_required;
    let field_id_prompt = field_id.as_str().to_case(Case::Sentence);

    match field_type {
        FieldType::Boolean => bool_prompt(skippable, &field_id_prompt),
        FieldType::String => string_prompt(skippable, &field_id_prompt),
        FieldType::Integer => int_prompt(skippable, &field_id_prompt),
        FieldType::Float => float_prompt(skippable, &field_id_prompt),
        FieldType::Currency => currency_prompt(skippable, &field_id_prompt),
        FieldType::Reference => {
            reference_prompt(skippable, &field_id_prompt, Arc::clone(&entity_graph))
        }
        FieldType::List => list_prompt(
            skippable,
            &field_id_prompt,
            Arc::clone(&entity_graph),
            source_path,
            workspace_dir,
        ),
        FieldType::DateTime => date_prompt(skippable, &field_id_prompt),
        FieldType::Path => path_prompt(
            skippable,
            &field_id_prompt,
            source_path,
            workspace_dir.clone(),
        ),
        FieldType::Enum => enum_prompt(skippable, &field_id_prompt, allowed_values),
    }
}

/// Prompts for a boolean field.
/// Value must be true or false.
fn bool_prompt(skippable: bool, field_id_prompt: &String) -> Result<Option<FieldValue>, CliError> {
    let skip_message = get_skippable_prompt(skippable);

    if skippable {
        let value = Confirm::new(&format!("{}{}:", field_id_prompt, skip_message))
            .prompt_skippable()
            .map_err(|_| CliError::InputError)?;
        Ok(value.map(FieldValue::Boolean))
    } else {
        let value = Confirm::new(&format!("{}{}:", field_id_prompt, skip_message))
            .prompt()
            .map_err(|_| CliError::InputError)?;
        Ok(Some(FieldValue::Boolean(value)))
    }
}

/// Prompts for a string field (only single-line supported).
/// String must not be empty.
fn string_prompt(
    skippable: bool,
    field_id_prompt: &String,
) -> Result<Option<FieldValue>, CliError> {
    let skip_message = get_skippable_prompt(skippable);
    let prompt_text = format!("{}{}:", field_id_prompt, skip_message);

    loop {
        let result = if skippable {
            Text::new(&prompt_text)
                .prompt_skippable()
                .map_err(|_| CliError::InputError)?
        } else {
            Some(
                Text::new(&prompt_text)
                    .prompt()
                    .map_err(|_| CliError::InputError)?,
            )
        };

        match result {
            Some(v) => {
                if !v.trim().is_empty() {
                    return Ok(Some(FieldValue::String(v)));
                } else {
                    eprintln!(
                        "{}",
                        style("This field cannot be empty. Please enter a value.").red()
                    );
                }
            }
            None => {
                // This branch is only reachable if skippable is true and skip was requested.
                if skippable {
                    return Ok(None);
                } else {
                    unreachable!("Text::prompt() for a non-skippable field should not return None");
                }
            }
        }
    }
}

/// Prompts for an integer field.
/// Value must not have a decimal place.
fn int_prompt(skippable: bool, field_id_prompt: &String) -> Result<Option<FieldValue>, CliError> {
    let skip_message = get_skippable_prompt(skippable);
    let prompt_text = format!("{}{}:", field_id_prompt, skip_message);

    let value = CustomType::<i64>::new(&prompt_text)
        .with_error_message("Enter a valid integer")
        .with_help_message("Enter a whole number");

    if skippable {
        let result = value.prompt_skippable().map_err(|_| CliError::InputError)?;
        Ok(result.map(FieldValue::Integer))
    } else {
        let result = value.prompt().map_err(|_| CliError::InputError)?;
        Ok(Some(FieldValue::Integer(result)))
    }
}

/// Prompts for a float field.
/// Value must have a decimal place.
fn float_prompt(skippable: bool, field_id_prompt: &String) -> Result<Option<FieldValue>, CliError> {
    let skip_message = get_skippable_prompt(skippable);
    let prompt_text = format!("{}{}:", field_id_prompt, skip_message);

    let value = CustomType::<f64>::new(&prompt_text)
        .with_error_message("Enter a valid decimal number")
        .with_help_message("Enter a decimal number (e.g., 3.14)");

    if skippable {
        let result = value.prompt_skippable().map_err(|_| CliError::InputError)?;
        Ok(result.map(FieldValue::Float))
    } else {
        let result = value.prompt().map_err(|_| CliError::InputError)?;
        Ok(Some(FieldValue::Float(result)))
    }
}

/// Wraps currency for use in Inquire custom prompt.
struct CurrencyOption {
    currency: Currency,
}

impl std::fmt::Display for CurrencyOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.currency.code(), self.currency.name())
    }
}

/// Prompts for a currency field.
/// Currency amount must be a valid number. Currency code is selected from a list of valid options.
fn currency_prompt(
    skippable: bool,
    field_id_prompt: &String,
) -> Result<Option<FieldValue>, CliError> {
    let skip_message = get_skippable_prompt(skippable);
    let amount_prompt = format!("Amount for {}{}:", field_id_prompt, skip_message);

    // Get the amount
    let amount = CustomType::<Decimal>::new(&amount_prompt)
        .with_error_message("Enter a valid decimal amount (e.g., 123.45)")
        .with_help_message("Enter the monetary amount as a decimal number")
        .with_parser(&|input| Decimal::from_str_exact(input).map_err(|_| ()));

    let amount_value = if skippable {
        let result = amount
            .prompt_skippable()
            .map_err(|_| CliError::InputError)?;

        match result {
            Some(val) => val,
            None => return Ok(None),
        }
    } else {
        amount.prompt().map_err(|_| CliError::InputError)?
    };

    // Get the currency code
    let currencies: Vec<CurrencyOption> = Currency::iter()
        .map(|currency| CurrencyOption { currency })
        .collect();

    let currency_prompt = format!("Currency for {}:", field_id_prompt);
    let selected_option = Select::new(&currency_prompt, currencies)
        .with_help_message("Select the currency")
        .prompt()
        .map_err(|_| CliError::InputError)?;

    Ok(Some(FieldValue::Currency {
        amount: amount_value,
        currency: selected_option.currency,
    }))
}

/// Prompt for a reference field.
/// Reference must be to an existing entity or field in the graph.
/// Auto-complete is provided based on entities in the current graph.
fn reference_prompt(
    skippable: bool,
    field_id_prompt: &String,
    entity_graph: Arc<EntityGraph>,
) -> Result<Option<FieldValue>, CliError> {
    let skip_message = get_skippable_prompt(skippable);
    let prompt_text = format!("{}{}:", field_id_prompt, skip_message);

    let graph_for_validator = Arc::clone(&entity_graph);
    let validator = move |input: &str| parse_reference(input, &graph_for_validator);
    let graph_for_autocomplete = Arc::clone(&entity_graph);
    let autocomplete = move |input: &str| get_reference_suggestions(input, &graph_for_autocomplete);
    let reference_value_prompt = Text::new(&prompt_text)
        .with_help_message("Start typing the reference for autocompletion")
        .with_validator(validator)
        .with_autocomplete(autocomplete);

    let result_str = if skippable {
        let result = reference_value_prompt
            .prompt_skippable()
            .map_err(|_| CliError::InputError)?;

        match result {
            Some(val) => val,
            None => return Ok(None),
        }
    } else {
        reference_value_prompt
            .prompt()
            .map_err(|_| CliError::InputError)?
    };

    let parts: Vec<&str> = result_str.split('.').collect();
    match parts.len() {
        2 => Ok(Some(FieldValue::Reference(ReferenceValue::Entity(
            compose_entity_id(parts[0], parts[1]),
        )))),
        3 => Ok(Some(FieldValue::Reference(ReferenceValue::Field(
            compose_entity_id(parts[0], parts[1]),
            FieldId(parts[2].into()),
        )))),
        _ => unreachable!("Parser should have prevented this format."),
    }
}

/// Parses a string reference by decomposing it and checking the graph if it exists.
fn parse_reference(
    input: &str,
    graph: &EntityGraph,
) -> Result<Validation, Box<dyn Error + Send + Sync>> {
    let parts: Vec<&str> = input.split(".").collect();
    match parts.len() {
        2 => {
            let entity_type = parts[0];
            let entity_id = parts[1];
            let composite_id = compose_entity_id(entity_type, entity_id);
            match graph.get_entity(&composite_id) {
                Some(_) => Ok(Validation::Valid),
                None => Ok(Validation::Invalid(
                    "There is no entity matching this ID".into(),
                )),
            }
        }
        3 => {
            let entity_type = parts[0];
            let entity_id = parts[1];
            let composite_id = compose_entity_id(entity_type, entity_id);
            match graph.get_entity(&composite_id) {
                Some(entity) => {
                    let field_id = parts[2];
                    match entity.get_field(&field_id.into()) {
                        Some(_) => Ok(Validation::Valid),
                        None => Ok(Validation::Invalid(
                            "There is no field matching this ID".into(),
                        )),
                    }
                }
                None => Ok(Validation::Invalid(
                    "There is no entity matching this ID".into(),
                )),
            }
        }
        _ => Ok(Validation::Invalid(
            "References should have 2 or 3 parts separated by '.'".into(),
        )),
    }
}

/// Gets suggestions for the reference prompt by searching the graph for partial matches.
fn get_reference_suggestions(
    input: &str,
    graph: &EntityGraph,
) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
    let parts: Vec<&str> = input.split('.').collect();
    let mut suggestions = Vec::new();

    match parts.len() {
        1 => {
            // Suggesting entity types
            let partial_type = parts[0];
            for entity_type in graph.get_all_entity_types() {
                if entity_type.to_string().starts_with(partial_type) {
                    suggestions.push(format!("{}.", entity_type));
                }
            }
        }
        2 => {
            // Suggesting entity IDs
            let entity_type = parts[0];
            let entity_id = parts[1];
            let composite_id = compose_entity_id(entity_type, entity_id);
            let entities = graph.list_by_type(&entity_type.into());
            for entity in entities {
                if entity.id.as_str().starts_with(composite_id.as_str()) {
                    suggestions.push(entity.id.to_string());
                }
            }
        }
        3 => {
            // Suggesting field IDs
            let entity_type = parts[0];
            let entity_id = parts[1];
            let partial_field = parts[2];
            let composite_id = compose_entity_id(entity_type, entity_id);

            if let Some(entity) = graph.get_entity(&composite_id) {
                for (field_id, _) in &entity.fields {
                    if field_id.as_str().starts_with(partial_field) {
                        suggestions.push(format!("{}.{}", entity.id, field_id.as_str()));
                    }
                }
            }
        }
        _ => {
            // No suggestions for invalid formats
        }
    }

    Ok(suggestions)
}

/// Prompt for a list field.
/// Lists must have homogeneous types.
/// User can select a valid type, then iteratively inputs values to it.
fn list_prompt(
    skippable: bool,
    field_id_prompt: &String,
    entity_graph: Arc<EntityGraph>,
    source_path: &PathBuf,
    workspace_dir: &PathBuf,
) -> Result<Option<FieldValue>, CliError> {
    // Ask for the item type
    let item_types = vec![
        FieldType::String,
        FieldType::Integer,
        FieldType::Float,
        FieldType::Boolean,
        FieldType::DateTime,
        FieldType::Currency,
    ];

    let item_type_prompt_text = format!(
        "Type for list {}{}",
        field_id_prompt,
        get_skippable_prompt(skippable)
    );

    let item_type = if skippable {
        let result = Select::new(&item_type_prompt_text, item_types)
            .with_formatter(&|field_type| format!("{}", field_type))
            .prompt_skippable()
            .map_err(|_| CliError::InputError)?;
        match result {
            Some(t) => t,
            None => return Ok(None),
        }
    } else {
        Select::new(&item_type_prompt_text, item_types)
            .with_formatter(&|field_type| format!("{}", field_type))
            .prompt()
            .map_err(|_| CliError::InputError)?
    };

    // Collect items until user skips
    let mut items = Vec::new();
    let mut item_index = 1;
    loop {
        // Prompt for each item (always treat as skippable so user can skip to finish)
        let item_field_id = FieldId::new(format!("item_{}", item_index));
        match prompt_for_field_value(
            &item_field_id,
            &item_type,
            false,
            None,
            Arc::clone(&entity_graph),
            source_path,
            workspace_dir,
        )? {
            Some(value) => {
                items.push(value);
                item_index += 1;
            }
            None => {
                // User skipped, finish the list
                break;
            }
        }
    }

    Ok(Some(FieldValue::List(items)))
}

/// Prompts for a date field.
/// We do in 3 steps, first a calendar, then time, then UTC offset.
fn date_prompt(skippable: bool, field_id_prompt: &String) -> Result<Option<FieldValue>, CliError> {
    let skip_message = get_skippable_prompt(skippable);

    // Get the date
    let date = if skippable {
        match DateSelect::new(&format!("{}{}:", field_id_prompt, skip_message))
            .with_help_message("Use arrow keys to navigate, Enter to select")
            .prompt_skippable()
            .map_err(|_| CliError::InputError)?
        {
            Some(d) => d,
            None => return Ok(None),
        }
    } else {
        DateSelect::new(&format!("{}{}:", field_id_prompt, skip_message))
            .with_help_message("Use arrow keys to navigate, Enter to select")
            .prompt()
            .map_err(|_| CliError::InputError)?
    };

    // Get the time (HH:MM only)
    let time_input = if skippable {
        match CustomType::<NaiveTime>::new("at (esc to skip):")
            .with_error_message("Enter time in HH:MM format (e.g., 14:30)")
            .with_help_message("Format: HH:MM (24-hour format)")
            .with_parser(&|input| {
                NaiveTime::parse_from_str(input, "%H:%M")
                    .map(|t| t.with_second(0).unwrap())
                    .map_err(|_| ())
            })
            .with_default(NaiveTime::from_hms_opt(12, 0, 0).unwrap())
            .prompt_skippable()
            .map_err(|_| CliError::InputError)?
        {
            Some(t) => t,
            None => return Ok(None),
        }
    } else {
        CustomType::<NaiveTime>::new("at:")
            .with_error_message("Enter time in HH:MM format (e.g., 14:30)")
            .with_help_message("Format: HH:MM (24-hour format)")
            .with_parser(&|input| {
                NaiveTime::parse_from_str(input, "%H:%M")
                    .map(|t| t.with_second(0).unwrap())
                    .map_err(|_| ())
            })
            .with_default(NaiveTime::from_hms_opt(12, 0, 0).unwrap())
            .prompt()
            .map_err(|_| CliError::InputError)?
    };

    let naive_datetime = date.and_time(time_input);

    // Get the local timezone offset in hours
    let local_offset_seconds = Local::now().offset().local_minus_utc();
    let local_offset_hours = local_offset_seconds / 3600;

    // Get timezone offset as integer hours
    let timezone_offset = if skippable {
        match CustomType::<i32>::new("UTC offset (esc to skip):")
            .with_error_message("Enter a valid integer between -12 and +14")
            .with_help_message(&format!("Enter hours offset from UTC (e.g., 2 for +02:00, -5 for -05:00), default is {} (local timezone)", local_offset_hours))
            .with_default(local_offset_hours)
            .prompt_skippable()
            .map_err(|_| CliError::InputError)?
        {
            Some(o) => o,
            None => return Ok(None),
        }
    } else {
        CustomType::<i32>::new("UTC offset:")
            .with_error_message("Enter a valid integer between -12 and +14")
            .with_help_message(&format!("Enter hours offset from UTC (e.g., 2 for +02:00, -5 for -05:00), default is {} (local timezone)", local_offset_hours))
            .with_default(local_offset_hours)
            .prompt()
            .map_err(|_| CliError::InputError)?
    };

    // Validate offset range
    if !(-12..=14).contains(&timezone_offset) {
        return Err(CliError::InputError);
    }

    let offset = FixedOffset::east_opt(timezone_offset * 3600).unwrap();
    let datetime = offset.from_local_datetime(&naive_datetime).unwrap();

    Ok(Some(FieldValue::DateTime(datetime)))
}

/// Prompts for a path field.
fn path_prompt(
    skippable: bool,
    field_id_prompt: &String,
    source_path: &PathBuf,
    workspace_dir: PathBuf,
) -> Result<Option<FieldValue>, CliError> {
    let skip_message = get_skippable_prompt(skippable);
    let prompt_text = format!("{}{}:", field_id_prompt, skip_message);

    let autocomplete_workspace = workspace_dir.clone();
    let autocomplete =
        move |input: &str| get_path_suggestions(input, autocomplete_workspace.clone());
    let reference_value_prompt = Text::new(&prompt_text)
        .with_help_message("Start typing the path for autocompletion")
        .with_autocomplete(autocomplete);

    let result_str = if skippable {
        let result = reference_value_prompt
            .prompt_skippable()
            .map_err(|_| CliError::InputError)?;

        match result {
            Some(val) => val,
            None => return Ok(None),
        }
    } else {
        reference_value_prompt
            .prompt()
            .map_err(|_| CliError::InputError)?
    };

    // Transform the workspace-relative path to the source-file relative path
    let full_path = workspace_dir.join(&result_str);
    let source_dir = source_path.parent().unwrap_or(Path::new(""));
    let relative_path =
        diff_paths(&full_path, source_dir).unwrap_or_else(|| PathBuf::from(&result_str));

    Ok(Some(FieldValue::Path(relative_path)))
}

fn get_path_suggestions(
    input: &str,
    workspace_dir: PathBuf,
) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
    let input_path = Path::new(input);
    let search_dir = if input.ends_with('/') || input.is_empty() {
        workspace_dir.join(input_path)
    } else {
        workspace_dir
            .join(input_path)
            .parent()
            .unwrap_or(&workspace_dir)
            .to_path_buf()
    };

    let mut suggestions = Vec::new();
    if let Ok(entries) = std::fs::read_dir(search_dir) {
        for entry in entries.flatten() {
            let full_path = entry.path();

            // Get the path relative to the current workspace directory
            if let Some(relative_path) = diff_paths(&full_path, &workspace_dir) {
                let mut suggestion = relative_path.to_string_lossy().to_string();

                // Add a trailing slash to directories for clairty
                if let Ok(file_type) = entry.file_type()
                    && file_type.is_dir() {
                        suggestion.push('/');
                    }

                // Add the suggestion if it starts with the user's input
                if suggestion.starts_with(input) {
                    suggestions.push(suggestion);
                }
            }
        }
    }

    Ok(suggestions)
}

/// Prompts for an enum field.
fn enum_prompt(
    skippable: bool,
    field_id_prompt: &String,
    allowed_values: Option<&Vec<String>>,
) -> Result<Option<FieldValue>, CliError> {
    match allowed_values {
        Some(values) => {
            let skip_message = get_skippable_prompt(skippable);
            let prompt_text = format!("{}{}:", field_id_prompt, skip_message);

            let selected_option = Select::new(&prompt_text, values.to_vec())
                .prompt()
                .map_err(|_| CliError::InputError)?;

            Ok(Some(FieldValue::Enum(selected_option)))
        }
        None => Err(CliError::InputError),
    }
}

/// Helper to get a prompt message fragment or empty string depending on whether field is skippable.
fn get_skippable_prompt(skippable: bool) -> &'static str {
    if skippable { SKIP_PROMPT_FRAGMENT } else { "" }
}
