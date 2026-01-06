use chrono::{DateTime, FixedOffset};
use std::path::PathBuf;

use firm_core::{FieldValue, ReferenceValue};

use super::GeneratorOptions;

/// Generate DSL for en entity field value.
pub fn generate_value(value: &FieldValue, options: &GeneratorOptions) -> String {
    match value {
        FieldValue::Boolean(b) => b.to_string(),
        FieldValue::String(s) => generate_string(s, options),
        FieldValue::Integer(i) => i.to_string(),
        FieldValue::Float(f) => generate_float(f),
        FieldValue::Currency { amount, currency } => {
            format!("{} {}", amount, currency.code())
        }
        FieldValue::Reference(reference) => generate_reference(reference),
        FieldValue::List(values) => generate_list(values, options),
        FieldValue::DateTime(dt) => generate_datetime(dt),
        FieldValue::Path(path) => generate_path(path),
        FieldValue::Enum(s) => generate_enum(s),
    }
}

/// Generate string value with proper quoting.
fn generate_string(s: &str, options: &GeneratorOptions) -> String {
    if s.contains('\n') {
        // Multi-line string
        let indent = options.indent_style.indent_string(1);
        let lines: Vec<&str> = s.lines().collect();

        let mut result = String::from("\"\"\"\n");
        for line in lines {
            result.push_str(&format!("{}{}\n", indent, line));
        }
        result.push_str(&format!("{}\"\"\"", indent));
        result
    } else {
        // Single-line string with escape handling
        format!("\"{}\"", s.replace('\"', "\\\""))
    }
}

/// Generate float value, ensuring it always has a decimal place.
fn generate_float(f: &f64) -> String {
    let formatted = f.to_string();

    // If the float doesn't contain a decimal point, add .0
    if !formatted.contains('.') {
        format!("{}.0", formatted)
    } else {
        formatted
    }
}

/// Generate entity/field reference value.
fn generate_reference(reference: &ReferenceValue) -> String {
    match reference {
        ReferenceValue::Entity(entity_id) => entity_id.0.clone(),
        ReferenceValue::Field(entity_id, field_id) => {
            format!("{}.{}", entity_id.0, field_id.0)
        }
    }
}

/// Generate list value.
fn generate_list(values: &[FieldValue], options: &GeneratorOptions) -> String {
    if values.is_empty() {
        return "[]".to_string();
    }

    let value_strings: Vec<String> = values.iter().map(|v| generate_value(v, options)).collect();

    format!("[{}]", value_strings.join(", "))
}

/// Generate datetime value.
fn generate_datetime(dt: &DateTime<FixedOffset>) -> String {
    let date_str = dt.format("%Y-%m-%d").to_string();
    let time_str = dt.format("%H:%M").to_string();

    // Handle timezone
    let offset_seconds = dt.offset().local_minus_utc();
    let timezone_str = if offset_seconds == 0 {
        "UTC".to_string()
    } else {
        let hours = offset_seconds / 3600;
        if hours > 0 {
            format!("UTC+{}", hours)
        } else {
            format!("UTC{}", hours) // hours is already negative
        }
    };

    format!("{} at {} {}", date_str, time_str, timezone_str)
}

/// Generate path value.
fn generate_path(path: &PathBuf) -> String {
    format!("path\"{}\"", path.display())
}

/// Generate enum value.
fn generate_enum(value: &str) -> String {
    format!("enum\"{}\"", value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use firm_core::{EntityId, FieldId, FieldValue, ReferenceValue};
    use iso_currency::Currency;
    use rust_decimal::Decimal;

    #[test]
    fn test_generate_boolean_true() {
        let options = GeneratorOptions::default();
        let result = generate_value(&FieldValue::Boolean(true), &options);
        assert_eq!(result, "true");
    }

    #[test]
    fn test_generate_boolean_false() {
        let options = GeneratorOptions::default();
        let result = generate_value(&FieldValue::Boolean(false), &options);
        assert_eq!(result, "false");
    }

    #[test]
    fn test_generate_string_single_line() {
        let options = GeneratorOptions::default();
        let result = generate_string("Hello World", &options);
        assert_eq!(result, "\"Hello World\"");
    }

    #[test]
    fn test_generate_string_with_quotes() {
        let options = GeneratorOptions::default();
        let result = generate_string("Say \"Hello\"", &options);
        assert_eq!(result, "\"Say \\\"Hello\\\"\"");
    }

    #[test]
    fn test_generate_string_multiline() {
        let options = GeneratorOptions::default();
        let multiline = "Line 1\nLine 2\nLine 3";
        let result = generate_string(multiline, &options);

        let expected = "\"\"\"\n    Line 1\n    Line 2\n    Line 3\n    \"\"\"";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_generate_string_empty() {
        let options = GeneratorOptions::default();
        let result = generate_string("", &options);
        assert_eq!(result, "\"\"");
    }

    #[test]
    fn test_generate_integer_positive() {
        let options = GeneratorOptions::default();
        let result = generate_value(&FieldValue::Integer(42), &options);
        assert_eq!(result, "42");
    }

    #[test]
    fn test_generate_integer_negative() {
        let options = GeneratorOptions::default();
        let result = generate_value(&FieldValue::Integer(-123), &options);
        assert_eq!(result, "-123");
    }

    #[test]
    fn test_generate_integer_zero() {
        let options = GeneratorOptions::default();
        let result = generate_value(&FieldValue::Integer(0), &options);
        assert_eq!(result, "0");
    }

    #[test]
    fn test_generate_float_positive() {
        let options = GeneratorOptions::default();
        let result = generate_value(&FieldValue::Float(3.14159), &options);
        assert_eq!(result, "3.14159");
    }

    #[test]
    fn test_generate_float_negative() {
        let options = GeneratorOptions::default();
        let result = generate_value(&FieldValue::Float(-2.5), &options);
        assert_eq!(result, "-2.5");
    }

    #[test]
    fn test_generate_float_zero() {
        let options = GeneratorOptions::default();
        let result = generate_value(&FieldValue::Float(0.0), &options);
        assert_eq!(result, "0.0");
    }

    #[test]
    fn test_generate_currency_usd() {
        let options = GeneratorOptions::default();
        let result = generate_value(
            &FieldValue::Currency {
                amount: Decimal::from_str_exact("1000.50").unwrap(),
                currency: Currency::USD,
            },
            &options,
        );
        assert_eq!(result, "1000.50 USD");
    }

    #[test]
    fn test_generate_currency_eur() {
        let options = GeneratorOptions::default();
        let result = generate_value(
            &FieldValue::Currency {
                amount: Decimal::from_str_exact("750").unwrap(),
                currency: Currency::EUR,
            },
            &options,
        );
        assert_eq!(result, "750 EUR");
    }

    #[test]
    fn test_generate_currency_zero() {
        let options = GeneratorOptions::default();
        let result = generate_value(
            &FieldValue::Currency {
                amount: Decimal::ZERO,
                currency: Currency::DKK,
            },
            &options,
        );
        assert_eq!(result, "0 DKK");
    }

    #[test]
    fn test_generate_reference_entity() {
        let reference = ReferenceValue::Entity(EntityId("person.john".to_string()));
        let result = generate_reference(&reference);
        assert_eq!(result, "person.john");
    }

    #[test]
    fn test_generate_reference_field() {
        let reference = ReferenceValue::Field(
            EntityId("person.john".to_string()),
            FieldId("name".to_string()),
        );
        let result = generate_reference(&reference);
        assert_eq!(result, "person.john.name");
    }

    #[test]
    fn test_generate_empty_list() {
        let options = GeneratorOptions::default();
        let result = generate_list(&[], &options);
        assert_eq!(result, "[]");
    }

    #[test]
    fn test_generate_string_list() {
        let options = GeneratorOptions::default();
        let values = vec![
            FieldValue::String("first".to_string()),
            FieldValue::String("second".to_string()),
        ];
        let result = generate_list(&values, &options);
        assert_eq!(result, "[\"first\", \"second\"]");
    }

    #[test]
    fn test_generate_integer_list() {
        let options = GeneratorOptions::default();
        let values = vec![
            FieldValue::Integer(1),
            FieldValue::Integer(2),
            FieldValue::Integer(3),
        ];
        let result = generate_list(&values, &options);
        assert_eq!(result, "[1, 2, 3]");
    }

    #[test]
    fn test_generate_boolean_list() {
        let options = GeneratorOptions::default();
        let values = vec![
            FieldValue::Boolean(true),
            FieldValue::Boolean(false),
            FieldValue::Boolean(true),
        ];
        let result = generate_list(&values, &options);
        assert_eq!(result, "[true, false, true]");
    }

    #[test]
    fn test_generate_nested_lists() {
        let options = GeneratorOptions::default();

        // Create nested list: [["a", "b"], ["c", "d"]]
        let inner_list1 = vec![
            FieldValue::String("a".to_string()),
            FieldValue::String("b".to_string()),
        ];
        let inner_list2 = vec![
            FieldValue::String("c".to_string()),
            FieldValue::String("d".to_string()),
        ];

        let nested_list = vec![FieldValue::List(inner_list1), FieldValue::List(inner_list2)];

        let result = generate_list(&nested_list, &options);
        assert_eq!(result, "[[\"a\", \"b\"], [\"c\", \"d\"]]");
    }

    #[test]
    fn test_generate_datetime_utc() {
        let dt = FixedOffset::east_opt(0)
            .unwrap()
            .with_ymd_and_hms(2024, 3, 20, 14, 30, 0)
            .unwrap();
        let result = generate_datetime(&dt);
        assert_eq!(result, "2024-03-20 at 14:30 UTC");
    }

    #[test]
    fn test_generate_datetime_positive_offset() {
        let dt = FixedOffset::east_opt(3 * 3600)
            .unwrap() // UTC+3
            .with_ymd_and_hms(2024, 3, 20, 14, 30, 0)
            .unwrap();
        let result = generate_datetime(&dt);
        assert_eq!(result, "2024-03-20 at 14:30 UTC+3");
    }

    #[test]
    fn test_generate_datetime_negative_offset() {
        let dt = FixedOffset::west_opt(5 * 3600)
            .unwrap() // UTC-5
            .with_ymd_and_hms(2024, 3, 20, 14, 30, 0)
            .unwrap();
        let result = generate_datetime(&dt);
        assert_eq!(result, "2024-03-20 at 14:30 UTC-5");
    }

    #[test]
    fn test_generate_datetime_single_digit_hour() {
        let dt = FixedOffset::east_opt(0)
            .unwrap()
            .with_ymd_and_hms(2024, 3, 20, 9, 5, 0)
            .unwrap();
        let result = generate_datetime(&dt);
        assert_eq!(result, "2024-03-20 at 09:05 UTC");
    }

    #[test]
    fn test_generate_datetime_midnight() {
        let dt = FixedOffset::east_opt(0)
            .unwrap()
            .with_ymd_and_hms(2024, 12, 31, 0, 0, 0)
            .unwrap();
        let result = generate_datetime(&dt);
        assert_eq!(result, "2024-12-31 at 00:00 UTC");
    }

    #[test]
    fn test_generate_path() {
        let result = generate_path(&PathBuf::from("./relative/path.txt"));
        assert_eq!(result, "path\"./relative/path.txt\"");
    }
}
