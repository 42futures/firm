//! Tests for query conversion from parsed AST to executable queries

use firm_lang::parser::query::parse_query;
use firm_core::graph::{Query, QueryOperation, EntitySelector, FilterOperator, FilterValue, FieldRef, MetadataField, SortDirection};
use firm_core::EntityType;

#[test]
fn test_convert_simple_query() {
    let query_str = "from task | limit 5";
    let parsed = parse_query(query_str).unwrap();
    let query: Query = parsed.try_into().unwrap();

    assert!(matches!(query.from, EntitySelector::Type(_)));
    assert_eq!(query.operations.len(), 1);
    assert!(matches!(query.operations[0], QueryOperation::Limit(5)));
}

#[test]
fn test_convert_wildcard_selector() {
    let query_str = "from *";
    let parsed = parse_query(query_str).unwrap();
    let query: Query = parsed.try_into().unwrap();

    assert!(matches!(query.from, EntitySelector::All));
}

#[test]
fn test_convert_where_with_regular_field() {
    let query_str = "from task | where is_completed == true";
    let parsed = parse_query(query_str).unwrap();
    let query: Query = parsed.try_into().unwrap();

    assert_eq!(query.operations.len(), 1);
    if let QueryOperation::Where(condition) = &query.operations[0] {
        assert!(matches!(condition.field, FieldRef::Regular(_)));
        assert!(matches!(condition.operator, FilterOperator::Equal));
        assert!(matches!(condition.value, FilterValue::Boolean(true)));
    } else {
        panic!("Expected Where operation");
    }
}

#[test]
fn test_convert_where_with_metadata_field() {
    let query_str = "from * | where @type == \"task\"";
    let parsed = parse_query(query_str).unwrap();
    let query: Query = parsed.try_into().unwrap();

    assert_eq!(query.operations.len(), 1);
    if let QueryOperation::Where(condition) = &query.operations[0] {
        assert!(matches!(condition.field, FieldRef::Metadata(MetadataField::Type)));
        assert!(matches!(condition.value, FilterValue::String(_)));
    } else {
        panic!("Expected Where operation");
    }
}

#[test]
fn test_convert_order_regular_field() {
    let query_str = "from task | order name";
    let parsed = parse_query(query_str).unwrap();
    let query: Query = parsed.try_into().unwrap();

    assert_eq!(query.operations.len(), 1);
    if let QueryOperation::Order { field, direction } = &query.operations[0] {
        assert!(matches!(field, FieldRef::Regular(_)));
        assert!(matches!(direction, SortDirection::Ascending));
    } else {
        panic!("Expected Order operation");
    }
}

#[test]
fn test_convert_order_metadata_field() {
    let query_str = "from * | order @type desc";
    let parsed = parse_query(query_str).unwrap();
    let query: Query = parsed.try_into().unwrap();

    assert_eq!(query.operations.len(), 1);
    if let QueryOperation::Order { field, direction } = &query.operations[0] {
        assert!(matches!(field, FieldRef::Metadata(MetadataField::Type)));
        assert!(matches!(direction, SortDirection::Descending));
    } else {
        panic!("Expected Order operation");
    }
}

#[test]
fn test_convert_related_with_degree() {
    let query_str = "from person | related(2)";
    let parsed = parse_query(query_str).unwrap();
    let query: Query = parsed.try_into().unwrap();

    assert_eq!(query.operations.len(), 1);
    if let QueryOperation::Related { degrees, entity_type } = &query.operations[0] {
        assert_eq!(*degrees, 2);
        assert!(entity_type.is_none());
    } else {
        panic!("Expected Related operation");
    }
}

#[test]
fn test_convert_related_with_type_filter() {
    let query_str = "from person | related task";
    let parsed = parse_query(query_str).unwrap();
    let query: Query = parsed.try_into().unwrap();

    assert_eq!(query.operations.len(), 1);
    if let QueryOperation::Related { degrees, entity_type } = &query.operations[0] {
        assert_eq!(*degrees, 1); // Default degree
        assert!(entity_type.is_some());
        assert_eq!(entity_type.as_ref().unwrap(), &EntityType::new("task"));
    } else {
        panic!("Expected Related operation");
    }
}

#[test]
fn test_convert_related_with_degree_and_type() {
    let query_str = "from person | related(3) task";
    let parsed = parse_query(query_str).unwrap();
    let query: Query = parsed.try_into().unwrap();

    assert_eq!(query.operations.len(), 1);
    if let QueryOperation::Related { degrees, entity_type } = &query.operations[0] {
        assert_eq!(*degrees, 3);
        assert!(entity_type.is_some());
        assert_eq!(entity_type.as_ref().unwrap(), &EntityType::new("task"));
    } else {
        panic!("Expected Related operation");
    }
}

#[test]
fn test_convert_chained_operations() {
    let query_str = "from task | where is_completed == false | order due_date | limit 10";
    let parsed = parse_query(query_str).unwrap();
    let query: Query = parsed.try_into().unwrap();

    assert_eq!(query.operations.len(), 3);
    assert!(matches!(query.operations[0], QueryOperation::Where(_)));
    assert!(matches!(query.operations[1], QueryOperation::Order { .. }));
    assert!(matches!(query.operations[2], QueryOperation::Limit(10)));
}

#[test]
fn test_convert_currency_value() {
    let query_str = "from project | where budget == 5000.50 USD";
    let parsed = parse_query(query_str).unwrap();
    let query: Query = parsed.try_into().unwrap();

    if let QueryOperation::Where(condition) = &query.operations[0] {
        if let FilterValue::Currency { amount, code } = &condition.value {
            assert_eq!(*amount, 5000.50);
            assert_eq!(code, "USD");
        } else {
            panic!("Expected Currency value");
        }
    } else {
        panic!("Expected Where operation");
    }
}

#[test]
fn test_convert_reference_value() {
    let query_str = "from task | where assignee == person.john_doe";
    let parsed = parse_query(query_str).unwrap();
    let query: Query = parsed.try_into().unwrap();

    if let QueryOperation::Where(condition) = &query.operations[0] {
        if let FilterValue::Reference(ref_str) = &condition.value {
            assert_eq!(ref_str, "person.john_doe");
        } else {
            panic!("Expected Reference value");
        }
    } else {
        panic!("Expected Where operation");
    }
}
