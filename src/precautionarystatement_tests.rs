#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use crate::precautionarystatement::*;
    use chimitheque_types::requestfilter::RequestFilter;

    // The base data loads the statements of resources/ghscode_11.txt.
    const PRECAUTIONARY_STATEMENTS: usize = 169;

    fn references(statements: &[PrecautionaryStatementStruct]) -> Vec<&str> {
        statements
            .iter()
            .map(|statement| statement.precautionary_statement_reference.as_str())
            .collect()
    }

    #[test]
    fn test_parse_precautionary_statement() {
        let db = crate::test_utils::init_test_with_fixture();

        let statement = parse(&db, "P210").unwrap().unwrap();
        assert_eq!(
            statement.precautionary_statement_label,
            "Keep away from heat, hot surface, sparks, open flames and other ignition sources. No smoking."
        );
        // Combined codes.
        assert!(parse(&db, "P301+P310").unwrap().is_some());
        assert!(parse(&db, "P305+P351+P338").unwrap().is_some());
        // With an optional last code in brackets.
        assert!(parse(&db, "P370+P380+P375[+P378]").unwrap().is_some());
        // The reference must match exactly.
        assert!(parse(&db, "p210").unwrap().is_none());
        assert!(parse(&db, "P999").unwrap().is_none());
    }

    #[test]
    fn test_get_precautionary_statements() {
        let db = crate::test_utils::init_test_with_fixture();

        let (statements, count) =
            get_precautionary_statements(&db, &RequestFilter::default()).unwrap();
        assert_eq!(count, PRECAUTIONARY_STATEMENTS);
        assert_eq!(statements.len(), PRECAUTIONARY_STATEMENTS);
        // Sorted by reference.
        let found = references(&statements);
        let mut sorted = found.clone();
        sorted.sort_unstable();
        assert_eq!(found, sorted);
    }

    #[test]
    fn test_get_precautionary_statements_with_search() {
        let db = crate::test_utils::init_test_with_fixture();

        let filter = RequestFilter {
            search: Some("P30".to_string()),
            ..Default::default()
        };
        let (statements, count) = get_precautionary_statements(&db, &filter).unwrap();
        assert_eq!(count, statements.len());
        assert!(references(&statements).contains(&"P305"));
        assert!(
            references(&statements)
                .iter()
                .all(|reference| reference.contains("P30"))
        );
        assert!(
            statements
                .iter()
                .all(|statement| !statement.match_exact_search)
        );

        // An exact match comes first, flagged; the combined codes containing
        // it follow.
        let filter = RequestFilter {
            search: Some("P305".to_string()),
            ..Default::default()
        };
        let (statements, _) = get_precautionary_statements(&db, &filter).unwrap();
        assert_eq!(statements[0].precautionary_statement_reference, "P305");
        assert!(statements[0].match_exact_search);
        assert!(references(&statements).contains(&"P305+P351+P338"));
        assert!(
            statements[1..]
                .iter()
                .all(|statement| !statement.match_exact_search)
        );
    }

    #[test]
    fn test_get_precautionary_statements_with_limit_and_offset() {
        let db = crate::test_utils::init_test_with_fixture();
        let (all, _) = get_precautionary_statements(&db, &RequestFilter::default()).unwrap();

        let filter = RequestFilter {
            limit: Some(3),
            offset: Some(10),
            ..Default::default()
        };
        let (statements, count) = get_precautionary_statements(&db, &filter).unwrap();

        // The count ignores the limit and offset.
        assert_eq!(count, PRECAUTIONARY_STATEMENTS);
        assert_eq!(references(&statements), references(&all[10..13]));
    }
}
