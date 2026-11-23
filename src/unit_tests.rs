#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use crate::unit::*;
    use chimitheque_defines::UNITS;
    use chimitheque_types::unittype::UnitType;
    use std::str::FromStr;

    // The units of the base data (`UNITS`) matching `keep`, in the order of the
    // list: by type, then by label.
    fn sorted_unit_labels(keep: impl Fn(&str, &str) -> bool) -> Vec<&'static str> {
        let mut units: Vec<(&str, &str)> = UNITS
            .iter()
            .map(|(_, label, _, unit_type, _)| (*unit_type, *label))
            .filter(|(unit_type, label)| keep(unit_type, label))
            .collect();
        units.sort_unstable();
        units.into_iter().map(|(_, label)| label).collect()
    }

    fn labels(units: &[UnitStruct]) -> Vec<&str> {
        units.iter().map(|unit| unit.unit_label.as_str()).collect()
    }

    // LIKE ignores the ASCII case.
    fn like(label: &str, search: &str) -> bool {
        label
            .to_ascii_lowercase()
            .contains(&search.to_ascii_lowercase())
    }

    #[test]
    fn test_parse_existing_units() {
        let db = crate::test_utils::init_test_with_fixture();

        for (id, label, multiplier, unit_type, parent_id) in UNITS {
            let unit = parse(&db, label).unwrap().unwrap();
            assert_eq!(unit.unit_id, Some(u64::try_from(*id).unwrap()), "{label}");
            assert_eq!(unit.unit_label, label, "{label}");
            assert!(
                (unit.unit_multiplier - multiplier).abs() < f64::EPSILON,
                "{label}"
            );
            assert_eq!(
                unit.unit_type,
                UnitType::from_str(unit_type).unwrap(),
                "{label}"
            );
            assert_eq!(
                unit.unit.and_then(|parent| parent.unit_id),
                parent_id.map(|parent_id| u64::try_from(parent_id).unwrap()),
                "{label}"
            );
        }
    }

    #[test]
    fn test_parse_nonexisting_unit() {
        let db = crate::test_utils::init_test_with_fixture();

        assert!(parse(&db, "nonexisting_unit").unwrap().is_none());
    }

    #[test]
    fn test_parse_case_sensitivity() {
        let db = crate::test_utils::init_test_with_fixture();

        assert_eq!(parse(&db, "L").unwrap().unwrap().unit_label, "L");
        assert!(parse(&db, "l").unwrap().is_none());
    }

    #[test]
    fn test_get_units_with_no_filter() {
        let db = crate::test_utils::init_test_with_fixture();

        let (units, count) = get_units(&db, RequestFilter::default()).unwrap();

        assert_eq!(count, UNITS.len());
        assert_eq!(labels(&units), sorted_unit_labels(|_, _| true));
    }

    #[test]
    fn test_get_units_with_search_filter() {
        let db = crate::test_utils::init_test_with_fixture();

        let filter = RequestFilter {
            search: Some("m".to_string()),
            ..Default::default()
        };
        let (units, count) = get_units(&db, filter).unwrap();

        let expected = sorted_unit_labels(|_, label| like(label, "m"));
        assert!(expected.contains(&"mM"));
        assert_eq!(count, expected.len());
        assert_eq!(labels(&units), expected);
    }

    #[test]
    fn test_get_units_with_unit_type_filter() {
        let db = crate::test_utils::init_test_with_fixture();

        for unit_type in [
            UnitType::Quantity,
            UnitType::Concentration,
            UnitType::Temperature,
            UnitType::MolecularWeight,
        ] {
            let filter = RequestFilter {
                unit_type: Some(unit_type.to_string()),
                ..Default::default()
            };
            let (units, count) = get_units(&db, filter).unwrap();

            let expected = sorted_unit_labels(|this_type, _| this_type == unit_type.to_string());
            assert_eq!(count, expected.len(), "{unit_type}");
            assert_eq!(labels(&units), expected, "{unit_type}");
            assert!(units.iter().all(|unit| unit.unit_type == unit_type));
        }
    }

    #[test]
    fn test_get_units_with_unicode_characters() {
        let db = crate::test_utils::init_test_with_fixture();

        for (search, expected) in [
            ("µ", vec!["µL", "µg", "µM", "µg/L"]),
            ("°", vec!["°C", "°F"]),
        ] {
            let filter = RequestFilter {
                search: Some(search.to_string()),
                ..Default::default()
            };
            let (units, count) = get_units(&db, filter).unwrap();

            let mut found = labels(&units);
            found.sort_unstable();
            let mut expected = expected;
            expected.sort_unstable();
            assert_eq!(count, expected.len(), "{search}");
            assert_eq!(found, expected, "{search}");
        }
    }

    #[test]
    fn test_get_units_with_limit_and_offset() {
        let db = crate::test_utils::init_test_with_fixture();
        let all = sorted_unit_labels(|_, _| true);

        let filter = RequestFilter {
            limit: Some(5),
            ..Default::default()
        };
        let (units, count) = get_units(&db, filter).unwrap();
        // The count ignores the limit and offset.
        assert_eq!(count, UNITS.len());
        assert_eq!(labels(&units), all[..5]);

        let filter = RequestFilter {
            limit: Some(3),
            offset: Some(5),
            ..Default::default()
        };
        let (units, count) = get_units(&db, filter).unwrap();
        assert_eq!(count, UNITS.len());
        assert_eq!(labels(&units), all[5..8]);
    }

    #[test]
    fn test_get_units_with_combined_filters() {
        let db = crate::test_utils::init_test_with_fixture();

        let filter = RequestFilter {
            unit_type: Some(UnitType::Temperature.to_string()),
            search: Some("°C".to_string()),
            ..Default::default()
        };
        let (units, count) = get_units(&db, filter).unwrap();
        assert_eq!(count, 1);
        assert_eq!(labels(&units), ["°C"]);
        assert_eq!(units[0].unit_type, UnitType::Temperature);

        // The concentration units containing "/L": g/L, mg/L, ng/L, µg/L; the
        // second one.
        let filter = RequestFilter {
            unit_type: Some(UnitType::Concentration.to_string()),
            search: Some("/L".to_string()),
            limit: Some(1),
            offset: Some(1),
            ..Default::default()
        };
        let (units, count) = get_units(&db, filter).unwrap();
        let expected = sorted_unit_labels(|unit_type, label| {
            unit_type == "concentration" && like(label, "/L")
        });
        assert_eq!(count, expected.len());
        assert_eq!(labels(&units), expected[1..2]);
    }

    #[test]
    fn test_get_units_with_empty_search() {
        let db = crate::test_utils::init_test_with_fixture();

        let filter = RequestFilter {
            search: Some(String::new()),
            ..Default::default()
        };
        let (units, count) = get_units(&db, filter).unwrap();

        assert_eq!(count, UNITS.len());
        assert_eq!(units.len(), UNITS.len());
    }
}
