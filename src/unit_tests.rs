#[cfg(test)]
mod tests {

    use crate::unit::*;
    use chimitheque_types::unittype::UnitType;
    use rusqlite::Connection;

    fn init_test_units() -> Connection {
        let db = crate::test_utils::init_test();

        // Delete existing records if any
        db.execute("DELETE FROM unit", []).unwrap();

        // Insert test units
        db.execute(
            "INSERT INTO unit (unit_id, unit_label, unit_multiplier, unit_type) VALUES (1, 'L', 1, 'quantity')",
            [],
        ).unwrap();
        db.execute(
            "INSERT INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit) VALUES (2, 'mL', 0.001, 'quantity', 1)",
            [],
        ).unwrap();
        db.execute(
            "INSERT INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit) VALUES (3, 'µL', 1.0e-06, 'quantity', 1)",
            [],
        ).unwrap();
        db.execute(
            "INSERT INTO unit (unit_id, unit_label, unit_multiplier, unit_type) VALUES (4, '%', 1, 'concentration')",
            [],
        ).unwrap();
        db.execute(
            "INSERT INTO unit (unit_id, unit_label, unit_multiplier, unit_type) VALUES (5, 'g/mol', 1, 'molecular_weight')",
            [],
        ).unwrap();
        db.execute(
            "INSERT INTO unit (unit_id, unit_label, unit_multiplier, unit_type) VALUES (6, '°C', 1, 'temperature')",
            [],
        ).unwrap();
        db.execute(
            "INSERT INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit) VALUES (7, 'mg', 0.001, 'quantity', 1)",
            [],
        ).unwrap();
        db.execute(
            "INSERT INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit) VALUES (8, 'mM', 1, 'concentration', 2)",
            [],
        ).unwrap();
        db.execute(
            "INSERT INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit) VALUES (9, 'µM', 1.0e-03, 'concentration', 2)",
            [],
        ).unwrap();
        db.execute(
            "INSERT INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit) VALUES (10, '°F', 1, 'temperature', 6)",
            [],
        ).unwrap();
        db.execute(
            "INSERT INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit) VALUES (11, '°K', 1, 'temperature', 6)",
            [],
        ).unwrap();
        db.execute(
            "INSERT INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit) VALUES (12, 'µg', 1.0e-06, 'quantity', 1)",
            [],
        ).unwrap();
        db.execute(
            "INSERT INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit) VALUES (13, 'kg', 1000, 'quantity', 1)",
            [],
        ).unwrap();
        db.execute(
            "INSERT INTO unit (unit_id, unit_label, unit_multiplier, unit_type) VALUES (14, 'Unit with special ßcharâctérs', 1, 'quantity')",
            [],
        ).unwrap();

        db
    }

    #[test]
    fn test_parse_existing_units() {
        let db_connection = init_test_units();

        // Test base units
        let base_units = vec![
            ("L", UnitType::Quantity),
            ("%", UnitType::Concentration),
            ("°C", UnitType::Temperature),
            ("g/mol", UnitType::MolecularWeight),
        ];

        for (label, unit_type) in &base_units {
            let result = parse(&db_connection, label).unwrap().unwrap();
            assert_eq!(
                result.unit_label, *label,
                "Unit label doesn't match for {label}"
            );
            assert_eq!(
                result.unit_type, *unit_type,
                "Unit type doesn't match for {label}"
            );
            assert!(
                result.unit.is_none(),
                "Expected no parent for base unit {label}"
            );
        }

        // Test derived units with parents
        let derived_units = vec![
            ("mL", UnitType::Quantity, Some(1)),
            ("µL", UnitType::Quantity, Some(1)),
            ("mg", UnitType::Quantity, Some(1)),
            ("µg", UnitType::Quantity, Some(1)),
            ("kg", UnitType::Quantity, Some(1)),
            ("mM", UnitType::Concentration, Some(2)),
            ("µM", UnitType::Concentration, Some(2)),
            ("°F", UnitType::Temperature, Some(6)),
            ("°K", UnitType::Temperature, Some(6)),
        ];

        for (label, unit_type, parent_id) in &derived_units {
            let result = parse(&db_connection, label).unwrap().unwrap();
            assert_eq!(
                result.unit_label, *label,
                "Unit label doesn't match for {label}"
            );
            assert_eq!(
                result.unit_type, *unit_type,
                "Unit type doesn't match for {label}"
            );

            if let Some(parent) = &result.unit {
                assert_eq!(
                    parent.unit_id, *parent_id,
                    "Parent ID doesn't match for {label}"
                );
            } else {
                panic!("Expected parent for derived unit {label}");
            }
        }
    }

    #[test]
    fn test_parse_nonexistent_unit() {
        let db_connection = init_test_units();
        assert!(parse(&db_connection, "nonexistent_unit").is_ok_and(|u| u.is_none()));
    }

    #[test]
    fn test_parse_case_sensitivity() {
        let db_connection = init_test_units();

        // Test uppercase label
        let result = parse(&db_connection, "L").unwrap().unwrap();
        assert_eq!(result.unit_label, "L", "Uppercase label did not match");

        // Test lowercase label
        let result_lowercase = parse(&db_connection, "l").unwrap();
        assert!(
            result_lowercase.is_none(),
            "Lowercase label should not work"
        );
    }

    #[test]
    fn test_get_units_with_no_filter() {
        let db_connection = init_test_units();

        let filter = RequestFilter {
            ..Default::default()
        };

        let (units, count) = get_units(&db_connection, filter).unwrap();

        // Assert that we get all units from the test fixture
        assert_eq!(count, 14, "Expected 14 units in total with no filter");
        assert_eq!(
            units.len(),
            14,
            "Expected to receive all 14 units with no filter"
        );
    }

    #[test]
    fn test_get_units_with_search_filter() {
        let db_connection = init_test_units();

        // Test search for "m" (case insensitive)
        let filter = RequestFilter {
            search: Some("m".to_string()),
            ..Default::default()
        };

        let (units, count) = get_units(&db_connection, filter).unwrap();
        assert_eq!(count, 5, "Expected 5 units with 'm' in their label");
        assert_eq!(
            units.len(),
            5,
            "Expected to receive all 5 units with 'm' in their label"
        );

        let labels: Vec<String> = units.iter().map(|u| u.unit_label.clone()).collect();
        assert!(
            labels.contains(&"mL".to_string()),
            "Expected mL unit to be found"
        );
        assert!(
            labels.contains(&"mg".to_string()),
            "Expected mg unit to be found"
        );
        assert!(
            labels.contains(&"mM".to_string()),
            "Expected mM unit to be found"
        );
        assert!(
            labels.contains(&"µM".to_string()),
            "Expected µM unit to be found"
        );
        assert!(
            labels.contains(&"g/mol".to_string()),
            "Expected µM unit to be found"
        );
    }

    #[test]
    fn test_get_units_with_unit_type_filter() {
        let db_connection = init_test_units();

        // Test filter for Quantity units
        let filter = RequestFilter {
            unit_type: Some(UnitType::Quantity.to_string()),
            ..Default::default()
        };

        let (units, count) = get_units(&db_connection, filter).unwrap();
        assert_eq!(count, 7, "Expected 7 Quantity units in the test fixture");
        assert_eq!(units.len(), 7, "Expected to receive all 7 Quantity units");

        for unit in &units {
            assert_eq!(
                unit.unit_type,
                UnitType::Quantity,
                "Unit '{}' is not of type Quantity",
                unit.unit_label
            );
        }

        // Test filter for Concentration units
        let filter = RequestFilter {
            unit_type: Some(UnitType::Concentration.to_string()),
            ..Default::default()
        };

        let (units, count) = get_units(&db_connection, filter).unwrap();
        assert_eq!(
            count, 3,
            "Expected 3 Concentration units (%, mM, µM) in the test fixture"
        );
        assert_eq!(
            units.len(),
            3,
            "Expected to receive all 3 Concentration units"
        );

        for unit in &units {
            assert_eq!(
                unit.unit_type,
                UnitType::Concentration,
                "Unit '{}' is not of type Concentration",
                unit.unit_label
            );
        }

        // Test filter for Temperature units
        let filter = RequestFilter {
            unit_type: Some(UnitType::Temperature.to_string()),
            ..Default::default()
        };

        let (units, count) = get_units(&db_connection, filter).unwrap();
        assert_eq!(
            count, 3,
            "Expected 3 Temperature units (°C, °F, °K) in the test fixture"
        );
        assert_eq!(
            units.len(),
            3,
            "Expected to receive all 3 Temperature units"
        );

        for unit in &units {
            assert_eq!(
                unit.unit_type,
                UnitType::Temperature,
                "Unit '{}' is not of type Temperature",
                unit.unit_label
            );
        }

        // Test filter for MolecularWeight units
        let filter = RequestFilter {
            unit_type: Some(UnitType::MolecularWeight.to_string()),
            ..Default::default()
        };

        let (units, count) = get_units(&db_connection, filter).unwrap();
        assert_eq!(
            count, 1,
            "Expected 1 MolecularWeight unit (g/mol) in the test fixture"
        );
        assert_eq!(
            units.len(),
            1,
            "Expected to receive the MolecularWeight unit"
        );

        for unit in &units {
            assert_eq!(
                unit.unit_type,
                UnitType::MolecularWeight,
                "Unit '{}' is not of type MolecularWeight",
                unit.unit_label
            );
        }
    }

    #[test]
    fn test_get_units_with_special_characters() {
        let conn = init_test_units();

        // Test special character search
        let search_term = "ßcharâctérs";
        let filter = RequestFilter {
            search: Some(search_term.to_string()),
            ..Default::default()
        };
        let (units, count) = get_units(&conn, filter).unwrap();
        assert_eq!(count, 1);
        assert_eq!(units.len(), 1);
        assert!(units[0].unit_label.contains("ßcharâctérs"));

        // Test partial special character search
        let partial_search_term = "charâctérs";
        let filter = RequestFilter {
            search: Some(partial_search_term.to_string()),
            ..Default::default()
        };
        let (units, count) = get_units(&conn, filter).unwrap();
        assert_eq!(count, 1);
        assert_eq!(units.len(), 1);
        assert!(units[0].unit_label.contains(partial_search_term));
    }

    #[test]
    fn test_get_units_with_unicode_characters() {
        let conn = init_test_units();

        // Test edge case - Unicode characters
        let filter = RequestFilter {
            search: Some("ß".to_string()),
            ..Default::default()
        };
        let (units, count) = get_units(&conn, filter).unwrap();
        assert_eq!(count, 1);
        assert_eq!(units.len(), 1);
        assert!(units[0].unit_label.contains("ß"));
    }

    #[test]
    fn test_get_units_with_limit_and_offset() {
        let db_connection = init_test_units();

        // Test with limit = 5
        let filter = RequestFilter {
            limit: Some(5),
            ..Default::default()
        };

        let (units, count) = get_units(&db_connection, filter).unwrap();
        assert_eq!(count, 14, "Expected 14 units in total with no filter");
        assert_eq!(units.len(), 5, "Expected 5 units with limit of 5");

        // Verify that we're getting the first 5 units
        assert_eq!(
            units[0].unit_label, "%",
            "Expected % unit at position 0 when limit=5"
        );
        assert_eq!(
            units[1].unit_label, "mM",
            "Expected mM unit at position 1 when limit=5"
        );
        assert_eq!(
            units[2].unit_label, "µM",
            "Expected µM unit at position 2 when limit=5"
        );
        assert_eq!(
            units[3].unit_label, "g/mol",
            "Expected g/mol unit at position 4 when limit=5"
        );
        assert_eq!(
            units[4].unit_label, "L",
            "Expected L unit at position 3 when limit=5"
        );

        // Test with limit = 3 and offset = 5
        let filter = RequestFilter {
            limit: Some(3),
            offset: Some(5),
            ..Default::default()
        };

        let (units, count) = get_units(&db_connection, filter).unwrap();
        assert_eq!(count, 14, "Expected 14 units in total with no filter");
        assert_eq!(
            units.len(),
            3,
            "Expected 3 units with limit of 3 and offset of 5"
        );

        // Verify the results represent the correct units
        assert_eq!(
            units[0].unit_label, "Unit with special ßcharâctérs",
            "Expected Unit with special ßcharâctérs unit at position 0 with offset of 5 and limit of 3"
        );
        assert_eq!(
            units[1].unit_label, "kg",
            "Expected kg unit at position 1 with offset of 5 and limit of 3"
        );
        assert_eq!(
            units[2].unit_label, "mL",
            "Expected mL unit at position 2 with offset of 5 and limit of 3"
        );
    }

    #[test]
    fn test_get_units_with_combined_filters() {
        let db_connection = init_test_units();

        // Test with unit_type filter and search filter
        let filter = RequestFilter {
            unit_type: Some(UnitType::Temperature.to_string()),
            search: Some("°C".to_string()),
            ..Default::default()
        };

        let (units, count) = get_units(&db_connection, filter).unwrap();
        assert_eq!(count, 1, "Expected 1 Temperature unit with °C in its label");
        assert_eq!(units.len(), 1, "Expected to receive the °C unit");
        assert_eq!(units[0].unit_label, "°C", "Expected unit to be °C");
        assert_eq!(
            units[0].unit_type,
            UnitType::Temperature,
            "Expected unit type to be Temperature"
        );

        // Test with search, unit_type, limit, and offset filters
        let filter = RequestFilter {
            unit_type: Some(UnitType::Concentration.to_string()),
            search: Some("M".to_string()),
            limit: Some(1),
            offset: Some(1),
            ..Default::default()
        };

        let (units, count) = get_units(&db_connection, filter).unwrap();
        assert_eq!(
            count, 2,
            "Expected 2 Concentration units with 'M' in their label (mM and µM)"
        );
        assert_eq!(
            units.len(),
            1,
            "Expected to receive 1 unit with limit=1 and offset=1"
        );
        assert_eq!(
            units[0].unit_label, "µM",
            "Expected unit to be µM with offset=1"
        );
        assert_eq!(
            units[0].unit_type,
            UnitType::Concentration,
            "Expected unit type to be Concentration"
        );
    }

    #[test]
    fn test_get_units_with_empty_search() {
        let db_connection = init_test_units();

        let filter = RequestFilter {
            search: Some(String::new()),
            ..Default::default()
        };

        let (units, count) = get_units(&db_connection, filter).unwrap();

        // Assert that we get all units when searching for an empty string
        assert_eq!(count, 14, "Expected 14 units with empty search");
        assert_eq!(
            units.len(),
            14,
            "Expected to receive all 14 units with empty search"
        );
    }
}
