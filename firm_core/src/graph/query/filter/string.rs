//! String comparison logic for filters

use super::super::QueryError;
use super::types::{FilterOperator, FilterValue};
use crate::FieldValue;

/// Compare a string-like field value against a filter
/// Handles String, Enum, and Path field types
pub fn compare_string(
    field_value: &FieldValue,
    operator: &FilterOperator,
    filter_value: &FilterValue,
) -> Result<bool, QueryError> {
    let value = match field_value {
        FieldValue::String(s) => s.as_str(),
        FieldValue::Enum(s) => s.as_str(),
        FieldValue::Path(p) => {
            return compare_path(field_value, p, operator, filter_value);
        }
        _ => {
            return Err(QueryError::TypeMismatch {
                field_type: field_value.get_type().to_string(),
                filter_type: filter_value.type_name().to_string(),
            })
        }
    };

    // Get the filter string, which could be String, Enum, or Path variant
    let filter_str = match filter_value {
        FilterValue::String(s) => s,
        FilterValue::Enum(s) => s,
        FilterValue::Path(s) => s,
        _ => {
            return Err(QueryError::TypeMismatch {
                field_type: field_value.get_type().to_string(),
                filter_type: filter_value.type_name().to_string(),
            })
        }
    };

    match operator {
        FilterOperator::Equal => Ok(value.eq_ignore_ascii_case(filter_str)),
        FilterOperator::NotEqual => Ok(!value.eq_ignore_ascii_case(filter_str)),
        FilterOperator::Contains => {
            Ok(value.to_lowercase().contains(&filter_str.to_lowercase()))
        }
        FilterOperator::StartsWith => {
            Ok(value.to_lowercase().starts_with(&filter_str.to_lowercase()))
        }
        FilterOperator::EndsWith => {
            Ok(value.to_lowercase().ends_with(&filter_str.to_lowercase()))
        }
        _ => Err(QueryError::UnsupportedOperator {
            field_type: field_value.get_type().to_string(),
            operator: format!("{:?}", operator),
            supported: vec![
                "==".to_string(),
                "!=".to_string(),
                "contains".to_string(),
                "starts_with".to_string(),
                "ends_with".to_string(),
            ],
        }),
    }
}

/// Helper for Path field values that need to_str() conversion
fn compare_path(
    field_value: &FieldValue,
    path: &std::path::PathBuf,
    operator: &FilterOperator,
    filter_value: &FilterValue,
) -> Result<bool, QueryError> {
    let value = match path.to_str() {
        Some(s) => s,
        None => return Ok(false), // Invalid UTF-8 in path
    };

    let filter_str = match filter_value {
        FilterValue::String(s) => s,
        FilterValue::Enum(s) => s,
        FilterValue::Path(s) => s,
        _ => {
            return Err(QueryError::TypeMismatch {
                field_type: field_value.get_type().to_string(),
                filter_type: filter_value.type_name().to_string(),
            })
        }
    };

    match operator {
        FilterOperator::Equal => Ok(value.eq_ignore_ascii_case(filter_str)),
        FilterOperator::NotEqual => Ok(!value.eq_ignore_ascii_case(filter_str)),
        FilterOperator::Contains => {
            Ok(value.to_lowercase().contains(&filter_str.to_lowercase()))
        }
        FilterOperator::StartsWith => {
            Ok(value.to_lowercase().starts_with(&filter_str.to_lowercase()))
        }
        FilterOperator::EndsWith => {
            Ok(value.to_lowercase().ends_with(&filter_str.to_lowercase()))
        }
        _ => Err(QueryError::UnsupportedOperator {
            field_type: field_value.get_type().to_string(),
            operator: format!("{:?}", operator),
            supported: vec![
                "==".to_string(),
                "!=".to_string(),
                "contains".to_string(),
                "starts_with".to_string(),
                "ends_with".to_string(),
            ],
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn str_field(s: &str) -> FieldValue {
        FieldValue::String(s.to_string())
    }

    fn enum_field(s: &str) -> FieldValue {
        FieldValue::Enum(s.to_string())
    }

    // ===== String Filter Tests =====

    #[test]
    fn test_equal_exact_match() {
        let field = str_field("hello");
        assert!(compare_string(&field, &FilterOperator::Equal, &FilterValue::String("hello".to_string())).unwrap());
    }

    #[test]
    fn test_equal_case_insensitive() {
        assert!(compare_string(&str_field("Hello"), &FilterOperator::Equal, &FilterValue::String("hello".to_string())).unwrap());
        assert!(compare_string(&str_field("HELLO"), &FilterOperator::Equal, &FilterValue::String("hello".to_string())).unwrap());
        assert!(compare_string(&str_field("hello"), &FilterOperator::Equal, &FilterValue::String("HELLO".to_string())).unwrap());
    }

    #[test]
    fn test_not_equal() {
        assert!(!compare_string(&str_field("hello"), &FilterOperator::Equal, &FilterValue::String("world".to_string())).unwrap());
    }

    #[test]
    fn test_not_equal_operator() {
        assert!(compare_string(&str_field("hello"), &FilterOperator::NotEqual, &FilterValue::String("world".to_string())).unwrap());
        assert!(!compare_string(&str_field("hello"), &FilterOperator::NotEqual, &FilterValue::String("hello".to_string())).unwrap());
    }

    #[test]
    fn test_contains() {
        assert!(compare_string(&str_field("hello world"), &FilterOperator::Contains, &FilterValue::String("world".to_string())).unwrap());
        assert!(compare_string(&str_field("hello world"), &FilterOperator::Contains, &FilterValue::String("hello".to_string())).unwrap());
        assert!(compare_string(&str_field("hello world"), &FilterOperator::Contains, &FilterValue::String("lo wo".to_string())).unwrap());
    }

    #[test]
    fn test_contains_case_insensitive() {
        assert!(compare_string(&str_field("Hello World"), &FilterOperator::Contains, &FilterValue::String("world".to_string())).unwrap());
        assert!(compare_string(&str_field("hello world"), &FilterOperator::Contains, &FilterValue::String("WORLD".to_string())).unwrap());
    }

    #[test]
    fn test_contains_not_found() {
        assert!(!compare_string(&str_field("hello world"), &FilterOperator::Contains, &FilterValue::String("goodbye".to_string())).unwrap());
    }

    #[test]
    fn test_starts_with() {
        assert!(compare_string(&str_field("hello world"), &FilterOperator::StartsWith, &FilterValue::String("hello".to_string())).unwrap());
    }

    #[test]
    fn test_starts_with_case_insensitive() {
        assert!(compare_string(&str_field("Hello World"), &FilterOperator::StartsWith, &FilterValue::String("hello".to_string())).unwrap());
        assert!(compare_string(&str_field("hello world"), &FilterOperator::StartsWith, &FilterValue::String("HELLO".to_string())).unwrap());
    }

    #[test]
    fn test_starts_with_not_match() {
        assert!(!compare_string(&str_field("hello world"), &FilterOperator::StartsWith, &FilterValue::String("world".to_string())).unwrap());
    }

    #[test]
    fn test_ends_with() {
        assert!(compare_string(&str_field("hello world"), &FilterOperator::EndsWith, &FilterValue::String("world".to_string())).unwrap());
    }

    #[test]
    fn test_ends_with_case_insensitive() {
        assert!(compare_string(&str_field("Hello World"), &FilterOperator::EndsWith, &FilterValue::String("world".to_string())).unwrap());
        assert!(compare_string(&str_field("hello world"), &FilterOperator::EndsWith, &FilterValue::String("WORLD".to_string())).unwrap());
    }

    #[test]
    fn test_ends_with_not_match() {
        assert!(!compare_string(&str_field("hello world"), &FilterOperator::EndsWith, &FilterValue::String("hello".to_string())).unwrap());
    }

    // ===== Enum Filter Tests =====

    #[test]
    fn test_equal_with_enum_filter() {
        assert!(compare_string(&enum_field("Active"), &FilterOperator::Equal, &FilterValue::Enum("Active".to_string())).unwrap());
    }

    #[test]
    fn test_equal_enum_case_insensitive() {
        assert!(compare_string(&enum_field("Active"), &FilterOperator::Equal, &FilterValue::Enum("active".to_string())).unwrap());
    }

    #[test]
    fn test_contains_with_enum_filter() {
        assert!(compare_string(&enum_field("IsActive"), &FilterOperator::Contains, &FilterValue::Enum("Active".to_string())).unwrap());
    }

    // ===== Path Filter Tests =====

    #[test]
    fn test_equal_with_path_filter() {
        assert!(compare_string(&str_field("./docs/file.txt"), &FilterOperator::Equal, &FilterValue::Path("./docs/file.txt".to_string())).unwrap());
    }

    #[test]
    fn test_equal_path_case_insensitive() {
        assert!(compare_string(&str_field("./Docs/File.txt"), &FilterOperator::Equal, &FilterValue::Path("./docs/file.txt".to_string())).unwrap());
    }

    #[test]
    fn test_contains_with_path_filter() {
        assert!(compare_string(&str_field("./docs/my_file.txt"), &FilterOperator::Contains, &FilterValue::Path("my_file".to_string())).unwrap());
    }

    #[test]
    fn test_starts_with_path() {
        assert!(compare_string(&str_field("./docs/file.txt"), &FilterOperator::StartsWith, &FilterValue::Path("./docs".to_string())).unwrap());
    }

    #[test]
    fn test_ends_with_path_extension() {
        assert!(compare_string(&str_field("./docs/file.txt"), &FilterOperator::EndsWith, &FilterValue::Path(".txt".to_string())).unwrap());
    }

    // ===== Edge Cases =====

    #[test]
    fn test_empty_string_equal() {
        assert!(compare_string(&str_field(""), &FilterOperator::Equal, &FilterValue::String("".to_string())).unwrap());
    }

    #[test]
    fn test_empty_string_not_equal_nonempty() {
        assert!(!compare_string(&str_field(""), &FilterOperator::Equal, &FilterValue::String("hello".to_string())).unwrap());
    }

    #[test]
    fn test_contains_empty_string() {
        // Every string contains an empty string
        assert!(compare_string(&str_field("hello"), &FilterOperator::Contains, &FilterValue::String("".to_string())).unwrap());
    }

    #[test]
    fn test_special_characters() {
        assert!(compare_string(&str_field("hello@world.com"), &FilterOperator::Equal, &FilterValue::String("hello@world.com".to_string())).unwrap());
        assert!(compare_string(&str_field("hello@world.com"), &FilterOperator::Contains, &FilterValue::String("@".to_string())).unwrap());
    }

    #[test]
    fn test_unicode_characters() {
        assert!(compare_string(&str_field("café"), &FilterOperator::Equal, &FilterValue::String("café".to_string())).unwrap());
        assert!(compare_string(&str_field("こんにちは"), &FilterOperator::Contains, &FilterValue::String("にち".to_string())).unwrap());
    }

    #[test]
    fn test_whitespace() {
        assert!(compare_string(&str_field("hello world"), &FilterOperator::Equal, &FilterValue::String("hello world".to_string())).unwrap());
        assert!(compare_string(&str_field("hello world"), &FilterOperator::Contains, &FilterValue::String(" ".to_string())).unwrap());
    }

    #[test]
    fn test_multiline_string() {
        assert!(compare_string(&str_field("hello\nworld"), &FilterOperator::Contains, &FilterValue::String("world".to_string())).unwrap());
    }

    #[test]
    fn test_unsupported_operator_greater_than() {
        let result = compare_string(&str_field("hello"), &FilterOperator::GreaterThan, &FilterValue::String("hello".to_string()));
        assert!(matches!(result, Err(QueryError::UnsupportedOperator { .. })));
    }

    #[test]
    fn test_unsupported_operator_less_than() {
        let result = compare_string(&str_field("hello"), &FilterOperator::LessThan, &FilterValue::String("hello".to_string()));
        assert!(matches!(result, Err(QueryError::UnsupportedOperator { .. })));
    }

    #[test]
    fn test_wrong_filter_type_integer() {
        let result = compare_string(&str_field("42"), &FilterOperator::Equal, &FilterValue::Integer(42));
        assert!(matches!(result, Err(QueryError::TypeMismatch { .. })));
    }

    #[test]
    fn test_wrong_filter_type_boolean() {
        let result = compare_string(&str_field("true"), &FilterOperator::Equal, &FilterValue::Boolean(true));
        assert!(matches!(result, Err(QueryError::TypeMismatch { .. })));
    }
}
