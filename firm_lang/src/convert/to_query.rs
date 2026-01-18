//! Conversion from ParsedQuery to executable Query

use firm_core::graph::{
    EntitySelector, FieldRef, FilterCondition, FilterOperator, FilterValue, MetadataField, Query,
    QueryOperation, SortDirection,
};
use firm_core::{EntityType, FieldId};

use crate::parser::query::*;

/// Error type for query conversion
#[derive(Debug, Clone, PartialEq)]
pub enum QueryConversionError {
    UnsupportedOperation(String),
    InvalidValue(String),
}

impl std::fmt::Display for QueryConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryConversionError::UnsupportedOperation(msg) => {
                write!(f, "Unsupported operation: {}", msg)
            }
            QueryConversionError::InvalidValue(msg) => write!(f, "Invalid value: {}", msg),
        }
    }
}

impl std::error::Error for QueryConversionError {}

/// Convert ParsedQuery to executable Query
impl TryFrom<ParsedQuery> for Query {
    type Error = QueryConversionError;

    fn try_from(parsed: ParsedQuery) -> Result<Self, Self::Error> {
        // Convert the "from" clause
        let from = match parsed.from.selector {
            ParsedEntitySelector::Type(type_str) => {
                EntitySelector::Type(EntityType::new(&type_str))
            }
            ParsedEntitySelector::Wildcard => EntitySelector::All,
        };

        let mut query = Query::new(from);

        // Convert each operation
        for parsed_op in parsed.operations {
            let operation = convert_operation(parsed_op)?;
            query = query.with_operation(operation);
        }

        Ok(query)
    }
}

fn convert_operation(parsed: ParsedOperation) -> Result<QueryOperation, QueryConversionError> {
    match parsed {
        ParsedOperation::Where(condition) => {
            let filter_condition = convert_condition(condition)?;
            Ok(QueryOperation::Where(filter_condition))
        }
        ParsedOperation::Limit(n) => Ok(QueryOperation::Limit(n)),
        ParsedOperation::Order { field, direction } => convert_order(field, direction),
        ParsedOperation::Related { degree, selector } => convert_related(degree, selector),
    }
}

fn convert_condition(parsed: ParsedCondition) -> Result<FilterCondition, QueryConversionError> {
    let field = convert_field(parsed.field);
    let operator = convert_operator(parsed.operator);
    let value = convert_value(parsed.value)?;

    Ok(FilterCondition::new(field, operator, value))
}

fn convert_order(
    field: ParsedField,
    direction: ParsedDirection,
) -> Result<QueryOperation, QueryConversionError> {
    let field_ref = convert_field(field);
    let sort_direction = convert_direction(direction);

    Ok(QueryOperation::Order {
        field: field_ref,
        direction: sort_direction,
    })
}

fn convert_related(
    degree: Option<usize>,
    selector: Option<ParsedEntitySelector>,
) -> Result<QueryOperation, QueryConversionError> {
    // Default to 1 degree if not specified
    let degrees = degree.unwrap_or(1);
    let entity_type = selector.and_then(|sel| match sel {
        ParsedEntitySelector::Type(type_str) => Some(EntityType::new(&type_str)),
        ParsedEntitySelector::Wildcard => None,
    });

    Ok(QueryOperation::Related {
        degrees,
        entity_type,
    })
}

fn convert_field(parsed: ParsedField) -> FieldRef {
    match parsed {
        ParsedField::Metadata(name) => {
            let metadata = match name.as_str() {
                "type" => MetadataField::Type,
                "id" => MetadataField::Id,
                _ => MetadataField::Type, // Default fallback
            };
            FieldRef::Metadata(metadata)
        }
        ParsedField::Regular(name) => FieldRef::Regular(FieldId::new(&name)),
    }
}

fn convert_operator(parsed: ParsedOperator) -> FilterOperator {
    match parsed {
        ParsedOperator::Equal => FilterOperator::Equal,
        ParsedOperator::NotEqual => FilterOperator::NotEqual,
        ParsedOperator::GreaterThan => FilterOperator::GreaterThan,
        ParsedOperator::LessThan => FilterOperator::LessThan,
        ParsedOperator::GreaterOrEqual => FilterOperator::GreaterOrEqual,
        ParsedOperator::LessOrEqual => FilterOperator::LessOrEqual,
        ParsedOperator::Contains => FilterOperator::Contains,
        ParsedOperator::StartsWith => FilterOperator::StartsWith,
        ParsedOperator::EndsWith => FilterOperator::EndsWith,
        ParsedOperator::In => FilterOperator::In,
    }
}

fn convert_value(parsed: ParsedQueryValue) -> Result<FilterValue, QueryConversionError> {
    match parsed {
        ParsedQueryValue::String(s) => Ok(FilterValue::String(s)),
        ParsedQueryValue::Number(n) => {
            // Try to determine if it's an integer or float
            if n.fract() == 0.0 && n.is_finite() {
                Ok(FilterValue::Integer(n as i64))
            } else {
                Ok(FilterValue::Float(n))
            }
        }
        ParsedQueryValue::Boolean(b) => Ok(FilterValue::Boolean(b)),
        ParsedQueryValue::Currency { amount, code } => Ok(FilterValue::Currency { amount, code }),
        ParsedQueryValue::DateTime(s) => Ok(FilterValue::DateTime(s)),
        ParsedQueryValue::Reference(s) => Ok(FilterValue::Reference(s)),
        ParsedQueryValue::Path(s) => {
            // TODO: Path resolution context
            // Currently assumes paths in queries are workspace-relative (matching how paths are stored in the graph).
            // In the future, we may want to pass workspace_path here and apply the same transformation as DSL parsing:
            // - Join with workspace path if relative
            // - Clean/normalize the path
            // This would make query paths CWD-relative and transform them to workspace-relative for comparison.
            Ok(FilterValue::Path(s))
        }
        ParsedQueryValue::Enum(s) => Ok(FilterValue::Enum(s)),
        ParsedQueryValue::List(items) => {
            let converted: Result<Vec<FilterValue>, _> =
                items.into_iter().map(convert_value).collect();
            Ok(FilterValue::List(converted?))
        }
    }
}

fn convert_direction(parsed: ParsedDirection) -> SortDirection {
    match parsed {
        ParsedDirection::Ascending => SortDirection::Ascending,
        ParsedDirection::Descending => SortDirection::Descending,
    }
}
