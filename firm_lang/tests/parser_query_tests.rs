//! Tests for query language parsing

use firm_lang::parser::query::{
    ParsedCombinator, ParsedDirection, ParsedEntitySelector, ParsedField, ParsedOperation,
    ParsedQueryValue, parse_query,
};

#[test]
fn test_parse_simple_query() {
    let query_str = "from task | where is_completed == false | limit 5";
    let result = parse_query(query_str);
    assert!(result.is_ok());

    let query = result.unwrap();
    assert_eq!(
        query.from.selector,
        ParsedEntitySelector::Type("task".to_string())
    );
    assert_eq!(query.operations.len(), 2);
}

#[test]
fn test_parse_wildcard() {
    let query_str = "from * | where @type == \"task\"";
    let result = parse_query(query_str);
    assert!(result.is_ok());

    let query = result.unwrap();
    assert_eq!(query.from.selector, ParsedEntitySelector::Wildcard);
}

#[test]
fn test_parse_related_with_degree() {
    let query_str = "from project | related(2) task";
    let result = parse_query(query_str);
    assert!(result.is_ok());

    let query = result.unwrap();
    if let Some(ParsedOperation::Related { degree, selector }) = query.operations.first() {
        assert_eq!(*degree, Some(2));
        assert_eq!(
            *selector,
            Some(ParsedEntitySelector::Type("task".to_string()))
        );
    } else {
        panic!("Expected Related operation");
    }
}

#[test]
fn test_parse_order_with_direction() {
    let query_str = "from task | order due_date desc";
    let result = parse_query(query_str);
    assert!(result.is_ok());

    let query = result.unwrap();
    if let Some(ParsedOperation::Order { field, direction }) = query.operations.first() {
        assert_eq!(*field, ParsedField::Regular("due_date".to_string()));
        assert_eq!(*direction, ParsedDirection::Descending);
    } else {
        panic!("Expected Order operation");
    }
}

#[test]
fn test_parse_currency_value() {
    let query_str = "from project | where budget == 5000.50 USD";
    let result = parse_query(query_str);
    assert!(result.is_ok());

    let query = result.unwrap();
    if let Some(ParsedOperation::Where(compound)) = query.operations.first() {
        let condition = &compound.conditions[0];
        if let ParsedQueryValue::Currency { amount, code } = &condition.value {
            assert!((amount - 5000.50).abs() < f64::EPSILON);
            assert_eq!(code, "USD");
        } else {
            panic!("Expected Currency value");
        }
    }
}

#[test]
fn test_parse_datetime_value() {
    let query_str = "from task | where created > 2025-01-15";
    let result = parse_query(query_str);
    assert!(result.is_ok());

    let query = result.unwrap();
    if let Some(ParsedOperation::Where(compound)) = query.operations.first() {
        let condition = &compound.conditions[0];
        assert!(matches!(condition.value, ParsedQueryValue::DateTime(_)));
    }
}

#[test]
fn test_parse_reference_value() {
    let query_str = "from task | where assignee == person.john_doe";
    let result = parse_query(query_str);
    assert!(result.is_ok());

    let query = result.unwrap();
    if let Some(ParsedOperation::Where(compound)) = query.operations.first() {
        let condition = &compound.conditions[0];
        if let ParsedQueryValue::Reference(ref_str) = &condition.value {
            assert_eq!(ref_str, "person.john_doe");
        } else {
            panic!("Expected Reference value");
        }
    }
}

#[test]
fn test_parse_enum_value() {
    let query_str = "from task | where status == enum\"completed\"";
    let result = parse_query(query_str);
    assert!(result.is_ok());

    let query = result.unwrap();
    if let Some(ParsedOperation::Where(compound)) = query.operations.first() {
        let condition = &compound.conditions[0];
        if let ParsedQueryValue::Enum(enum_val) = &condition.value {
            assert_eq!(enum_val, "completed");
        } else {
            panic!("Expected Enum value");
        }
    }
}

#[test]
fn test_parse_path_value() {
    let query_str = "from document | where file == path\"./file.pdf\"";
    let result = parse_query(query_str);
    assert!(result.is_ok());

    let query = result.unwrap();
    if let Some(ParsedOperation::Where(compound)) = query.operations.first() {
        let condition = &compound.conditions[0];
        if let ParsedQueryValue::Path(path_str) = &condition.value {
            assert_eq!(path_str, "./file.pdf");
        } else {
            panic!("Expected Path value");
        }
    }
}

#[test]
fn test_parse_compound_condition_or() {
    let query_str = "from invoice | where status == \"draft\" or status == \"sent\"";
    let result = parse_query(query_str);
    assert!(result.is_ok());

    let query = result.unwrap();
    if let Some(ParsedOperation::Where(compound)) = query.operations.first() {
        assert_eq!(compound.conditions.len(), 2);
        assert_eq!(compound.combinator, ParsedCombinator::Or);
    } else {
        panic!("Expected Where operation");
    }
}

#[test]
fn test_parse_compound_condition_and() {
    let query_str = "from task | where is_completed == true and priority > 5";
    let result = parse_query(query_str);
    assert!(result.is_ok());

    let query = result.unwrap();
    if let Some(ParsedOperation::Where(compound)) = query.operations.first() {
        assert_eq!(compound.conditions.len(), 2);
        assert_eq!(compound.combinator, ParsedCombinator::And);
    } else {
        panic!("Expected Where operation");
    }
}

#[test]
fn test_parse_compound_condition_multiple_or() {
    let query_str =
        "from invoice | where status == \"draft\" or status == \"sent\" or status == \"overdue\"";
    let result = parse_query(query_str);
    assert!(result.is_ok());

    let query = result.unwrap();
    if let Some(ParsedOperation::Where(compound)) = query.operations.first() {
        assert_eq!(compound.conditions.len(), 3);
        assert_eq!(compound.combinator, ParsedCombinator::Or);
    } else {
        panic!("Expected Where operation");
    }
}

#[test]
fn test_parse_compound_condition_case_insensitive() {
    // Test uppercase OR
    let query_str = "from invoice | where status == \"draft\" OR status == \"sent\"";
    let query = parse_query(query_str).unwrap();
    if let Some(ParsedOperation::Where(compound)) = query.operations.first() {
        assert_eq!(compound.combinator, ParsedCombinator::Or);
    }

    // Test uppercase AND
    let query_str = "from task | where a == 1 AND b == 2";
    let query = parse_query(query_str).unwrap();
    if let Some(ParsedOperation::Where(compound)) = query.operations.first() {
        assert_eq!(compound.combinator, ParsedCombinator::And);
    }
}

#[test]
fn test_parse_compound_condition_mixed_error() {
    let query_str = "from task | where a == 1 or b == 2 and c == 3";
    let result = parse_query(query_str);
    assert!(result.is_err());
}
