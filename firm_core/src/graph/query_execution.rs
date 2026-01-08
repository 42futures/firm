//! Query execution structures for the entity graph
//!
//! This module defines the types and logic for executing queries against
//! the entity graph. It is separate from the query language parsing (in firm_lang)
//! and focuses purely on the in-memory graph operations.

use chrono::{DateTime, FixedOffset};
use rust_decimal::Decimal;
use iso_currency::Currency;

use crate::{Entity, EntityType, FieldId, FieldValue};

/// A filter condition for matching entities
#[derive(Debug, Clone, PartialEq)]
pub struct FilterCondition {
    pub field: FieldRef,
    pub operator: FilterOperator,
    pub value: FilterValue,
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

/// Sort direction
#[derive(Debug, Clone, PartialEq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl Default for SortDirection {
    fn default() -> Self {
        SortDirection::Ascending
    }
}

/// A query that can be executed against an entity graph
#[derive(Debug, Clone)]
pub struct Query {
    pub from: EntitySelector,
    pub operations: Vec<QueryOperation>,
}

impl Query {
    /// Create a new query with a starting selector
    pub fn new(from: EntitySelector) -> Self {
        Self {
            from,
            operations: Vec::new(),
        }
    }

    /// Add an operation to the query
    pub fn with_operation(mut self, operation: QueryOperation) -> Self {
        self.operations.push(operation);
        self
    }

    /// Execute the query against an entity graph
    pub fn execute<'a>(&self, graph: &'a super::EntityGraph) -> Vec<&'a Entity> {
        // Start by selecting entities based on the "from" clause
        let mut entities = match &self.from {
            EntitySelector::Type(entity_type) => graph.list_by_type(entity_type),
            EntitySelector::All => {
                // Get all entity types and collect all entities
                let all_types = graph.get_all_entity_types();
                all_types
                    .iter()
                    .flat_map(|entity_type| graph.list_by_type(entity_type))
                    .collect()
            }
        };

        // Apply each operation in sequence
        for operation in &self.operations {
            entities = match operation {
                QueryOperation::Where(condition) => {
                    entities.into_iter().filter(|e| condition.matches(e)).collect()
                }
                QueryOperation::Limit(n) => {
                    entities.into_iter().take(*n).collect()
                }
                // TODO: Implement other operations
                _ => entities, // For now, unimplemented operations just pass through
            };
        }

        entities
    }
}

/// Selects the starting set of entities
#[derive(Debug, Clone, PartialEq)]
pub enum EntitySelector {
    /// Select entities of a specific type
    Type(EntityType),
    /// Select all entities (wildcard)
    All,
}

/// Operations that can be applied to entity collections
#[derive(Debug, Clone)]
pub enum QueryOperation {
    /// Filter entities by a condition
    Where(FilterCondition),
    /// Traverse to related entities
    Related {
        degrees: usize,
        entity_type: Option<EntityType>,
    },
    /// Sort entities by a field
    Order {
        field: FieldId,
        direction: SortDirection,
    },
    /// Limit the number of results
    Limit(usize),
}

impl FilterCondition {
    /// Create a new filter condition
    pub fn new(field: FieldRef, operator: FilterOperator, value: FilterValue) -> Self {
        Self {
            field,
            operator,
            value,
        }
    }

    /// Check if an entity matches this condition
    pub fn matches(&self, entity: &Entity) -> bool {
        match &self.field {
            FieldRef::Metadata(metadata) => self.matches_metadata(entity, metadata),
            FieldRef::Regular(field_id) => self.matches_field(entity, field_id),
        }
    }

    fn matches_metadata(&self, entity: &Entity, metadata: &MetadataField) -> bool {
        match metadata {
            MetadataField::Type => {
                let entity_type = entity.entity_type.to_string();
                self.compare_string(&entity_type)
            }
            MetadataField::Id => {
                let entity_id = entity.id.to_string();
                self.compare_string(&entity_id)
            }
        }
    }

    fn matches_field(&self, entity: &Entity, field_id: &FieldId) -> bool {
        // Get the field value from the entity
        let field_value = match entity.get_field(field_id) {
            Some(value) => value,
            None => return false, // Field doesn't exist, condition fails
        };

        // Compare based on field value type
        match field_value {
            FieldValue::String(s) => self.compare_string(s),
            FieldValue::Integer(i) => self.compare_integer(*i),
            FieldValue::Float(f) => self.compare_float(*f),
            FieldValue::Boolean(b) => self.compare_boolean(*b),
            FieldValue::Currency { amount, currency } => {
                self.compare_currency(amount, currency)
            }
            FieldValue::DateTime(dt) => self.compare_datetime(dt),
            FieldValue::Enum(s) => self.compare_string(s),
            FieldValue::Path(p) => {
                if let Some(path_str) = p.to_str() {
                    self.compare_string(path_str)
                } else {
                    false
                }
            }
            FieldValue::Reference(ref_value) => self.compare_reference(ref_value),
            FieldValue::List(items) => self.compare_list(items)
        }
    }

    fn compare_string(&self, value: &str) -> bool {
        // Get the filter string, which could be String, Enum, or Path variant
        let filter_str = match &self.value {
            FilterValue::String(s) => s,
            FilterValue::Enum(s) => s,
            FilterValue::Path(s) => s,
            _ => return false,
        };

        match self.operator {
            FilterOperator::Equal => value.eq_ignore_ascii_case(filter_str),
            FilterOperator::NotEqual => !value.eq_ignore_ascii_case(filter_str),
            FilterOperator::Contains => {
                value.to_lowercase().contains(&filter_str.to_lowercase())
            }
            FilterOperator::StartsWith => {
                value.to_lowercase().starts_with(&filter_str.to_lowercase())
            }
            FilterOperator::EndsWith => {
                value.to_lowercase().ends_with(&filter_str.to_lowercase())
            }
            _ => false,
        }
    }

    fn compare_integer(&self, value: i64) -> bool {
        match &self.value {
            FilterValue::Integer(filter_int) => match self.operator {
                FilterOperator::Equal => value == *filter_int,
                FilterOperator::NotEqual => value != *filter_int,
                FilterOperator::GreaterThan => value > *filter_int,
                FilterOperator::LessThan => value < *filter_int,
                FilterOperator::GreaterOrEqual => value >= *filter_int,
                FilterOperator::LessOrEqual => value <= *filter_int,
                _ => false,
            },
            FilterValue::Float(filter_float) => match self.operator {
                FilterOperator::Equal => value as f64 == *filter_float,
                FilterOperator::NotEqual => value as f64 != *filter_float,
                FilterOperator::GreaterThan => (value as f64) > *filter_float,
                FilterOperator::LessThan => (value as f64) < *filter_float,
                FilterOperator::GreaterOrEqual => (value as f64) >= *filter_float,
                FilterOperator::LessOrEqual => (value as f64) <= *filter_float,
                _ => false,
            },
            _ => false,
        }
    }

    fn compare_float(&self, value: f64) -> bool {
        match &self.value {
            FilterValue::Float(filter_float) => match self.operator {
                FilterOperator::Equal => (value - filter_float).abs() < f64::EPSILON,
                FilterOperator::NotEqual => (value - filter_float).abs() >= f64::EPSILON,
                FilterOperator::GreaterThan => value > *filter_float,
                FilterOperator::LessThan => value < *filter_float,
                FilterOperator::GreaterOrEqual => value >= *filter_float,
                FilterOperator::LessOrEqual => value <= *filter_float,
                _ => false,
            },
            FilterValue::Integer(filter_int) => match self.operator {
                FilterOperator::Equal => (value - *filter_int as f64).abs() < f64::EPSILON,
                FilterOperator::NotEqual => (value - *filter_int as f64).abs() >= f64::EPSILON,
                FilterOperator::GreaterThan => value > *filter_int as f64,
                FilterOperator::LessThan => value < *filter_int as f64,
                FilterOperator::GreaterOrEqual => value >= *filter_int as f64,
                FilterOperator::LessOrEqual => value <= *filter_int as f64,
                _ => false,
            },
            _ => false,
        }
    }

    fn compare_boolean(&self, value: bool) -> bool {
        match &self.value {
            FilterValue::Boolean(filter_bool) => match self.operator {
                FilterOperator::Equal => value == *filter_bool,
                FilterOperator::NotEqual => value != *filter_bool,
                _ => false,
            },
            _ => false,
        }
    }

    fn compare_currency(&self, amount: &Decimal, currency: &Currency) -> bool {
        match &self.value {
            FilterValue::Currency {
                amount: filter_amount,
                code: filter_code,
            } => {
                // Currency code must match (use code() to get ISO code like "EUR", not full name)
                let currency_code = currency.code();
                if currency_code != filter_code.as_str() {
                    return false;
                }

                // Convert filter amount to Decimal for comparison
                let filter_decimal = Decimal::from_f64_retain(*filter_amount);

                if let Some(filter_dec) = filter_decimal {
                    // Then compare amounts
                    match self.operator {
                        FilterOperator::Equal => amount == &filter_dec,
                        FilterOperator::NotEqual => amount != &filter_dec,
                        FilterOperator::GreaterThan => amount > &filter_dec,
                        FilterOperator::LessThan => amount < &filter_dec,
                        FilterOperator::GreaterOrEqual => amount >= &filter_dec,
                        FilterOperator::LessOrEqual => amount <= &filter_dec,
                        _ => false,
                    }
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn compare_datetime(&self, value: &DateTime<FixedOffset>) -> bool {
        match &self.value {
            FilterValue::DateTime(filter_str) => {
                // Try to parse the filter string as a DateTime
                // Support both full datetime and date-only formats
                if let Ok(filter_dt) = filter_str.parse::<DateTime<FixedOffset>>() {
                    match self.operator {
                        FilterOperator::Equal => value == &filter_dt,
                        FilterOperator::NotEqual => value != &filter_dt,
                        FilterOperator::GreaterThan => value > &filter_dt,
                        FilterOperator::LessThan => value < &filter_dt,
                        FilterOperator::GreaterOrEqual => value >= &filter_dt,
                        FilterOperator::LessOrEqual => value <= &filter_dt,
                        _ => false,
                    }
                } else {
                    // Try parsing as just a date (YYYY-MM-DD) and compare dates only
                    if let Ok(filter_date) = chrono::NaiveDate::parse_from_str(filter_str, "%Y-%m-%d") {
                        let value_date = value.date_naive();
                        match self.operator {
                            FilterOperator::Equal => value_date == filter_date,
                            FilterOperator::NotEqual => value_date != filter_date,
                            FilterOperator::GreaterThan => value_date > filter_date,
                            FilterOperator::LessThan => value_date < filter_date,
                            FilterOperator::GreaterOrEqual => value_date >= filter_date,
                            FilterOperator::LessOrEqual => value_date <= filter_date,
                            _ => false,
                        }
                    } else {
                        false
                    }
                }
            }
            _ => false,
        }
    }

    fn compare_reference(&self, reference: &crate::ReferenceValue) -> bool {
        // Compare reference by converting to string representation
        // Entity references: "person.john_doe"
        // Field references: "person.john_doe.name"
        let ref_str = reference.to_string();

        // The filter value should be a Reference or String variant
        match &self.value {
            FilterValue::Reference(filter_ref_str) => {
                // Case-insensitive comparison of reference strings
                ref_str.eq_ignore_ascii_case(filter_ref_str)
            }
            FilterValue::String(filter_str) => {
                // Also allow comparing against plain strings for convenience
                ref_str.eq_ignore_ascii_case(filter_str)
            }
            _ => false,
        }
    }

    fn compare_list(&self, items: &[FieldValue]) -> bool {
        match self.operator {
            FilterOperator::Contains => {
                // For "contains" operator, check if any list item matches the filter value
                // Support string matching for now (most common use case)
                match &self.value {
                    FilterValue::String(filter_str) => {
                        items.iter().any(|item| {
                            match item {
                                FieldValue::String(s) => s.to_lowercase().contains(&filter_str.to_lowercase()),
                                _ => false,
                            }
                        })
                    }
                    _ => false,
                }
            }
            FilterOperator::Equal => {
                // For equality, compare the entire list (exact match)
                match &self.value {
                    FilterValue::List(filter_items) => {
                        if items.len() != filter_items.len() {
                            return false;
                        }
                        // Compare each item (simplified comparison for now)
                        items.iter().zip(filter_items.iter()).all(|(item, filter_item)| {
                            self.compare_list_item(item, filter_item)
                        })
                    }
                    _ => false,
                }
            }
            _ => false, // Other operators not supported for lists yet
        }
    }

    fn compare_list_item(&self, item: &FieldValue, filter_item: &FilterValue) -> bool {
        // Compare individual list items
        match (item, filter_item) {
            (FieldValue::String(s1), FilterValue::String(s2)) => s1.eq_ignore_ascii_case(s2),
            (FieldValue::Integer(i1), FilterValue::Integer(i2)) => i1 == i2,
            (FieldValue::Float(f1), FilterValue::Float(f2)) => (f1 - f2).abs() < f64::EPSILON,
            (FieldValue::Boolean(b1), FilterValue::Boolean(b2)) => b1 == b2,
            // Add more types as needed
            _ => false,
        }
    }
}
