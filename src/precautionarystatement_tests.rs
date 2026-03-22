#[cfg(test)]
mod tests {

    use crate::{init::populate_db_with_base_data, precautionarystatement::*};
    use chimitheque_types::requestfilter::RequestFilter;

    #[test]
    fn test_parse_precautionary_statement() {
        let mut db_connection = crate::test_utils::init_test();
        populate_db_with_base_data(&mut db_connection).unwrap();

        assert!(parse(&db_connection, "P243").is_ok_and(|u| u.is_some()));
        assert!(parse(&db_connection, "not exist").is_ok_and(|u| u.is_none()));
    }

    #[test]
    fn test_get_precautionary_statements() {
        let mut db_connection = crate::test_utils::init_test();
        populate_db_with_base_data(&mut db_connection).unwrap();

        let filter = RequestFilter {
            ..Default::default()
        };
        assert!(get_precautionary_statements(&db_connection, &filter,).is_ok());

        let filter = RequestFilter {
            search: Some(String::from("P24")),
            ..Default::default()
        };
        let (_, count) = get_precautionary_statements(&db_connection, &filter).unwrap();

        // expected number of results.
        assert_eq!(count, 5);
    }
}
