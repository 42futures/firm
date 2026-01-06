use chrono::{DateTime, FixedOffset, Local, NaiveDate, NaiveTime, Offset, TimeZone};
use iso_currency::Currency;
use path_clean::PathClean;
use rust_decimal::Decimal;
use std::path::PathBuf;
use tree_sitter::Node;

use super::{parser_errors::ValueParseError, parser_utils::get_node_text};

const VALUE_KIND: &str = "value";

/// Internal enum for identifying value types during parsing.
#[derive(Debug, Clone, PartialEq)]
enum ValueKind {
    Boolean,
    String,
    Number,
    Currency,
    Reference,
    List,
    DateTime,
    Date,
    Path,
    Enum,
    Unknown(String),
}

impl From<&str> for ValueKind {
    fn from(kind: &str) -> Self {
        match kind {
            "boolean" => ValueKind::Boolean,
            "string" => ValueKind::String,
            "number" => ValueKind::Number,
            "currency" => ValueKind::Currency,
            "reference" => ValueKind::Reference,
            "list" => ValueKind::List,
            "datetime" => ValueKind::DateTime,
            "date" => ValueKind::Date,
            "path" => ValueKind::Path,
            "enum" => ValueKind::Enum,
            _ => ValueKind::Unknown(kind.to_string()),
        }
    }
}

/// A typed value parsed from Firm DSL source.
///
/// Supports Firm value types including primitives, structured,
/// and temporal types with type safety and validation.
#[derive(Debug, Clone, PartialEq)]
pub enum ParsedValue {
    /// Boolean value (`true` or `false`)
    Boolean(bool),
    /// String value (single or multi-line)
    String(String),
    /// Integer value (`42`)
    Integer(i64),
    /// Floating-point value (`42.5`)
    Float(f64),
    /// Currency value with amount and code (`100.50 USD`)
    Currency { amount: Decimal, currency: Currency },
    /// Entity reference (`contact.john_doe`)
    EntityReference {
        entity_type: String,
        entity_id: String,
    },
    /// Field reference (`contact.john_doe.name`)
    FieldReference {
        entity_type: String,
        entity_id: String,
        field_id: String,
    },
    /// List of values (`["item1", "item2", 42]`)
    List(Vec<ParsedValue>),
    /// Date or datetime value with timezone
    DateTime(DateTime<FixedOffset>),
    /// A path to a file or directory
    Path(PathBuf),
    /// An enum value (`enum"customer"`)
    Enum(String),
}

impl ParsedValue {
    /// Gets the type name of this parsed value for error reporting and type checking.
    pub fn get_type_name(&self) -> &'static str {
        match self {
            ParsedValue::Boolean(_) => "Boolean",
            ParsedValue::String(_) => "String",
            ParsedValue::Integer(_) => "Integer",
            ParsedValue::Float(_) => "Float",
            ParsedValue::Currency { .. } => "Currency",
            ParsedValue::EntityReference { .. } => "EntityReference",
            ParsedValue::FieldReference { .. } => "FieldReference",
            ParsedValue::List(_) => "List",
            ParsedValue::DateTime(_) => "DateTime",
            ParsedValue::Path(_) => "Path",
            ParsedValue::Enum(_) => "Enum",
        }
    }

    /// Parses a value from a tree-sitter node with type conversion.
    pub fn from_node<'a>(
        node: Node<'a>,
        source: &'a str,
        path: &'a PathBuf,
    ) -> Result<ParsedValue, ValueParseError> {
        let kind = Self::get_value_kind(node).ok_or(ValueParseError::UnknownValueKind)?;
        let raw = get_node_text(&node, source);

        match kind {
            ValueKind::Boolean => Self::parse_boolean(&raw),
            ValueKind::String => Self::parse_string(&raw),
            ValueKind::Number => Self::parse_number(&raw),
            ValueKind::Currency => Self::parse_currency(&raw),
            ValueKind::Reference => Self::parse_reference(&raw),
            ValueKind::List => Self::parse_list(node, source, path),
            ValueKind::Date => Self::parse_date(&raw),
            ValueKind::DateTime => Self::parse_datetime(&raw),
            ValueKind::Path => Self::parse_path(&raw, path),
            ValueKind::Enum => Self::parse_enum(&raw),
            _ => Err(ValueParseError::MissingParseMethod),
        }
    }

    /// Determines the value type from a tree-sitter node.
    fn get_value_kind<'a>(value_node: Node<'a>) -> Option<ValueKind> {
        let mut cursor = value_node.walk();
        let kind = value_node.children(&mut cursor).next()?.kind();
        Some(kind.into())
    }

    /// Parses boolean values (`true` or `false`).
    fn parse_boolean(raw: &str) -> Result<ParsedValue, ValueParseError> {
        raw.parse()
            .map(ParsedValue::Boolean)
            .map_err(|_| ValueParseError::InvalidBoolean(raw.to_string()))
    }

    /// Parses string values, handling both single-line and multi-line formats.
    fn parse_string(raw: &str) -> Result<ParsedValue, ValueParseError> {
        // Multi-line strings start and end with triple quotes ("""stuff""")
        if raw.starts_with("\"\"\"") && raw.ends_with("\"\"\"") {
            // Handle triple quotes
            let content = raw.trim_start_matches("\"\"\"").trim_end_matches("\"\"\"");

            // Remove common indentation
            let trimmed = Self::trim_common_indentation(content);
            Ok(ParsedValue::String(trimmed))
        }
        // Single-line strings start and end with single quotes ("stuff")
        else {
            // Handle single quotes
            Ok(ParsedValue::String(raw.trim_matches('"').to_string()))
        }
    }

    /// Parses numeric values, distinguishing between integers and floats.
    fn parse_number(raw: &str) -> Result<ParsedValue, ValueParseError> {
        // Numbers with a period are floats (42.0)
        if raw.contains(".") {
            raw.parse()
                .map(ParsedValue::Float)
                .map_err(|_| ValueParseError::InvalidFloat(raw.to_string()))
        }
        // Numbers without are period are ints (42)
        else {
            raw.parse()
                .map(ParsedValue::Integer)
                .map_err(|_| ValueParseError::InvalidInteger(raw.to_string()))
        }
    }

    /// Parses currency values with amount and currency code (`42.50 USD`).
    fn parse_currency(raw: &str) -> Result<ParsedValue, ValueParseError> {
        let parts: Vec<&str> = raw.split(" ").collect();

        // Currencies have 2 parts: number and currency (42 USD or 42.24 EUR)
        match parts.as_slice() {
            [raw_amount, raw_currency] => {
                let amount = rust_decimal::Decimal::from_str_exact(&raw_amount)
                    .map_err(|_| ValueParseError::InvalidCurrencyAmount(raw_amount.to_string()))?;

                let currency = raw_currency
                    .parse::<Currency>()
                    .map_err(|_| ValueParseError::InvalidCurrencyCode(raw_currency.to_string()))?;

                Ok(ParsedValue::Currency { amount, currency })
            }
            _ => Err(ValueParseError::InvalidCurrencyFormat {
                source: raw.to_string(),
                parts_count: parts.len(),
            }),
        }
    }

    /// Parses reference values (entity or field references).
    fn parse_reference(raw: &str) -> Result<ParsedValue, ValueParseError> {
        let parts: Vec<&str> = raw.split(".").collect();
        match parts.len() {
            // References with 2 parts are for entities (contact.john)
            2 => Ok(ParsedValue::EntityReference {
                entity_type: parts[0].to_string(),
                entity_id: parts[1].to_string(),
            }),
            // References with 3 parts are for fields (contact.john.name)
            3 => Ok(ParsedValue::FieldReference {
                entity_type: parts[0].to_string(),
                entity_id: parts[1].to_string(),
                field_id: parts[2].to_string(),
            }),
            // References with more or less parts are invalid
            _ => Err(ValueParseError::InvalidReferenceFormat {
                source: raw.to_string(),
                parts_count: parts.len(),
            }),
        }
    }

    /// Parses list values by recursively parsing each contained value.
    /// Ensures all list items are of the same type (homogeneous lists).
    fn parse_list<'a>(
        node: Node<'a>,
        source: &'a str,
        path: &'a PathBuf,
    ) -> Result<ParsedValue, ValueParseError> {
        // For lists, we walk each child value node and parse it
        let mut items: Vec<ParsedValue> = Vec::new();
        let mut cursor = node.walk();
        let mut expected_type: Option<&'static str> = None;

        if let Some(list_node) = node.children(&mut cursor).next() {
            let mut list_cursor = list_node.walk();
            let mut index = 0;

            for child in list_node.children(&mut list_cursor) {
                if child.kind() == VALUE_KIND {
                    // Recursively parse the list child value
                    let item = Self::from_node(child, source, path)?;

                    // Check type homogeneity
                    let item_type = item.get_type_name();
                    match expected_type {
                        None => {
                            // First item - establish the expected type
                            expected_type = Some(item_type);
                        }
                        Some(expected) => {
                            // Subsequent items - check they match the first type
                            if item_type != expected {
                                return Err(ValueParseError::HeterogeneousList {
                                    expected_type: expected.to_string(),
                                    found_type: item_type.to_string(),
                                    index,
                                });
                            }
                        }
                    }

                    items.push(item);
                    index += 1;
                }
            }
        }

        Ok(ParsedValue::List(items))
    }

    /// Parses date values (`2024-03-20`) as datetime at midnight local time.
    fn parse_date(raw: &str) -> Result<ParsedValue, ValueParseError> {
        // Parse "naive date" in year-month-day format (2025-07-31)
        let date = NaiveDate::parse_from_str(raw, "%Y-%m-%d")
            .map_err(|_| ValueParseError::InvalidDate(raw.to_string()))?;

        // Assume time is midnight local time
        let datetime = date.and_hms_opt(0, 0, 0).unwrap();
        let local_offset = Local::now().offset().fix();

        // Convert from local datetime to timezoned datetime
        let with_tz = local_offset
            .from_local_datetime(&datetime)
            .single()
            .ok_or_else(|| ValueParseError::InvalidDate(raw.to_string()))?;

        Ok(ParsedValue::DateTime(with_tz))
    }

    /// Parses datetime values with optional timezone (`2024-03-20 at 14:30 UTC-5`).
    fn parse_datetime(raw: &str) -> Result<ParsedValue, ValueParseError> {
        // Datetimes start with year-month-day (2025-07-31) followed by " at ", then time (09:42), optionally timezone " UTC+3"
        let parts: Vec<&str> = raw.split(" at ").collect();
        match parts.as_slice() {
            [date_part, time_and_tz] => {
                // Parse the date part
                let date = NaiveDate::parse_from_str(date_part, "%Y-%m-%d")
                    .map_err(|_| ValueParseError::InvalidDateTime(raw.to_string()))?;

                // Split time and timezone
                let (time_part, tz_offset) = if time_and_tz.contains(" UTC") {
                    let tz_parts: Vec<&str> = time_and_tz.split(" UTC").collect();
                    let offset_str = tz_parts.get(1).unwrap_or(&"");
                    (tz_parts[0], Self::parse_utc_offset(offset_str)?)
                } else {
                    // No timezone specified - use local timezone
                    let local_offset = Local::now().offset().fix();
                    (*time_and_tz, local_offset)
                };

                // Parse time (handle both h:mm and hh:mm)
                let time = NaiveTime::parse_from_str(time_part, "%H:%M")
                    .or_else(|_| NaiveTime::parse_from_str(time_part, "%k:%M"))
                    .map_err(|_| ValueParseError::InvalidDateTime(raw.to_string()))?;

                // Combine date and time, then apply timezone
                let naive_dt = date.and_time(time);
                let dt = tz_offset
                    .from_local_datetime(&naive_dt)
                    .single()
                    .ok_or_else(|| ValueParseError::InvalidDateTime(raw.to_string()))?;

                Ok(ParsedValue::DateTime(dt))
            }
            _ => Err(ValueParseError::InvalidDateTime(raw.to_string())),
        }
    }

    /// Parses file path values.
    ///
    /// Relative paths are assumed to be relative to the source file they're defined in.
    /// On parse, we transform the source-relative path to a workspace-relative path and normalize it.
    /// Absolute paths are left as-is.
    fn parse_path(raw: &str, source_path: &PathBuf) -> Result<ParsedValue, ValueParseError> {
        let raw_path = raw.replace("path\"", "").trim_matches('"').to_string();
        let target_path = PathBuf::from(raw_path);

        // Transform the target path to the source path's parent directory
        let combined_path = if target_path.is_absolute() {
            target_path
        } else if let Some(source_dir) = source_path.parent() {
            source_dir.join(&target_path)
        } else {
            target_path
        };

        // Clean the path, and prepend ./ for consistency if it's relative
        let cleaned_path = combined_path.clean();
        let final_path = if cleaned_path.is_relative() && !cleaned_path.starts_with("..") {
            PathBuf::from("./").join(cleaned_path)
        } else {
            cleaned_path
        };

        Ok(ParsedValue::Path(final_path))
    }

    /// Parses enum values (`enum"customer"`).
    fn parse_enum(raw: &str) -> Result<ParsedValue, ValueParseError> {
        let enum_value = raw.replace("enum\"", "").trim_matches('"').to_string();
        Ok(ParsedValue::Enum(enum_value))
    }

    /// Removes common leading whitespace from multi-line strings.
    fn trim_common_indentation(s: &str) -> String {
        let lines: Vec<&str> = s.lines().collect();

        // Find minimum indentation of non-empty lines (skip first/last if empty)
        let min_indent = lines
            .iter()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.len() - line.trim_start().len())
            .min()
            .unwrap_or(0);

        // Remove common indentation and trim leading/trailing empty lines
        lines
            .iter()
            .map(|line| {
                if line.len() >= min_indent {
                    &line[min_indent..]
                } else {
                    line
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string()
    }

    /// Parses UTC timezone offset strings (e.g., "+3", "-5", or empty for UTC).
    fn parse_utc_offset(offset_str: &str) -> Result<FixedOffset, ValueParseError> {
        if offset_str.is_empty() {
            // Just "UTC" with no offset
            return Ok(FixedOffset::east_opt(0).unwrap());
        }

        let hours: i32 = offset_str
            .parse()
            .map_err(|_| ValueParseError::InvalidTimezone(offset_str.to_string()))?;

        FixedOffset::east_opt(hours * 3600)
            .ok_or_else(|| ValueParseError::InvalidTimezone(offset_str.to_string()))
    }
}
