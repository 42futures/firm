//! Parsed query structures for query language

use std::fmt;

/// Represents a complete parsed query
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedQuery {
    pub from: ParsedFromClause,
    pub operations: Vec<ParsedOperation>,
}

/// The FROM clause specifies the starting entity type(s)
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedFromClause {
    pub selector: ParsedEntitySelector,
}

/// Entity selector: specific type or wildcard
#[derive(Debug, Clone, PartialEq)]
pub enum ParsedEntitySelector {
    Type(String),
    Wildcard,
}

/// Operations that can be chained in a query
#[derive(Debug, Clone, PartialEq)]
pub enum ParsedOperation {
    Where(ParsedCompoundCondition),
    Related {
        degree: Option<usize>,
        selector: Option<ParsedEntitySelector>,
    },
    Order {
        field: ParsedField,
        direction: ParsedDirection,
    },
    Limit(usize),
}

/// A compound condition combining multiple conditions with AND/OR
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedCompoundCondition {
    pub conditions: Vec<ParsedCondition>,
    pub combinator: ParsedCombinator,
}

/// Logical combinator for compound conditions
#[derive(Debug, Clone, PartialEq, Default)]
pub enum ParsedCombinator {
    #[default]
    And,
    Or,
}

/// A single condition in a WHERE clause
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedCondition {
    pub field: ParsedField,
    pub operator: ParsedOperator,
    pub value: ParsedQueryValue,
}

/// Field reference (metadata or regular field)
#[derive(Debug, Clone, PartialEq)]
pub enum ParsedField {
    Metadata(String), // @type, @id
    Regular(String),  // field_name
}

/// Comparison operators
#[derive(Debug, Clone, PartialEq)]
pub enum ParsedOperator {
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

/// Values in conditions
#[derive(Debug, Clone, PartialEq)]
pub enum ParsedQueryValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Currency { amount: f64, code: String },
    DateTime(String),  // ISO format string
    Reference(String), // Reference string like "person.john_doe" or "person.john_doe.field"
    Path(String),
    Enum(String),
    List(Vec<ParsedQueryValue>),
}

/// Sort direction
#[derive(Debug, Clone, PartialEq)]
#[derive(Default)]
pub enum ParsedDirection {
    #[default]
    Ascending,
    Descending,
}


impl fmt::Display for ParsedEntitySelector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParsedEntitySelector::Type(t) => write!(f, "{}", t),
            ParsedEntitySelector::Wildcard => write!(f, "*"),
        }
    }
}

impl fmt::Display for ParsedOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParsedOperator::Equal => write!(f, "=="),
            ParsedOperator::NotEqual => write!(f, "!="),
            ParsedOperator::GreaterThan => write!(f, ">"),
            ParsedOperator::LessThan => write!(f, "<"),
            ParsedOperator::GreaterOrEqual => write!(f, ">="),
            ParsedOperator::LessOrEqual => write!(f, "<="),
            ParsedOperator::Contains => write!(f, "contains"),
            ParsedOperator::StartsWith => write!(f, "startswith"),
            ParsedOperator::EndsWith => write!(f, "endswith"),
            ParsedOperator::In => write!(f, "in"),
        }
    }
}

impl fmt::Display for ParsedDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParsedDirection::Ascending => write!(f, "asc"),
            ParsedDirection::Descending => write!(f, "desc"),
        }
    }
}
