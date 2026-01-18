//! String comparison logic for filters

use super::types::{FilterOperator, FilterValue};

/// Compare a string value against a filter
pub fn compare_string(value: &str, operator: &FilterOperator, filter_value: &FilterValue) -> bool {
    // Get the filter string, which could be String, Enum, or Path variant
    let filter_str = match filter_value {
        FilterValue::String(s) => s,
        FilterValue::Enum(s) => s,
        FilterValue::Path(s) => s,
        _ => return false,
    };

    match operator {
        FilterOperator::Equal => value.eq_ignore_ascii_case(filter_str),
        FilterOperator::NotEqual => !value.eq_ignore_ascii_case(filter_str),
        FilterOperator::Contains => value.to_lowercase().contains(&filter_str.to_lowercase()),
        FilterOperator::StartsWith => value.to_lowercase().starts_with(&filter_str.to_lowercase()),
        FilterOperator::EndsWith => value.to_lowercase().ends_with(&filter_str.to_lowercase()),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== String Filter Tests =====

    #[test]
    fn test_equal_exact_match() {
        assert!(compare_string(
            "hello",
            &FilterOperator::Equal,
            &FilterValue::String("hello".to_string())
        ));
    }

    #[test]
    fn test_equal_case_insensitive() {
        assert!(compare_string(
            "Hello",
            &FilterOperator::Equal,
            &FilterValue::String("hello".to_string())
        ));
        assert!(compare_string(
            "HELLO",
            &FilterOperator::Equal,
            &FilterValue::String("hello".to_string())
        ));
        assert!(compare_string(
            "hello",
            &FilterOperator::Equal,
            &FilterValue::String("HELLO".to_string())
        ));
    }

    #[test]
    fn test_not_equal() {
        assert!(!compare_string(
            "hello",
            &FilterOperator::Equal,
            &FilterValue::String("world".to_string())
        ));
    }

    #[test]
    fn test_not_equal_operator() {
        assert!(compare_string(
            "hello",
            &FilterOperator::NotEqual,
            &FilterValue::String("world".to_string())
        ));
        assert!(!compare_string(
            "hello",
            &FilterOperator::NotEqual,
            &FilterValue::String("hello".to_string())
        ));
    }

    #[test]
    fn test_contains() {
        assert!(compare_string(
            "hello world",
            &FilterOperator::Contains,
            &FilterValue::String("world".to_string())
        ));
        assert!(compare_string(
            "hello world",
            &FilterOperator::Contains,
            &FilterValue::String("hello".to_string())
        ));
        assert!(compare_string(
            "hello world",
            &FilterOperator::Contains,
            &FilterValue::String("lo wo".to_string())
        ));
    }

    #[test]
    fn test_contains_case_insensitive() {
        assert!(compare_string(
            "Hello World",
            &FilterOperator::Contains,
            &FilterValue::String("world".to_string())
        ));
        assert!(compare_string(
            "hello world",
            &FilterOperator::Contains,
            &FilterValue::String("WORLD".to_string())
        ));
    }

    #[test]
    fn test_contains_not_found() {
        assert!(!compare_string(
            "hello world",
            &FilterOperator::Contains,
            &FilterValue::String("goodbye".to_string())
        ));
    }

    #[test]
    fn test_starts_with() {
        assert!(compare_string(
            "hello world",
            &FilterOperator::StartsWith,
            &FilterValue::String("hello".to_string())
        ));
    }

    #[test]
    fn test_starts_with_case_insensitive() {
        assert!(compare_string(
            "Hello World",
            &FilterOperator::StartsWith,
            &FilterValue::String("hello".to_string())
        ));
        assert!(compare_string(
            "hello world",
            &FilterOperator::StartsWith,
            &FilterValue::String("HELLO".to_string())
        ));
    }

    #[test]
    fn test_starts_with_not_match() {
        assert!(!compare_string(
            "hello world",
            &FilterOperator::StartsWith,
            &FilterValue::String("world".to_string())
        ));
    }

    #[test]
    fn test_ends_with() {
        assert!(compare_string(
            "hello world",
            &FilterOperator::EndsWith,
            &FilterValue::String("world".to_string())
        ));
    }

    #[test]
    fn test_ends_with_case_insensitive() {
        assert!(compare_string(
            "Hello World",
            &FilterOperator::EndsWith,
            &FilterValue::String("world".to_string())
        ));
        assert!(compare_string(
            "hello world",
            &FilterOperator::EndsWith,
            &FilterValue::String("WORLD".to_string())
        ));
    }

    #[test]
    fn test_ends_with_not_match() {
        assert!(!compare_string(
            "hello world",
            &FilterOperator::EndsWith,
            &FilterValue::String("hello".to_string())
        ));
    }

    // ===== Enum Filter Tests =====

    #[test]
    fn test_equal_with_enum_filter() {
        assert!(compare_string(
            "Active",
            &FilterOperator::Equal,
            &FilterValue::Enum("Active".to_string())
        ));
    }

    #[test]
    fn test_equal_enum_case_insensitive() {
        assert!(compare_string(
            "Active",
            &FilterOperator::Equal,
            &FilterValue::Enum("active".to_string())
        ));
    }

    #[test]
    fn test_contains_with_enum_filter() {
        assert!(compare_string(
            "IsActive",
            &FilterOperator::Contains,
            &FilterValue::Enum("Active".to_string())
        ));
    }

    // ===== Path Filter Tests =====

    #[test]
    fn test_equal_with_path_filter() {
        assert!(compare_string(
            "./docs/file.txt",
            &FilterOperator::Equal,
            &FilterValue::Path("./docs/file.txt".to_string())
        ));
    }

    #[test]
    fn test_equal_path_case_insensitive() {
        assert!(compare_string(
            "./Docs/File.txt",
            &FilterOperator::Equal,
            &FilterValue::Path("./docs/file.txt".to_string())
        ));
    }

    #[test]
    fn test_contains_with_path_filter() {
        assert!(compare_string(
            "./docs/my_file.txt",
            &FilterOperator::Contains,
            &FilterValue::Path("my_file".to_string())
        ));
    }

    #[test]
    fn test_starts_with_path() {
        assert!(compare_string(
            "./docs/file.txt",
            &FilterOperator::StartsWith,
            &FilterValue::Path("./docs".to_string())
        ));
    }

    #[test]
    fn test_ends_with_path_extension() {
        assert!(compare_string(
            "./docs/file.txt",
            &FilterOperator::EndsWith,
            &FilterValue::Path(".txt".to_string())
        ));
    }

    // ===== Edge Cases =====

    #[test]
    fn test_empty_string_equal() {
        assert!(compare_string(
            "",
            &FilterOperator::Equal,
            &FilterValue::String("".to_string())
        ));
    }

    #[test]
    fn test_empty_string_not_equal_nonempty() {
        assert!(!compare_string(
            "",
            &FilterOperator::Equal,
            &FilterValue::String("hello".to_string())
        ));
    }

    #[test]
    fn test_contains_empty_string() {
        // Every string contains an empty string
        assert!(compare_string(
            "hello",
            &FilterOperator::Contains,
            &FilterValue::String("".to_string())
        ));
    }

    #[test]
    fn test_special_characters() {
        assert!(compare_string(
            "hello@world.com",
            &FilterOperator::Equal,
            &FilterValue::String("hello@world.com".to_string())
        ));
        assert!(compare_string(
            "hello@world.com",
            &FilterOperator::Contains,
            &FilterValue::String("@".to_string())
        ));
    }

    #[test]
    fn test_unicode_characters() {
        assert!(compare_string(
            "café",
            &FilterOperator::Equal,
            &FilterValue::String("café".to_string())
        ));
        assert!(compare_string(
            "こんにちは",
            &FilterOperator::Contains,
            &FilterValue::String("にち".to_string())
        ));
    }

    #[test]
    fn test_whitespace() {
        assert!(compare_string(
            "hello world",
            &FilterOperator::Equal,
            &FilterValue::String("hello world".to_string())
        ));
        assert!(compare_string(
            "hello world",
            &FilterOperator::Contains,
            &FilterValue::String(" ".to_string())
        ));
    }

    #[test]
    fn test_multiline_string() {
        assert!(compare_string(
            "hello\nworld",
            &FilterOperator::Contains,
            &FilterValue::String("world".to_string())
        ));
    }

    #[test]
    fn test_unsupported_operator_greater_than() {
        assert!(!compare_string(
            "hello",
            &FilterOperator::GreaterThan,
            &FilterValue::String("hello".to_string())
        ));
    }

    #[test]
    fn test_unsupported_operator_less_than() {
        assert!(!compare_string(
            "hello",
            &FilterOperator::LessThan,
            &FilterValue::String("hello".to_string())
        ));
    }

    #[test]
    fn test_wrong_filter_type_integer() {
        assert!(!compare_string(
            "42",
            &FilterOperator::Equal,
            &FilterValue::Integer(42)
        ));
    }

    #[test]
    fn test_wrong_filter_type_boolean() {
        assert!(!compare_string(
            "true",
            &FilterOperator::Equal,
            &FilterValue::Boolean(true)
        ));
    }
}
