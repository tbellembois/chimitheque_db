#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::too_many_lines
    )]

    use crate::{hazardstatement::*, init::populate_db_with_base_data};
    use chimitheque_types::requestfilter::RequestFilter;

    #[test]
    fn test_parse_hazard_statement() {
        let mut db_connection = crate::test_utils::init_test();
        populate_db_with_base_data(&mut db_connection).unwrap();

        assert!(parse(&db_connection, "H301").is_ok_and(|u| u.is_some()));
        assert!(parse(&db_connection, "not exist").is_ok_and(|u| u.is_none()));
    }

    #[test]
    fn test_get_hazard_statements() {
        let mut db_connection = crate::test_utils::init_test();
        populate_db_with_base_data(&mut db_connection).unwrap();

        let filter = RequestFilter {
            ..Default::default()
        };
        assert!(get_hazard_statements(&db_connection, &filter,).is_ok());

        let filter = RequestFilter {
            search: Some(String::from("H20")),
            ..Default::default()
        };
        let (_, count) = get_hazard_statements(&db_connection, &filter).unwrap();

        // expected number of results.
        assert_eq!(count, 10);
    }
}
