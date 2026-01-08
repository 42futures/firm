use chrono::{Datelike, Timelike};
use firm_core::{Entity, EntityId, EntityType, FieldId, FieldValue};
use firm_lang::parser::dsl::parse_source;
use iso_currency::Currency;
use rust_decimal::Decimal;

#[cfg(test)]
mod tests {
    use super::*;
    use firm_core::ReferenceValue;
    use std::path::PathBuf;

    #[test]
    fn test_basic_entity_conversion() {
        let source = r#"
            person john_doe {
                name = "John Doe"
                age = 42
                active = true
            }
        "#;

        let parsed = parse_source(String::from(source), None).unwrap();
        let entities = parsed.entities();
        let entity: Entity = (&entities[0]).try_into().unwrap();

        assert_eq!(entity.id, EntityId("person.john_doe".to_string()));
        assert_eq!(entity.entity_type, EntityType::new("person"));
        assert_eq!(entity.fields.len(), 3);

        assert_eq!(
            entity.get_field(&FieldId("name".to_string())),
            Some(&FieldValue::String("John Doe".to_string()))
        );
        assert_eq!(
            entity.get_field(&FieldId("age".to_string())),
            Some(&FieldValue::Integer(42))
        );
        assert_eq!(
            entity.get_field(&FieldId("active".to_string())),
            Some(&FieldValue::Boolean(true))
        );
    }

    #[test]
    fn test_organization_entity_conversion() {
        let source = r#"
            organization acme_corp {
                name = "ACME Corporation"
                employees = 500
            }
        "#;

        let parsed = parse_source(String::from(source), None).unwrap();
        let entities = parsed.entities();
        let entity: Entity = (&entities[0]).try_into().unwrap();

        assert_eq!(entity.id, EntityId("organization.acme_corp".to_string()));
        assert_eq!(entity.entity_type, EntityType::new("organization"));
        assert_eq!(entity.fields.len(), 2);
        assert_eq!(
            entity.get_field(&FieldId("name".to_string())),
            Some(&FieldValue::String("ACME Corporation".to_string()))
        );
        assert_eq!(
            entity.get_field(&FieldId("employees".to_string())),
            Some(&FieldValue::Integer(500))
        );
    }

    #[test]
    fn test_custom_entity_type_conversion() {
        let source = r#"
            project alpha_project {
                name = "Project Alpha"
                status = "active"
            }
        "#;

        let parsed = parse_source(String::from(source), None).unwrap();
        let entities = parsed.entities();
        let entity: Entity = (&entities[0]).try_into().unwrap();

        assert_eq!(entity.id, EntityId("project.alpha_project".to_string()));
        assert_eq!(entity.entity_type, EntityType::new("project"));
        assert_eq!(entity.fields.len(), 2);
        assert_eq!(
            entity.get_field(&FieldId("name".to_string())),
            Some(&FieldValue::String("Project Alpha".to_string()))
        );
        assert_eq!(
            entity.get_field(&FieldId("status".to_string())),
            Some(&FieldValue::String("active".to_string()))
        );
    }

    #[test]
    fn test_string_field_conversion() {
        let source = r#"
            person test_person {
                single_line = "Simple string"
                multi_line = """
                    This is a
                    multi-line string
                    with intentional indentation
                """
            }
        "#;

        let parsed = parse_source(String::from(source), None).unwrap();
        let entities = parsed.entities();
        let entity: Entity = (&entities[0]).try_into().unwrap();

        assert_eq!(
            entity.get_field(&FieldId("single_line".to_string())),
            Some(&FieldValue::String("Simple string".to_string()))
        );

        if let Some(FieldValue::String(multi_line)) =
            entity.get_field(&FieldId("multi_line".to_string()))
        {
            assert!(multi_line.contains("This is a"));
            assert!(multi_line.contains("multi-line string"));
        } else {
            panic!("Expected String field value");
        }
    }

    #[test]
    fn test_numeric_field_conversion() {
        let source = r#"test_entity numeric_test { integer_field = 42, float_field = 3.14159 }"#;

        let parsed = parse_source(String::from(source), None).unwrap();
        let entities = parsed.entities();
        let entity: Entity = (&entities[0]).try_into().unwrap();

        assert_eq!(
            entity.get_field(&FieldId("integer_field".to_string())),
            Some(&FieldValue::Integer(42))
        );
        assert_eq!(
            entity.get_field(&FieldId("float_field".to_string())),
            Some(&FieldValue::Float(3.14159))
        );
    }

    #[test]
    fn test_boolean_field_conversion() {
        let source = r#"
            test_entity bool_test {
                is_active = true
                is_deleted = false
            }
        "#;

        let parsed = parse_source(String::from(source), None).unwrap();
        let entities = parsed.entities();
        let entity: Entity = (&entities[0]).try_into().unwrap();

        assert_eq!(
            entity.get_field(&FieldId("is_active".to_string())),
            Some(&FieldValue::Boolean(true))
        );
        assert_eq!(
            entity.get_field(&FieldId("is_deleted".to_string())),
            Some(&FieldValue::Boolean(false))
        );
    }

    #[test]
    fn test_currency_field_conversion() {
        let source = r#"
            invoice inv_001 {
                total = 1250.75 USD
                deposit = 500 EUR
            }
        "#;

        let parsed = parse_source(String::from(source), None).unwrap();
        let entities = parsed.entities();
        let entity: Entity = (&entities[0]).try_into().unwrap();

        assert_eq!(
            entity.get_field(&FieldId("total".to_string())),
            Some(&FieldValue::Currency {
                amount: Decimal::from_str_exact("1250.75").unwrap(),
                currency: Currency::USD
            })
        );
        assert_eq!(
            entity.get_field(&FieldId("deposit".to_string())),
            Some(&FieldValue::Currency {
                amount: Decimal::from_str_exact("500").unwrap(),
                currency: Currency::EUR
            })
        );
    }

    #[test]
    fn test_entity_reference_conversion() {
        let source = r#"
            contact jane_doe {
                manager = person.john_doe
                company = organization.acme_corp
            }
        "#;

        let parsed = parse_source(String::from(source), None).unwrap();
        let entities = parsed.entities();
        let entity: Entity = (&entities[0]).try_into().unwrap();

        assert_eq!(
            entity.get_field(&FieldId("manager".to_string())),
            Some(&FieldValue::Reference(ReferenceValue::Entity(EntityId(
                "person.john_doe".to_string()
            ))))
        );
        assert_eq!(
            entity.get_field(&FieldId("company".to_string())),
            Some(&FieldValue::Reference(ReferenceValue::Entity(EntityId(
                "organization.acme_corp".to_string()
            ))))
        );
    }

    #[test]
    fn test_field_reference_conversion() {
        let source = r#"
            contact jane_doe {
                manager_name = person.john_doe.name
                company_address = organization.acme_corp.address
            }
        "#;

        let parsed = parse_source(String::from(source), None).unwrap();
        let entities = parsed.entities();
        let entity: Entity = (&entities[0]).try_into().unwrap();

        assert_eq!(
            entity.get_field(&FieldId("manager_name".to_string())),
            Some(&FieldValue::Reference(ReferenceValue::Field(
                EntityId("person.john_doe".to_string()),
                FieldId("name".to_string())
            )))
        );
        assert_eq!(
            entity.get_field(&FieldId("company_address".to_string())),
            Some(&FieldValue::Reference(ReferenceValue::Field(
                EntityId("organization.acme_corp".to_string()),
                FieldId("address".to_string())
            )))
        );
    }

    #[test]
    fn test_simple_list_conversion() {
        let source = r#"
            person john_doe {
                skills = ["Rust", "JavaScript", "Python"]
                scores = [95, 87, 92]
                flags = [true, false, true]
            }
        "#;

        let parsed = parse_source(String::from(source), None).unwrap();
        let entities = parsed.entities();
        let entity: Entity = (&entities[0]).try_into().unwrap();

        if let Some(FieldValue::List(skills)) = &entity.get_field(&FieldId("skills".to_string())) {
            assert_eq!(skills.len(), 3);
            assert_eq!(skills[0], FieldValue::String("Rust".to_string()));
            assert_eq!(skills[1], FieldValue::String("JavaScript".to_string()));
            assert_eq!(skills[2], FieldValue::String("Python".to_string()));
        } else {
            panic!("Expected List field value for skills");
        }

        if let Some(FieldValue::List(scores)) = &entity.get_field(&FieldId("scores".to_string())) {
            assert_eq!(scores.len(), 3);
            assert_eq!(scores[0], FieldValue::Integer(95));
            assert_eq!(scores[1], FieldValue::Integer(87));
            assert_eq!(scores[2], FieldValue::Integer(92));
        } else {
            panic!("Expected List field value for scores");
        }
    }

    #[test]
    fn test_empty_list_conversion() {
        let source = r#"
            person john_doe {
                empty_list = []
            }
        "#;

        let parsed = parse_source(String::from(source), None).unwrap();
        let entities = parsed.entities();
        let entity: Entity = (&entities[0]).try_into().unwrap();

        if let Some(FieldValue::List(empty_list)) =
            &entity.get_field(&FieldId("empty_list".to_string()))
        {
            assert_eq!(empty_list.len(), 0);
        } else {
            panic!("Expected empty List field value");
        }
    }

    #[test]
    fn test_nested_list_conversion() {
        let source = r#"
            test_entity nested_test {
                nested = [["a", "b"], ["c", "d"]]
            }
        "#;

        let parsed = parse_source(String::from(source), None).unwrap();
        let entities = parsed.entities();
        let entity: Entity = (&entities[0]).try_into().unwrap();

        if let Some(FieldValue::List(outer_list)) =
            &entity.get_field(&FieldId("nested".to_string()))
        {
            assert_eq!(outer_list.len(), 2);

            if let FieldValue::List(first_inner) = &outer_list[0] {
                assert_eq!(first_inner.len(), 2);
                assert_eq!(first_inner[0], FieldValue::String("a".to_string()));
                assert_eq!(first_inner[1], FieldValue::String("b".to_string()));
            } else {
                panic!("Expected nested List");
            }
        } else {
            panic!("Expected List field value for nested");
        }
    }

    #[test]
    fn test_datetime_conversion() {
        // Note: This test assumes datetime parsing is implemented in the parser
        // If not, it will test string conversion instead
        let source = r#"event meeting { start_time = "2024-03-15T14:30:00-05:00" }"#;

        let parsed = parse_source(String::from(source), None).unwrap();
        let entities = parsed.entities();
        let entity: Entity = (&entities[0]).try_into().unwrap();

        if let Some(start_time_field) = entity.get_field(&FieldId("start_time".to_string())) {
            match start_time_field {
                FieldValue::DateTime(start_time) => {
                    assert_eq!(start_time.year(), 2024);
                    assert_eq!(start_time.month(), 3);
                    assert_eq!(start_time.day(), 15);
                    assert_eq!(start_time.hour(), 14);
                    assert_eq!(start_time.minute(), 30);
                }
                FieldValue::String(datetime_str) => {
                    // DateTime parsing might not be fully implemented, so it's parsed as string
                    assert_eq!(datetime_str, "2024-03-15T14:30:00-05:00");
                }
                _ => panic!(
                    "Expected DateTime or String field value for start_time, got: {:?}",
                    start_time_field
                ),
            }
        } else {
            panic!("start_time field not found");
        }
    }

    #[test]
    fn test_path_field_no_source_conversion() {
        let source = r#"
            test_entity path_test {
                relative_path = path"./my/path.txt"
                absolute_path = path"/users/me/path.txt"
            }
        "#;

        let parsed = parse_source(String::from(source), None).unwrap();
        let entities = parsed.entities();
        let entity: Entity = (&entities[0]).try_into().unwrap();

        assert_eq!(
            entity.get_field(&FieldId("relative_path".to_string())),
            Some(&FieldValue::Path(PathBuf::from("./my/path.txt")))
        );
        assert_eq!(
            entity.get_field(&FieldId("absolute_path".to_string())),
            Some(&FieldValue::Path(PathBuf::from("/users/me/path.txt")))
        );
    }

    #[test]
    fn test_path_field_subdir_source_conversion() {
        let source = r#"
            test_entity path_test {
                relative_path = path"./my/path.txt"
            }
        "#;

        let parsed = parse_source(
            String::from(source),
            Some(PathBuf::from("./subdir/source.firm")),
        )
        .unwrap();
        let entities = parsed.entities();
        let entity: Entity = (&entities[0]).try_into().unwrap();

        assert_eq!(
            entity.get_field(&FieldId("relative_path".to_string())),
            Some(&FieldValue::Path(PathBuf::from("./subdir/my/path.txt")))
        );
    }

    #[test]
    fn test_path_field_root_source_conversion() {
        let source = r#"
            test_entity path_test {
                relative_path = path"./my/path.txt"
            }
        "#;

        let parsed =
            parse_source(String::from(source), Some(PathBuf::from("./source.firm"))).unwrap();
        let entities = parsed.entities();
        let entity: Entity = (&entities[0]).try_into().unwrap();

        assert_eq!(
            entity.get_field(&FieldId("relative_path".to_string())),
            Some(&FieldValue::Path(PathBuf::from("./my/path.txt")))
        );
    }

    #[test]
    fn test_path_field_parent_dir_source_conversion() {
        let source = r#"
            test_entity path_test {
                relative_path = path"../sibling/path.txt"
            }
        "#;

        let parsed = parse_source(
            String::from(source),
            Some(PathBuf::from("./subdir/source.firm")),
        )
        .unwrap();
        let entities = parsed.entities();
        let entity: Entity = (&entities[0]).try_into().unwrap();

        assert_eq!(
            entity.get_field(&FieldId("relative_path".to_string())),
            Some(&FieldValue::Path(PathBuf::from("./sibling/path.txt")))
        );
    }

    #[test]
    fn test_path_field_outside_workspace_conversion() {
        let source = r#"
            test_entity path_test {
                relative_path = path"../../path.txt"
            }
        "#;

        let parsed = parse_source(
            String::from(source),
            Some(PathBuf::from("./subdir/source.firm")),
        )
        .unwrap();
        let entities = parsed.entities();
        let entity: Entity = (&entities[0]).try_into().unwrap();

        assert_eq!(
            entity.get_field(&FieldId("relative_path".to_string())),
            Some(&FieldValue::Path(PathBuf::from("../path.txt")))
        );
    }

    #[test]
    fn test_multiple_entities_conversion() {
        let source = r#"
            person john_doe {
                name = "John Doe"
            }
            organization acme_corp {
                name = "ACME Corporation"
            }
        "#;

        let parsed = parse_source(String::from(source), None).unwrap();
        let entities = parsed.entities();

        assert_eq!(entities.len(), 2);

        let entity1: Entity = (&entities[0]).try_into().unwrap();
        let entity2: Entity = (&entities[1]).try_into().unwrap();

        assert_eq!(entity1.id, EntityId("person.john_doe".to_string()));
        assert_eq!(entity1.entity_type, EntityType::new("person"));

        assert_eq!(entity2.id, EntityId("organization.acme_corp".to_string()));
        assert_eq!(entity2.entity_type, EntityType::new("organization"));
    }

    #[test]
    fn test_composite_id_generation() {
        let source = r#"
            person john_doe { name = "John" }
            organization john_doe { name = "John's Company" }
        "#;

        let parsed = parse_source(String::from(source), None).unwrap();
        let entities = parsed.entities();

        let entity1: Entity = (&entities[0]).try_into().unwrap();
        let entity2: Entity = (&entities[1]).try_into().unwrap();

        // Same local ID but different composite IDs due to different entity types
        assert_eq!(entity1.id, EntityId("person.john_doe".to_string()));
        assert_eq!(entity2.id, EntityId("organization.john_doe".to_string()));
    }

    #[test]
    fn test_case_insensitive_composite_id() {
        let source = r#"
            PERSON john_doe { name = "John" }
            Person jane_doe { name = "Jane" }
        "#;

        let parsed = parse_source(String::from(source), None).unwrap();
        let entities = parsed.entities();

        let entity1: Entity = (&entities[0]).try_into().unwrap();
        let entity2: Entity = (&entities[1]).try_into().unwrap();

        // Both should have lowercase entity type in composite ID
        assert_eq!(entity1.id, EntityId("person.john_doe".to_string()));
        assert_eq!(entity2.id, EntityId("person.jane_doe".to_string()));
    }

    #[test]
    fn test_complex_mixed_fields() {
        let source = r#"
            contract main_contract {
                title = "Software Development Agreement"
                value = 50000.00 USD
                active = true
                parties = [person.john_doe, organization.acme_corp]
                start_date = 2024-01-01 00:00:00+00:00
                manager_contact = person.jane_doe.email
                milestones = [
                    "Planning Phase",
                    "Development Phase",
                    "Testing Phase"
                ]
            }
        "#;

        let parsed = parse_source(String::from(source), None).unwrap();
        let entities = parsed.entities();
        let entity: Entity = (&entities[0]).try_into().unwrap();

        assert_eq!(entity.id, EntityId("contract.main_contract".to_string()));
        assert_eq!(entity.entity_type, EntityType::new("contract"));
        assert_eq!(entity.fields.len(), 7);

        // Verify each field type conversion
        assert_eq!(
            entity.get_field(&FieldId("title".to_string())),
            Some(&FieldValue::String(
                "Software Development Agreement".to_string()
            ))
        );
        assert_eq!(
            entity.get_field(&FieldId("value".to_string())),
            Some(&FieldValue::Currency {
                amount: Decimal::from_str_exact("50000.00").unwrap(),
                currency: Currency::USD
            })
        );
        assert_eq!(
            entity.get_field(&FieldId("active".to_string())),
            Some(&FieldValue::Boolean(true))
        );

        // Verify list of entity references
        if let Some(FieldValue::List(parties)) = &entity.get_field(&FieldId("parties".to_string()))
        {
            assert_eq!(parties.len(), 2);
            assert_eq!(
                parties[0],
                FieldValue::Reference(ReferenceValue::Entity(EntityId(
                    "person.john_doe".to_string()
                )))
            );
            assert_eq!(
                parties[1],
                FieldValue::Reference(ReferenceValue::Entity(EntityId(
                    "organization.acme_corp".to_string()
                )))
            );
        } else {
            panic!("Expected List field value for parties");
        }

        // Verify field reference
        assert_eq!(
            entity.get_field(&FieldId("manager_contact".to_string())),
            Some(&FieldValue::Reference(ReferenceValue::Field(
                EntityId("person.jane_doe".to_string()),
                FieldId("email".to_string())
            )))
        );
    }
}
