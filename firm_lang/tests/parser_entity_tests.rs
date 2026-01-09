#[cfg(test)]
mod tests {
    use chrono::{Datelike, Offset, Timelike};
    use firm_lang::parser::dsl::{ParsedValue, parse_source};

    #[test]
    fn test_basic_entity_parsing() {
        let source = r#"
            person john_doe {
                name = "John Doe"
                age = 42
                active = true
            }
        "#;

        let parsed = parse_source(String::from(source), None).unwrap();
        let entities = parsed.entities();

        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].entity_type(), Some("person"));
        assert_eq!(entities[0].id(), Some("john_doe"));
    }

    #[test]
    fn test_multiline_string_with_intentional_indentation() {
        let source = r#"contact test {
            code = """
            function example() {
                if (true) {
                    return "nested";
                }
            }
            """
        }"#;
        let parsed = parse_source(String::from(source), None).unwrap();
        let entities = parsed.entities();
        let fields = entities[0].fields();
        let field = &fields[0];

        assert_eq!(field.id(), Some("code"));
        assert_eq!(
            field.value(),
            Ok(ParsedValue::String(
                "function example() {\n    if (true) {\n        return \"nested\";\n    }\n}"
                    .to_string()
            ))
        );
    }

    #[test]
    fn test_multiline_string_single_line() {
        let source = r#"contact test { note = """Just one line""" }"#;
        let parsed = parse_source(String::from(source), None).unwrap();
        let entities = parsed.entities();
        let fields = entities[0].fields();
        let field = &fields[0];

        assert_eq!(field.id(), Some("note"));
        assert_eq!(
            field.value(),
            Ok(ParsedValue::String("Just one line".to_string()))
        );
    }

    #[test]
    fn test_multiline_string_empty_lines() {
        let source = r#"contact test {
            text = """
            First paragraph

            Second paragraph after empty line
            """
        }"#;
        let parsed = parse_source(String::from(source), None).unwrap();
        let entities = parsed.entities();
        let fields = entities[0].fields();
        let field = &fields[0];

        assert_eq!(field.id(), Some("text"));
        assert_eq!(
            field.value(),
            Ok(ParsedValue::String(
                "First paragraph\n\nSecond paragraph after empty line".to_string()
            ))
        );
    }

    #[test]
    fn test_mixed_string_types() {
        let source = r#"contact test {
            name = "John Doe"
            bio = """
            Software engineer
            Based in San Francisco
            """
        }"#;
        let parsed = parse_source(String::from(source), None).unwrap();
        let entities = parsed.entities();
        let fields = entities[0].fields();

        assert_eq!(
            fields[0].value(),
            Ok(ParsedValue::String("John Doe".to_string()))
        );
        assert_eq!(
            fields[1].value(),
            Ok(ParsedValue::String(
                "Software engineer\nBased in San Francisco".to_string()
            ))
        );
    }

    #[test]
    fn test_number_fields() {
        let source = r#"contact test { age = 42, height = 5.9 }"#;
        let parsed = parse_source(String::from(source), None).unwrap();

        assert_eq!(
            parsed.entities()[0].fields()[0].value(),
            Ok(ParsedValue::Integer(42))
        );
        assert_eq!(
            parsed.entities()[0].fields()[1].value(),
            Ok(ParsedValue::Float(5.9))
        );
    }

    #[test]
    fn test_boolean_fields() {
        let source = r#"contact test { active = true, verified = false }"#;
        let parsed = parse_source(String::from(source), None).unwrap();

        assert_eq!(
            parsed.entities()[0].fields()[0].value(),
            Ok(ParsedValue::Boolean(true))
        );
        assert_eq!(
            parsed.entities()[0].fields()[1].value(),
            Ok(ParsedValue::Boolean(false))
        );
    }

    #[test]
    fn test_currency_decimal_amount() {
        let source = r#"contact test {
            salary = 75000.50 USD
            bonus = 5000.00 EUR
        }"#;
        let parsed = parse_source(String::from(source), None).unwrap();
        let entities = parsed.entities();
        let fields = entities[0].fields();

        if let Ok(ParsedValue::Currency { amount, currency }) = &fields[0].value() {
            assert_eq!(amount.to_string(), "75000.50");
            assert_eq!(currency.code(), "USD");
        } else {
            panic!("Expected currency value");
        }
        if let Ok(ParsedValue::Currency { amount, currency }) = &fields[1].value() {
            assert_eq!(amount.to_string(), "5000.00");
            assert_eq!(currency.code(), "EUR");
        } else {
            panic!("Expected currency value");
        }
    }

    #[test]
    fn test_currency_integer_amount() {
        let source = r#"contact test {
            salary = 50000 USD
            budget = 1000 DKK
        }"#;
        let parsed = parse_source(String::from(source), None).unwrap();
        let entities = parsed.entities();
        let fields = entities[0].fields();

        if let Ok(ParsedValue::Currency { amount, currency }) = &fields[0].value() {
            assert_eq!(amount.to_string(), "50000");
            assert_eq!(currency.code(), "USD");
        } else {
            panic!("Expected currency value");
        }
        if let Ok(ParsedValue::Currency { amount, currency }) = &fields[1].value() {
            assert_eq!(amount.to_string(), "1000");
            assert_eq!(currency.code(), "DKK");
        } else {
            panic!("Expected currency value");
        }
    }

    #[test]
    fn test_multiple_currency_fields() {
        let source = r#"project budget_2023 {
            total_budget = 100000 USD
            marketing_spend = 25000.50 USD
            r_and_d = 40000 USD
            operations = 15000.25 USD
            contingency = 19999.25 USD
        }"#;

        let parsed = parse_source(String::from(source), None).unwrap();
        let entities = parsed.entities();
        assert_eq!(entities.len(), 1);

        let fields = entities[0].fields();
        assert_eq!(fields.len(), 5);

        for field in fields {
            match field.value() {
                Ok(ParsedValue::Currency {
                    amount: _,
                    currency,
                }) => {
                    assert_eq!(currency.code(), "USD");
                }
                _ => panic!("Expected currency value"),
            }
        }
    }

    #[test]
    fn test_entity_references() {
        let source = r#"contact test {
            manager = contact.john_doe
            company = organization.acme_corp
        }"#;
        let parsed = parse_source(String::from(source), None).unwrap();
        let entities = parsed.entities();
        let fields = entities[0].fields();

        assert_eq!(
            fields[0].value(),
            Ok(ParsedValue::EntityReference {
                entity_type: "contact".to_string(),
                entity_id: "john_doe".to_string()
            })
        );
        assert_eq!(
            fields[1].value(),
            Ok(ParsedValue::EntityReference {
                entity_type: "organization".to_string(),
                entity_id: "acme_corp".to_string()
            })
        );
    }

    #[test]
    fn test_field_references() {
        let source = r#"contact test {
            title = contact.john_doe.title
            department = organization.acme_corp.department
        }"#;
        let parsed = parse_source(String::from(source), None).unwrap();
        let entities = parsed.entities();
        let fields = entities[0].fields();

        assert_eq!(
            fields[0].value(),
            Ok(ParsedValue::FieldReference {
                entity_type: "contact".to_string(),
                entity_id: "john_doe".to_string(),
                field_id: "title".to_string()
            })
        );
        assert_eq!(
            fields[1].value(),
            Ok(ParsedValue::FieldReference {
                entity_type: "organization".to_string(),
                entity_id: "acme_corp".to_string(),
                field_id: "department".to_string()
            })
        );
    }

    #[test]
    fn test_simple_lists() {
        let source = r#"contact test {
            tags = ["work", "important", "urgent"]
            scores = [85, 92, 78]
            flags = [true, false, true]
        }"#;
        let parsed = parse_source(String::from(source), None).unwrap();
        let entities = parsed.entities();
        let fields = entities[0].fields();

        assert_eq!(
            fields[0].value(),
            Ok(ParsedValue::List(vec![
                ParsedValue::String("work".to_string()),
                ParsedValue::String("important".to_string()),
                ParsedValue::String("urgent".to_string())
            ]))
        );
        assert_eq!(
            fields[1].value(),
            Ok(ParsedValue::List(vec![
                ParsedValue::Integer(85),
                ParsedValue::Integer(92),
                ParsedValue::Integer(78)
            ]))
        );
        assert_eq!(
            fields[2].value(),
            Ok(ParsedValue::List(vec![
                ParsedValue::Boolean(true),
                ParsedValue::Boolean(false),
                ParsedValue::Boolean(true)
            ]))
        );
    }

    #[test]
    fn test_empty_lists() {
        let source = r#"contact test { tags = [] }"#;
        let parsed = parse_source(String::from(source), None).unwrap();

        assert_eq!(
            parsed.entities()[0].fields()[0].value(),
            Ok(ParsedValue::List(vec![]))
        );
    }

    #[test]
    fn test_nested_lists() {
        let source = r#"contact test {
            matrix = [[1, 2, 3], [4, 5, 6]]
        }"#;
        let parsed = parse_source(String::from(source), None).unwrap();
        let entities = parsed.entities();
        let fields = entities[0].fields();

        assert_eq!(
            fields[0].value(),
            Ok(ParsedValue::List(vec![
                ParsedValue::List(vec![
                    ParsedValue::Integer(1),
                    ParsedValue::Integer(2),
                    ParsedValue::Integer(3)
                ]),
                ParsedValue::List(vec![
                    ParsedValue::Integer(4),
                    ParsedValue::Integer(5),
                    ParsedValue::Integer(6)
                ])
            ]))
        );
    }

    #[test]
    fn test_date_fields() {
        let source = r#"contact test {
            birthday = 1990-05-15
            start_date = 2023-01-01
        }"#;
        let parsed = parse_source(String::from(source), None).unwrap();
        let entities = parsed.entities();
        let fields = entities[0].fields();

        if let Ok(ParsedValue::DateTime(dt)) = &fields[0].value() {
            assert_eq!(dt.naive_local().date().year(), 1990);
            assert_eq!(dt.naive_local().date().month(), 5);
            assert_eq!(dt.naive_local().date().day(), 15);
        } else {
            panic!("Expected date value");
        }

        if let Ok(ParsedValue::DateTime(dt)) = &fields[1].value() {
            assert_eq!(dt.naive_local().date().year(), 2023);
            assert_eq!(dt.naive_local().date().month(), 1);
            assert_eq!(dt.naive_local().date().day(), 1);
        } else {
            panic!("Expected date value");
        }
    }

    #[test]
    fn test_multiple_date_fields() {
        let source = r#"project timeline {
            start_date = 2023-01-15
            phase_1_end = 2023-03-31
            phase_2_end = 2023-06-30
            final_deadline = 2023-12-31
            review_date = 2024-01-15
        }"#;

        let parsed = parse_source(String::from(source), None).unwrap();
        let entities = parsed.entities();
        assert_eq!(entities.len(), 1);

        let fields = entities[0].fields();
        assert_eq!(fields.len(), 5);

        // All fields should be dates
        for field in fields {
            assert!(matches!(field.value(), Ok(ParsedValue::DateTime(_))));
        }
    }

    #[test]
    fn test_datetime_local_timezone() {
        let source = r#"contact test {
            meeting = 2025-01-15 at 14:30 UTC+2
            deadline = 2025-02-28 at 23:59 UTC-5
        }"#;
        let parsed = parse_source(String::from(source), None).unwrap();
        let entities = parsed.entities();
        let fields = entities[0].fields();

        if let Ok(ParsedValue::DateTime(dt)) = &fields[0].value() {
            assert_eq!(dt.naive_local().date().year(), 2025);
            assert_eq!(dt.naive_local().date().month(), 1);
            assert_eq!(dt.naive_local().date().day(), 15);
            assert_eq!(dt.naive_local().time().hour(), 14);
            assert_eq!(dt.naive_local().time().minute(), 30);
            assert_eq!(dt.offset().fix().local_minus_utc(), 2 * 3600);
        } else {
            panic!("Expected datetime value");
        }

        if let Ok(ParsedValue::DateTime(dt)) = &fields[1].value() {
            assert_eq!(dt.naive_local().date().year(), 2025);
            assert_eq!(dt.naive_local().date().month(), 2);
            assert_eq!(dt.naive_local().date().day(), 28);
            assert_eq!(dt.naive_local().time().hour(), 23);
            assert_eq!(dt.naive_local().time().minute(), 59);
            assert_eq!(dt.offset().fix().local_minus_utc(), -5 * 3600);
        } else {
            panic!("Expected datetime value");
        }
    }

    #[test]
    fn test_datetime_single_digit_hour() {
        let source = r#"contact test {
            meeting = 2025-01-15 at 9:30
        }"#;
        let parsed = parse_source(String::from(source), None).unwrap();
        let entities = parsed.entities();
        let fields = entities[0].fields();

        if let Ok(ParsedValue::DateTime(dt)) = &fields[0].value() {
            assert_eq!(dt.naive_local().date().year(), 2025);
            assert_eq!(dt.naive_local().date().month(), 1);
            assert_eq!(dt.naive_local().date().day(), 15);
            assert_eq!(dt.naive_local().time().hour(), 9);
            assert_eq!(dt.naive_local().time().minute(), 30);
        } else {
            panic!("Expected datetime value");
        }
    }

    #[test]
    fn test_datetime_utc_negative_offset() {
        let source = r#"contact test {
            meeting = 2025-01-15 at 14:30 UTC-7
        }"#;
        let parsed = parse_source(String::from(source), None).unwrap();
        let entities = parsed.entities();
        let fields = entities[0].fields();

        if let Ok(ParsedValue::DateTime(dt)) = &fields[0].value() {
            assert_eq!(dt.offset().fix().local_minus_utc(), -7 * 3600);
        } else {
            panic!("Expected datetime value");
        }
    }

    #[test]
    fn test_datetime_utc_positive_offset() {
        let source = r#"contact test {
            meeting = 2025-01-15 at 14:30 UTC+3
        }"#;
        let parsed = parse_source(String::from(source), None).unwrap();
        let entities = parsed.entities();
        let fields = entities[0].fields();

        if let Ok(ParsedValue::DateTime(dt)) = &fields[0].value() {
            assert_eq!(dt.offset().fix().local_minus_utc(), 3 * 3600);
        } else {
            panic!("Expected datetime value");
        }
    }

    #[test]
    fn test_datetime_utc() {
        let source = r#"contact test {
            meeting = 2025-01-15 at 14:30 UTC
        }"#;
        let parsed = parse_source(String::from(source), None).unwrap();
        let entities = parsed.entities();
        let fields = entities[0].fields();

        if let Ok(ParsedValue::DateTime(dt)) = &fields[0].value() {
            assert_eq!(dt.offset().fix().local_minus_utc(), 0);
        } else {
            panic!("Expected datetime value");
        }
    }

    #[test]
    fn test_heterogeneous_list_error() {
        let source = r#"contact test {
            mixed = ["string", 42]
        }"#;
        let parsed = parse_source(String::from(source), None).unwrap();
        let entities = parsed.entities();
        let fields = entities[0].fields();

        match fields[0].value() {
            Err(_) => {
                // Expected error for heterogeneous list
            }
            Ok(_) => panic!("Expected error for heterogeneous list"),
        }
    }

    #[test]
    fn test_invalid_boolean_error() {
        let source = r#"contact test { flag = maybe }"#;
        let parsed = parse_source(String::from(source), None).unwrap();
        assert!(parsed.has_error());
    }

    #[test]
    fn test_invalid_integer_error() {
        let source = r#"contact test { age = 42abc }"#;
        let parsed = parse_source(String::from(source), None).unwrap();
        assert!(parsed.has_error());
    }

    #[test]
    fn test_invalid_float_error() {
        let source = r#"contact test { height = 5.9.2 }"#;
        let parsed = parse_source(String::from(source), None).unwrap();
        assert!(parsed.has_error());
    }

    #[test]
    fn test_invalid_currency_format_error() {
        let source = r#"contact test { salary = 50000USD }"#;
        let parsed = parse_source(String::from(source), None).unwrap();
        assert!(parsed.has_error());
    }

    #[test]
    fn test_invalid_currency_amount_error() {
        let source = r#"contact test { salary = abc USD }"#;
        let parsed = parse_source(String::from(source), None).unwrap();
        assert!(parsed.has_error());
    }

    #[test]
    fn test_invalid_currency_code_error() {
        let source = r#"contact test {
            salary = 50000 DOLLARS
            bonus = 5000 US
        }"#;
        let parsed = parse_source(String::from(source), None).unwrap();
        assert!(parsed.has_error());
    }

    #[test]
    fn test_invalid_reference_format_error() {
        let source = r#"contact test { ref = contact.too.many.parts.here }"#;
        let parsed = parse_source(String::from(source), None).unwrap();
        assert!(parsed.has_error());
    }

    #[test]
    fn test_invalid_date_error() {
        let source = r#"contact test {
            bad_date1 = 2023-13-01
            bad_date2 = 2023-02-30
        }"#;
        let parsed = parse_source(String::from(source), None).unwrap();
        let entities = parsed.entities();
        let fields = entities[0].fields();

        assert!(fields[0].value().is_err());
        assert!(fields[1].value().is_err());
    }

    #[test]
    fn test_invalid_datetime_error() {
        let source = r#"contact test {
            bad_datetime1 = 2023-13-01 at 14:30
            bad_datetime2 = 2023-01-01 at 25:30
        }"#;
        let parsed = parse_source(String::from(source), None).unwrap();
        let entities = parsed.entities();
        let fields = entities[0].fields();

        assert!(fields[0].value().is_err());
        assert!(fields[1].value().is_err());
    }

    #[test]
    fn test_invalid_timezone_error() {
        let source = r#"contact test {
            bad_tz = 2023-01-01 at 14:30 UTC+25
        }"#;
        let parsed = parse_source(String::from(source), None).unwrap();
        let entities = parsed.entities();
        let fields = entities[0].fields();

        assert!(fields[0].value().is_err());
    }
}
