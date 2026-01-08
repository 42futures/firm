//! Query execution structures for the entity graph
//!
//! This module defines the types and logic for executing queries against
//! the entity graph. It is separate from the query language parsing (in firm_lang)
//! and focuses purely on the in-memory graph operations.

use rust_decimal::Decimal;
use iso_currency::Currency;

use crate::{Entity, EntityId, EntityType, FieldId, FieldValue};

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
            _ => false, // TODO: Handle other types
        }
    }

    fn compare_string(&self, value: &str) -> bool {
        match &self.value {
            FilterValue::String(filter_str) => match self.operator {
                FilterOperator::Equal => value == filter_str,
                FilterOperator::NotEqual => value != filter_str,
                FilterOperator::Contains => value.contains(filter_str),
                FilterOperator::StartsWith => value.starts_with(filter_str),
                FilterOperator::EndsWith => value.ends_with(filter_str),
                _ => false,
            },
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
                // Currency code must match
                let currency_str = currency.to_string();
                if currency_str != *filter_code {
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
}
