//! Add entity tool implementation.

use serde::Deserialize;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use firm_core::graph::EntityGraph;
use firm_core::{
    Entity, EntityId, EntityType, FieldId, FieldType, FieldValue, ReferenceValue, compose_entity_id,
};
use firm_lang::generate::generate_dsl;
use firm_lang::workspace::WorkspaceBuild;
use rmcp::model::{CallToolResult, Content};
use rmcp::schemars;

/// Parameters for the add_entity tool.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AddEntityParams {
    /// Entity type (e.g., "person", "task").
    pub r#type: String,

    /// Entity ID (e.g., "john_doe", "fix_bug").
    /// Will be converted to snake_case automatically.
    pub id: String,

    /// Field values as a key-value map.
    /// Values must match the schema types (string, number, boolean, array, etc.).
    /// Paths should be relative to the workspace root.
    pub fields: HashMap<String, serde_json::Value>,

    /// Optional target file path relative to workspace root.
    /// If omitted, defaults to "generated/<type>.firm".
    /// The file will be created if it doesn't exist.
    pub to_file: Option<String>,

    /// Optional type annotations for list fields.
    /// Maps field names to their inner type (e.g., "secondary_contacts" -> "reference").
    /// Required for any field with type List in the schema.
    /// Valid types: string, integer, float, boolean, currency, reference, datetime, path, enum.
    pub list_item_types: Option<HashMap<String, String>>,
}

/// Result of adding an entity.
#[derive(Debug)]
pub struct AddEntityResult {
    /// The path where the entity was written (relative to workspace root).
    pub path: String,
    /// The generated DSL content.
    pub dsl: String,
    /// Whether the file was created (true) or appended to (false).
    pub created_new_file: bool,
}

/// Execute the add_entity tool.
///
/// Validates the entity against the schema, generates DSL, and writes it to a file.
/// Returns the file path and generated content.
pub fn execute(
    workspace_path: &Path,
    build: &WorkspaceBuild,
    graph: &EntityGraph,
    params: &AddEntityParams,
) -> Result<AddEntityResult, String> {
    let entity_type_str = params.r#type.as_str();
    let entity_id_str = params.id.as_str();

    // 1. Validate Schema Exists
    let schema = build
        .schemas
        .iter()
        .find(|s| s.entity_type.as_str() == entity_type_str)
        .ok_or_else(|| format!("Schema for type '{}' not found", entity_type_str))?;

    // 2. Check ID Uniqueness
    // EntityId::new automatically converts to snake_case
    let entity_id = EntityId::new(entity_id_str);
    let composite_id = compose_entity_id(entity_type_str, entity_id.as_str());

    if graph.get_entity(&composite_id).is_some() {
        return Err(format!("Entity with ID '{}' already exists", composite_id));
    }

    // 3. Determine Target Path
    let target_rel_path = match &params.to_file {
        Some(p) => PathBuf::from(p),
        None => PathBuf::from("generated")
            .join(entity_type_str)
            .with_extension("firm"),
    };

    let target_abs_path = workspace_path.join(&target_rel_path);

    // 4. Construct Entity
    let mut entity = Entity::new(composite_id, EntityType::new(entity_type_str));

    // Convert fields
    for (name, json_value) in &params.fields {
        let field_id = FieldId::new(name);

        // Find field definition in schema
        let field_def = schema.fields.get(&field_id).ok_or_else(|| {
            format!(
                "Field '{}' not found in schema for '{}'",
                name, entity_type_str
            )
        })?;

        let value = json_to_field_value(
            json_value,
            field_def.expected_type(),
            workspace_path,
            &target_abs_path,
            &params.list_item_types,
            name,
        )?;

        entity = entity.with_field(field_id, value);
    }

    // 5. Validate Entity against Schema
    schema.validate(&entity).map_err(|errors| {
        let msgs: Vec<String> = errors.into_iter().map(|e| e.message.clone()).collect();
        format!("Validation failed:\n- {}", msgs.join("\n- "))
    })?;

    // 6. Generate DSL
    let dsl = generate_dsl(&[entity]);

    // 7. Write to File
    if let Some(parent) = target_abs_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    let file_exists = target_abs_path.exists();

    // Read existing content to ensure we append with a newline if needed
    let mut prefix = String::new();
    if file_exists {
        let mut file =
            File::open(&target_abs_path).map_err(|e| format!("Failed to open file: {}", e))?;
        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        if !content.ends_with('\n') && !content.is_empty() {
            prefix.push('\n');
        }
    }

    let mut file = File::options()
        .create(true)
        .append(true)
        .open(&target_abs_path)
        .map_err(|e| format!("Failed to open file for writing: {}", e))?;

    let final_content = format!("{}{}", prefix, dsl);
    file.write_all(final_content.as_bytes())
        .map_err(|e| format!("Failed to write to file: {}", e))?;

    Ok(AddEntityResult {
        path: target_rel_path.to_string_lossy().into_owned(),
        dsl,
        created_new_file: !file_exists,
    })
}

/// Convert JSON value to FieldValue based on expected type.
fn json_to_field_value(
    value: &serde_json::Value,
    expected_type: &FieldType,
    workspace_path: &Path,
    target_file_path: &Path,
    list_item_types: &Option<HashMap<String, String>>,
    field_name: &str,
) -> Result<FieldValue, String> {
    match expected_type {
        FieldType::String => match value {
            serde_json::Value::String(s) => Ok(FieldValue::String(s.clone())),
            _ => Err(format!(
                "Expected string for field type String, got {:?}",
                value
            )),
        },
        FieldType::Integer => match value {
            serde_json::Value::Number(n) if n.is_i64() => {
                Ok(FieldValue::Integer(n.as_i64().unwrap()))
            }
            serde_json::Value::Number(n) => {
                Err(format!("Expected integer, got float/other: {}", n))
            }
            _ => Err(format!(
                "Expected number for field type Integer, got {:?}",
                value
            )),
        },
        FieldType::Float => match value {
            serde_json::Value::Number(n) => {
                let f = n
                    .as_f64()
                    .ok_or_else(|| format!("Invalid float value: {}", n))?;
                Ok(FieldValue::Float(f))
            }
            _ => Err(format!(
                "Expected number for field type Float, got {:?}",
                value
            )),
        },
        FieldType::Boolean => match value {
            serde_json::Value::Bool(b) => Ok(FieldValue::Boolean(*b)),
            _ => Err(format!(
                "Expected boolean for field type Boolean, got {:?}",
                value
            )),
        },
        FieldType::Reference => {
            match value {
                serde_json::Value::String(s) => {
                    // Check for dots to determine reference type
                    if s.contains('.') {
                        // Could be type.id.field or type.id
                        let parts: Vec<&str> = s.split('.').collect();
                        if parts.len() >= 3 {
                            // type.id.field
                            let entity_id = compose_entity_id(parts[0], parts[1]);
                            let field_id = FieldId::new(parts[2]);
                            Ok(FieldValue::Reference(ReferenceValue::Field(
                                entity_id, field_id,
                            )))
                        } else if parts.len() == 2 {
                            // type.id (Entity ref)
                            let entity_id = EntityId::new(s); // Already has dot
                            Ok(FieldValue::Reference(ReferenceValue::Entity(entity_id)))
                        } else {
                            // Should not happen with split('.') but fallback
                            Ok(FieldValue::Reference(ReferenceValue::Entity(
                                EntityId::new(s),
                            )))
                        }
                    } else {
                        // Simple ID, assume Entity ref
                        Ok(FieldValue::Reference(ReferenceValue::Entity(
                            EntityId::new(s),
                        )))
                    }
                }
                _ => Err(format!(
                    "Expected string for field type Reference, got {:?}",
                    value
                )),
            }
        }
        FieldType::List => match value {
            serde_json::Value::Array(arr) => {
                if arr.is_empty() {
                    return Ok(FieldValue::List(Vec::new()));
                }

                let item_type_str = list_item_types
                        .as_ref()
                        .and_then(|types| types.get(field_name))
                        .ok_or_else(|| {
                            format!(
                                "Field '{}' has type List. Specify the inner type in list_item_types (e.g., {{\"{}\": \"reference\"}})",
                                field_name, field_name
                            )
                        })?;

                let item_type = parse_list_item_type(item_type_str)?;

                let mut values = Vec::new();
                for item in arr {
                    let val = json_to_field_value(
                        item,
                        &item_type,
                        workspace_path,
                        target_file_path,
                        list_item_types,
                        field_name,
                    )?;
                    values.push(val);
                }
                Ok(FieldValue::List(values))
            }
            _ => Err(format!(
                "Expected array for field type List, got {:?}",
                value
            )),
        },
        FieldType::Enum => match value {
            serde_json::Value::String(s) => Ok(FieldValue::Enum(s.clone())),
            _ => Err(format!(
                "Expected string for field type Enum, got {:?}",
                value
            )),
        },
        FieldType::Path => {
            match value {
                serde_json::Value::String(s) => {
                    // Path is relative to workspace root
                    let path_from_root = PathBuf::from(s);
                    let abs_target = workspace_path.join(path_from_root);

                    // Calculate relative path from target file dir
                    let target_dir = target_file_path.parent().unwrap_or(Path::new(""));
                    let rel_path =
                        pathdiff::diff_paths(&abs_target, target_dir).unwrap_or(abs_target);

                    Ok(FieldValue::Path(rel_path))
                }
                _ => Err(format!(
                    "Expected string for field type Path, got {:?}",
                    value
                )),
            }
        }
        FieldType::Currency => {
            match value {
                serde_json::Value::String(s) => {
                    // Parse "100 USD"
                    let parts: Vec<&str> = s.split_whitespace().collect();
                    if parts.len() != 2 {
                        return Err(format!(
                            "Invalid currency format '{}'. Expected 'AMOUNT CURRENCY' (e.g. '100 USD')",
                            s
                        ));
                    }
                    let amount = rust_decimal::Decimal::from_str_exact(parts[0])
                        .map_err(|e| format!("Invalid currency amount: {}", e))?;
                    let currency = iso_currency::Currency::from_code(parts[1])
                        .ok_or_else(|| format!("Invalid currency code: {}", parts[1]))?;

                    Ok(FieldValue::Currency { amount, currency })
                }
                _ => Err(format!(
                    "Expected string for field type Currency, got {:?}",
                    value
                )),
            }
        }
        FieldType::DateTime => match value {
            serde_json::Value::String(s) => {
                let dt = chrono::DateTime::parse_from_rfc3339(s)
                    .map_err(|e| format!("Invalid datetime format: {}", e))?;
                Ok(FieldValue::DateTime(dt))
            }
            _ => Err(format!(
                "Expected string (ISO 8601) for field type DateTime, got {:?}",
                value
            )),
        },
    }
}

/// Parses a list item type string into a FieldType enum.
fn parse_list_item_type(type_str: &str) -> Result<FieldType, String> {
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
        _ => Err(format!(
            "Invalid list item type '{}'. Valid types: string, integer, float, boolean, currency, reference, datetime, path, enum",
            type_str
        )),
    }
}

pub fn success_result(result: AddEntityResult) -> CallToolResult {
    let msg = if result.created_new_file {
        format!("Created new file '{}' and added entity.", result.path)
    } else {
        format!("Added entity to existing file '{}'.", result.path)
    };

    CallToolResult::success(vec![Content::text(msg), Content::text(result.dsl)])
}
