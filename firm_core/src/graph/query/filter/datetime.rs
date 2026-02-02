//! DateTime comparison logic for filters

use super::super::QueryError;
use super::types::{FilterOperator, FilterValue};
use crate::FieldValue;
use chrono::{DateTime, FixedOffset};

const SUPPORTED_OPS: [&str; 6] = ["==", "!=", ">", "<", ">=", "<="];

/// Compare a datetime field value against a filter
pub fn compare_datetime(
    field_value: &FieldValue,
    operator: &FilterOperator,
    filter_value: &FilterValue,
) -> Result<bool, QueryError> {
    let value = match field_value {
        FieldValue::DateTime(dt) => dt,
        _ => {
            return Err(QueryError::TypeMismatch {
                field_type: field_value.get_type().to_string(),
                filter_type: filter_value.type_name().to_string(),
            })
        }
    };

    match filter_value {
        FilterValue::DateTime(filter_str) => {
            // Try to parse the filter string as a DateTime
            // Support both full datetime and date-only formats
            if let Ok(filter_dt) = filter_str.parse::<DateTime<FixedOffset>>() {
                compare_with_datetime(field_value, value, &filter_dt, operator)
            } else {
                // Try parsing as just a date (YYYY-MM-DD) and compare dates only
                if let Ok(filter_date) = chrono::NaiveDate::parse_from_str(filter_str, "%Y-%m-%d") {
                    compare_with_date(field_value, value, &filter_date, operator)
                } else {
                    Ok(false) // Invalid date format in filter
                }
            }
        }
        _ => Err(QueryError::TypeMismatch {
            field_type: field_value.get_type().to_string(),
            filter_type: filter_value.type_name().to_string(),
        }),
    }
}

fn compare_with_datetime(
    field_value: &FieldValue,
    value: &DateTime<FixedOffset>,
    filter_dt: &DateTime<FixedOffset>,
    operator: &FilterOperator,
) -> Result<bool, QueryError> {
    match operator {
        FilterOperator::Equal => Ok(value == filter_dt),
        FilterOperator::NotEqual => Ok(value != filter_dt),
        FilterOperator::GreaterThan => Ok(value > filter_dt),
        FilterOperator::LessThan => Ok(value < filter_dt),
        FilterOperator::GreaterOrEqual => Ok(value >= filter_dt),
        FilterOperator::LessOrEqual => Ok(value <= filter_dt),
        _ => Err(QueryError::UnsupportedOperator {
            field_type: field_value.get_type().to_string(),
            operator: format!("{:?}", operator),
            supported: SUPPORTED_OPS.iter().map(|s| s.to_string()).collect(),
        }),
    }
}

fn compare_with_date(
    field_value: &FieldValue,
    value: &DateTime<FixedOffset>,
    filter_date: &chrono::NaiveDate,
    operator: &FilterOperator,
) -> Result<bool, QueryError> {
    let value_date = value.date_naive();
    match operator {
        FilterOperator::Equal => Ok(value_date == *filter_date),
        FilterOperator::NotEqual => Ok(value_date != *filter_date),
        FilterOperator::GreaterThan => Ok(value_date > *filter_date),
        FilterOperator::LessThan => Ok(value_date < *filter_date),
        FilterOperator::GreaterOrEqual => Ok(value_date >= *filter_date),
        FilterOperator::LessOrEqual => Ok(value_date <= *filter_date),
        _ => Err(QueryError::UnsupportedOperator {
            field_type: field_value.get_type().to_string(),
            operator: format!("{:?}", operator),
            supported: SUPPORTED_OPS.iter().map(|s| s.to_string()).collect(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn make_datetime_field(year: i32, month: u32, day: u32, hour: u32, min: u32, offset_hours: i32) -> FieldValue {
        let offset = FixedOffset::east_opt(offset_hours * 3600).unwrap();
        let dt = offset.with_ymd_and_hms(year, month, day, hour, min, 0).unwrap();
        FieldValue::DateTime(dt)
    }

    #[test]
    fn test_equal_full_datetime() {
        let field = make_datetime_field(2025, 9, 30, 10, 15, 2);
        assert!(compare_datetime(&field, &FilterOperator::Equal, &FilterValue::DateTime("2025-09-30T10:15:00+02:00".to_string())).unwrap());
    }

    #[test]
    fn test_not_equal_full_datetime() {
        let field = make_datetime_field(2025, 9, 30, 10, 15, 2);
        assert!(!compare_datetime(&field, &FilterOperator::Equal, &FilterValue::DateTime("2025-09-30T11:15:00+02:00".to_string())).unwrap());
    }

    #[test]
    fn test_equal_date_only() {
        let field = make_datetime_field(2025, 9, 30, 10, 15, 2);
        assert!(compare_datetime(&field, &FilterOperator::Equal, &FilterValue::DateTime("2025-09-30".to_string())).unwrap());
    }

    #[test]
    fn test_not_equal_date_only() {
        let field = make_datetime_field(2025, 9, 30, 10, 15, 2);
        assert!(!compare_datetime(&field, &FilterOperator::Equal, &FilterValue::DateTime("2025-10-01".to_string())).unwrap());
    }

    #[test]
    fn test_greater_than_date() {
        let field = make_datetime_field(2025, 10, 15, 10, 0, 2);
        assert!(compare_datetime(&field, &FilterOperator::GreaterThan, &FilterValue::DateTime("2025-09-01".to_string())).unwrap());
    }

    #[test]
    fn test_less_than_date() {
        let field = make_datetime_field(2025, 9, 15, 10, 0, 2);
        assert!(compare_datetime(&field, &FilterOperator::LessThan, &FilterValue::DateTime("2025-10-01".to_string())).unwrap());
    }

    #[test]
    fn test_greater_or_equal_date() {
        let field = make_datetime_field(2025, 9, 30, 10, 0, 2);
        assert!(compare_datetime(&field, &FilterOperator::GreaterOrEqual, &FilterValue::DateTime("2025-09-30".to_string())).unwrap());
    }

    #[test]
    fn test_less_or_equal_date() {
        let field = make_datetime_field(2025, 9, 30, 10, 0, 2);
        assert!(compare_datetime(&field, &FilterOperator::LessOrEqual, &FilterValue::DateTime("2025-09-30".to_string())).unwrap());
    }

    #[test]
    fn test_not_equal_operator() {
        let field = make_datetime_field(2025, 9, 30, 10, 0, 2);
        assert!(compare_datetime(&field, &FilterOperator::NotEqual, &FilterValue::DateTime("2025-10-01".to_string())).unwrap());
    }

    #[test]
    fn test_different_timezones_same_instant() {
        let field = make_datetime_field(2025, 9, 30, 10, 0, 2);
        assert!(compare_datetime(&field, &FilterOperator::Equal, &FilterValue::DateTime("2025-09-30T08:00:00+00:00".to_string())).unwrap());
    }

    #[test]
    fn test_different_timezones_different_instant() {
        let field = make_datetime_field(2025, 9, 30, 10, 0, 2);
        assert!(!compare_datetime(&field, &FilterOperator::Equal, &FilterValue::DateTime("2025-09-30T10:00:00+01:00".to_string())).unwrap());
    }

    #[test]
    fn test_greater_than_full_datetime() {
        let field = make_datetime_field(2025, 9, 30, 15, 30, 2);
        assert!(compare_datetime(&field, &FilterOperator::GreaterThan, &FilterValue::DateTime("2025-09-30T10:00:00+02:00".to_string())).unwrap());
    }

    #[test]
    fn test_less_than_full_datetime() {
        let field = make_datetime_field(2025, 9, 30, 9, 0, 2);
        assert!(compare_datetime(&field, &FilterOperator::LessThan, &FilterValue::DateTime("2025-09-30T10:00:00+02:00".to_string())).unwrap());
    }

    #[test]
    fn test_unsupported_operator_contains() {
        let field = make_datetime_field(2025, 9, 30, 10, 0, 2);
        let result = compare_datetime(&field, &FilterOperator::Contains, &FilterValue::DateTime("2025-09-30".to_string()));
        assert!(matches!(result, Err(QueryError::UnsupportedOperator { .. })));
    }

    #[test]
    fn test_invalid_date_format() {
        let field = make_datetime_field(2025, 9, 30, 10, 0, 2);
        assert!(!compare_datetime(&field, &FilterOperator::Equal, &FilterValue::DateTime("not a date".to_string())).unwrap());
    }

    #[test]
    fn test_wrong_filter_type() {
        let field = make_datetime_field(2025, 9, 30, 10, 0, 2);
        let result = compare_datetime(&field, &FilterOperator::Equal, &FilterValue::String("2025-09-30".to_string()));
        assert!(matches!(result, Err(QueryError::TypeMismatch { .. })));
    }

    #[test]
    fn test_year_boundary() {
        let field = make_datetime_field(2025, 1, 1, 0, 0, 0);
        assert!(compare_datetime(&field, &FilterOperator::GreaterThan, &FilterValue::DateTime("2024-12-31".to_string())).unwrap());
    }

    #[test]
    fn test_leap_year_date() {
        let field = make_datetime_field(2024, 2, 29, 12, 0, 0);
        assert!(compare_datetime(&field, &FilterOperator::Equal, &FilterValue::DateTime("2024-02-29".to_string())).unwrap());
    }
}
