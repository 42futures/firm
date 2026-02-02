//! Filter type definitions

use crate::FieldId;

/// Logical operator for combining multiple filter conditions
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Combinator {
    #[default]
    And,
    Or,
}

/// Reference to a field (either metadata or regular field)
#[derive(Debug, Clone, PartialEq)]
pub enum FieldRef {
    /// Metadata field like @type or @id
    Metadata(MetadataField),
    /// Regular entity field
    Regular(FieldId),
}

/// Metadata fields that can be queried
#[derive(Debug, Clone, PartialEq)]
pub enum MetadataField {
    Type,
    Id,
}

/// Comparison operators for filtering
#[derive(Debug, Clone, PartialEq)]
pub enum FilterOperator {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterOrEqual,
    LessOrEqual,
    Contains,
    StartsWith,
    EndsWith,
    In,
}

/// Values used in filter conditions
#[derive(Debug, Clone, PartialEq)]
pub enum FilterValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Currency { amount: f64, code: String },
    DateTime(String),
    Reference(String),
    Path(String),
    Enum(String),
    List(Vec<FilterValue>),
}

impl FilterValue {
    /// Returns the type name of this filter value for error messages
    pub fn type_name(&self) -> &'static str {
        match self {
            FilterValue::String(_) => "String",
            FilterValue::Integer(_) => "Integer",
            FilterValue::Float(_) => "Float",
            FilterValue::Boolean(_) => "Boolean",
            FilterValue::Currency { .. } => "Currency",
            FilterValue::DateTime(_) => "DateTime",
            FilterValue::Reference(_) => "Reference",
            FilterValue::Path(_) => "Path",
            FilterValue::Enum(_) => "Enum",
            FilterValue::List(_) => "List",
        }
    }
}
