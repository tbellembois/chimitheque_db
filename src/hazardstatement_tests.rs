#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use crate::hazardstatement::*;
    use chimitheque_types::requestfilter::RequestFilter;

    // The base data loads the statements of resources/ghscode_11.txt.
    const HAZARD_STATEMENTS: usize = 113;

    fn references(statements: &[HazardStatementStruct]) -> Vec<&str> {
        statements
            .iter()
            .map(|statement| statement.hazard_statement_reference.as_str())
            .collect()
    }

    #[test]
    fn test_parse_hazard_statement() {
        let db = crate::test_utils::init_test_with_fixture();

        let statement = parse(&db, "H225").unwrap().unwrap();
        assert_eq!(
            statement.hazard_statement_label,
            "Highly Flammable liquid and vapor"
        );
        // Suffixed and combined codes.
        assert!(parse(&db, "H350i").unwrap().is_some());
        assert!(parse(&db, "H315+H319").unwrap().is_some());
        // The reference must match exactly.
        assert!(parse(&db, "h225").unwrap().is_none());
        assert!(parse(&db, "H999").unwrap().is_none());
    }

    #[test]
    fn test_get_hazard_statements() {
        let db = crate::test_utils::init_test_with_fixture();

        let (statements, count) = get_hazard_statements(&db, &RequestFilter::default()).unwrap();
        assert_eq!(count, HAZARD_STATEMENTS);
        assert_eq!(statements.len(), HAZARD_STATEMENTS);
        // Sorted by reference.
        let found = references(&statements);
        let mut sorted = found.clone();
        sorted.sort_unstable();
        assert_eq!(found, sorted);
    }

    #[test]
    fn test_get_hazard_statements_with_search() {
        let db = crate::test_utils::init_test_with_fixture();

        let filter = RequestFilter {
            search: Some("H22".to_string()),
            ..Default::default()
        };
        let (statements, count) = get_hazard_statements(&db, &filter).unwrap();
        assert_eq!(count, statements.len());
        assert!(references(&statements).contains(&"H225"));
        assert!(
            references(&statements)
                .iter()
                .all(|reference| reference.contains("H22"))
        );
        assert!(
            statements
                .iter()
                .all(|statement| !statement.match_exact_search)
        );

        // An exact match comes first, flagged; the combined codes containing
        // it follow.
        let filter = RequestFilter {
            search: Some("H319".to_string()),
            ..Default::default()
        };
        let (statements, _) = get_hazard_statements(&db, &filter).unwrap();
        assert_eq!(statements[0].hazard_statement_reference, "H319");
        assert!(statements[0].match_exact_search);
        assert!(references(&statements).contains(&"H315+H319"));
        assert!(
            statements[1..]
                .iter()
                .all(|statement| !statement.match_exact_search)
        );
    }

    #[test]
    fn test_get_hazard_statements_with_limit_and_offset() {
        let db = crate::test_utils::init_test_with_fixture();
        let (all, _) = get_hazard_statements(&db, &RequestFilter::default()).unwrap();

        let filter = RequestFilter {
            limit: Some(3),
            offset: Some(10),
            ..Default::default()
        };
        let (statements, count) = get_hazard_statements(&db, &filter).unwrap();

        // The count ignores the limit and offset.
        assert_eq!(count, HAZARD_STATEMENTS);
        assert_eq!(references(&statements), references(&all[10..13]));
    }
}
