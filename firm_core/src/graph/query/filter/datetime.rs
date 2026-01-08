//! DateTime comparison logic for filters

use chrono::{DateTime, FixedOffset};
use super::types::{FilterOperator, FilterValue};

/// Compare a datetime value against a filter
pub fn compare_datetime(
    value: &DateTime<FixedOffset>,
    operator: &FilterOperator,
    filter_value: &FilterValue,
) -> bool {
    match filter_value {
        FilterValue::DateTime(filter_str) => {
            // Try to parse the filter string as a DateTime
            // Support both full datetime and date-only formats
            if let Ok(filter_dt) = filter_str.parse::<DateTime<FixedOffset>>() {
                match operator {
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
                    match operator {
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

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use super::*;

    // Helper to create a datetime
    fn make_datetime(year: i32, month: u32, day: u32, hour: u32, min: u32, offset_hours: i32) -> DateTime<FixedOffset> {
        let offset = FixedOffset::east_opt(offset_hours * 3600).unwrap();
        offset.with_ymd_and_hms(year, month, day, hour, min, 0).unwrap()
    }

    #[test]
    fn test_equal_full_datetime() {
        let value = make_datetime(2025, 9, 30, 10, 15, 2);
        assert!(compare_datetime(
            &value,
            &FilterOperator::Equal,
            &FilterValue::DateTime("2025-09-30T10:15:00+02:00".to_string()),
        ));
    }

    #[test]
    fn test_not_equal_full_datetime() {
        let value = make_datetime(2025, 9, 30, 10, 15, 2);
        assert!(!compare_datetime(
            &value,
            &FilterOperator::Equal,
            &FilterValue::DateTime("2025-09-30T11:15:00+02:00".to_string()),
        ));
    }

    #[test]
    fn test_equal_date_only() {
        let value = make_datetime(2025, 9, 30, 10, 15, 2);
        // Date-only comparison ignores time
        assert!(compare_datetime(
            &value,
            &FilterOperator::Equal,
            &FilterValue::DateTime("2025-09-30".to_string()),
        ));
    }

    #[test]
    fn test_not_equal_date_only() {
        let value = make_datetime(2025, 9, 30, 10, 15, 2);
        assert!(!compare_datetime(
            &value,
            &FilterOperator::Equal,
            &FilterValue::DateTime("2025-10-01".to_string()),
        ));
    }

    #[test]
    fn test_greater_than_date() {
        let value = make_datetime(2025, 10, 15, 10, 0, 2);
        assert!(compare_datetime(
            &value,
            &FilterOperator::GreaterThan,
            &FilterValue::DateTime("2025-09-01".to_string()),
        ));
    }

    #[test]
    fn test_less_than_date() {
        let value = make_datetime(2025, 9, 15, 10, 0, 2);
        assert!(compare_datetime(
            &value,
            &FilterOperator::LessThan,
            &FilterValue::DateTime("2025-10-01".to_string()),
        ));
    }

    #[test]
    fn test_greater_or_equal_date() {
        let value = make_datetime(2025, 9, 30, 10, 0, 2);
        assert!(compare_datetime(
            &value,
            &FilterOperator::GreaterOrEqual,
            &FilterValue::DateTime("2025-09-30".to_string()),
        ));
    }

    #[test]
    fn test_less_or_equal_date() {
        let value = make_datetime(2025, 9, 30, 10, 0, 2);
        assert!(compare_datetime(
            &value,
            &FilterOperator::LessOrEqual,
            &FilterValue::DateTime("2025-09-30".to_string()),
        ));
    }

    #[test]
    fn test_not_equal_operator() {
        let value = make_datetime(2025, 9, 30, 10, 0, 2);
        assert!(compare_datetime(
            &value,
            &FilterOperator::NotEqual,
            &FilterValue::DateTime("2025-10-01".to_string()),
        ));
    }

    #[test]
    fn test_different_timezones_same_instant() {
        // 10:00 UTC+2 = 08:00 UTC = 09:00 UTC+1
        let value = make_datetime(2025, 9, 30, 10, 0, 2);
        // This represents the same instant in time
        assert!(compare_datetime(
            &value,
            &FilterOperator::Equal,
            &FilterValue::DateTime("2025-09-30T08:00:00+00:00".to_string()),
        ));
    }

    #[test]
    fn test_different_timezones_different_instant() {
        let value = make_datetime(2025, 9, 30, 10, 0, 2);
        // 10:00 UTC+2 != 10:00 UTC+1
        assert!(!compare_datetime(
            &value,
            &FilterOperator::Equal,
            &FilterValue::DateTime("2025-09-30T10:00:00+01:00".to_string()),
        ));
    }

    #[test]
    fn test_greater_than_full_datetime() {
        let value = make_datetime(2025, 9, 30, 15, 30, 2);
        assert!(compare_datetime(
            &value,
            &FilterOperator::GreaterThan,
            &FilterValue::DateTime("2025-09-30T10:00:00+02:00".to_string()),
        ));
    }

    #[test]
    fn test_less_than_full_datetime() {
        let value = make_datetime(2025, 9, 30, 9, 0, 2);
        assert!(compare_datetime(
            &value,
            &FilterOperator::LessThan,
            &FilterValue::DateTime("2025-09-30T10:00:00+02:00".to_string()),
        ));
    }

    #[test]
    fn test_unsupported_operator_contains() {
        let value = make_datetime(2025, 9, 30, 10, 0, 2);
        assert!(!compare_datetime(
            &value,
            &FilterOperator::Contains,
            &FilterValue::DateTime("2025-09-30".to_string()),
        ));
    }

    #[test]
    fn test_invalid_date_format() {
        let value = make_datetime(2025, 9, 30, 10, 0, 2);
        assert!(!compare_datetime(
            &value,
            &FilterOperator::Equal,
            &FilterValue::DateTime("not a date".to_string()),
        ));
    }

    #[test]
    fn test_wrong_filter_type() {
        let value = make_datetime(2025, 9, 30, 10, 0, 2);
        assert!(!compare_datetime(
            &value,
            &FilterOperator::Equal,
            &FilterValue::String("2025-09-30".to_string()),
        ));
    }

    #[test]
    fn test_year_boundary() {
        let value = make_datetime(2025, 1, 1, 0, 0, 0);
        assert!(compare_datetime(
            &value,
            &FilterOperator::GreaterThan,
            &FilterValue::DateTime("2024-12-31".to_string()),
        ));
    }

    #[test]
    fn test_leap_year_date() {
        let value = make_datetime(2024, 2, 29, 12, 0, 0);
        assert!(compare_datetime(
            &value,
            &FilterOperator::Equal,
            &FilterValue::DateTime("2024-02-29".to_string()),
        ));
    }
}
