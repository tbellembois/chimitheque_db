#[cfg(test)]
mod tests {

    use crate::producer::*;

    fn init_test_producer() -> Connection {
        let db = crate::test_utils::init_test();

        // Disable synchronous operations and foreign key constraints for faster test execution
        db.execute("PRAGMA synchronous = OFF", []).unwrap();
        db.execute("PRAGMA foreign_keys = OFF", []).unwrap();

        // Enable foreign key constraints back
        db.execute("PRAGMA foreign_keys = ON", []).unwrap();

        db.execute(
            "INSERT INTO producer (producer_id, producer_label) VALUES (1, 'BASF')",
            [],
        )
        .unwrap();

        db.execute(
            "INSERT INTO producer (producer_id, producer_label) VALUES (2, 'Dow Chemical Company')",
            [],
        )
        .unwrap();

        db.execute(
            "INSERT INTO producer (producer_id, producer_label) VALUES (3, 'DuPont')",
            [],
        )
        .unwrap();

        db.execute(
            "INSERT INTO producer (producer_id, producer_label) VALUES (4, 'LyondellBasell')",
            [],
        )
        .unwrap();

        db.execute(
            "INSERT INTO producer (producer_id, producer_label) VALUES (5, 'Mitsubishi Chemical Holdings')",
            []
        ).unwrap();

        db.execute(
            "INSERT INTO producer (producer_id, producer_label) VALUES (6, 'SABIC')",
            [],
        )
        .unwrap();

        db.execute(
            "INSERT INTO producer (producer_id, producer_label) VALUES (7, 'INEOS')",
            [],
        )
        .unwrap();

        db.execute(
            "INSERT INTO producer (producer_id, producer_label) VALUES (8, 'Sumitomo Chemical')",
            [],
        )
        .unwrap();

        db.execute(
            "INSERT INTO producer (producer_id, producer_label) VALUES (9, 'Lanxess')",
            [],
        )
        .unwrap();

        db.execute(
            "INSERT INTO producer (producer_id, producer_label) VALUES (10, 'Solvay')",
            [],
        )
        .unwrap();

        // Enable foreign key constraints back
        db.execute("PRAGMA foreign_keys = ON", []).unwrap();

        db
    }

    #[test]
    fn test_get_producers_with_no_filter() {
        let conn = init_test_producer();

        // Test: Call the function with no filter
        let (producers, count) = get_producers(&conn, &RequestFilter::default()).unwrap();

        // Assert: Verify the results
        assert_eq!(count, 10);
        assert_eq!(producers[0].producer_label, "BASF");
        assert_eq!(producers[1].producer_label, "Dow Chemical Company");
        assert_eq!(producers[2].producer_label, "DuPont");
        assert_eq!(producers[3].producer_label, "INEOS");
        assert_eq!(producers[4].producer_label, "Lanxess");
        assert_eq!(producers[5].producer_label, "LyondellBasell");
        assert_eq!(producers[6].producer_label, "Mitsubishi Chemical Holdings");
        assert_eq!(producers[7].producer_label, "SABIC");
        assert_eq!(producers[8].producer_label, "Solvay");
        assert_eq!(producers[9].producer_label, "Sumitomo Chemical");
    }

    #[test]
    fn test_get_producers_with_search_filter() {
        let conn = init_test_producer();

        // Test: Call the function with a search filter
        let (producers, count) = get_producers(
            &conn,
            &RequestFilter {
                search: Some("Chemical".to_string()),
                ..Default::default()
            },
        )
        .unwrap();

        // Assert: Verify the results
        assert_eq!(count, 3);
        assert_eq!(producers[0].producer_label, "Dow Chemical Company");
        assert_eq!(producers[1].producer_label, "Mitsubishi Chemical Holdings");
        assert_eq!(producers[2].producer_label, "Sumitomo Chemical");
    }

    #[test]
    fn test_get_producers_with_limit_and_offset() {
        let conn = init_test_producer();

        // Test: Call the function with limit and offset
        let (producers, count) = get_producers(
            &conn,
            &RequestFilter {
                limit: Some(3),
                offset: Some(2),
                ..Default::default()
            },
        )
        .unwrap();

        // Assert: Verify the results
        assert_eq!(count, 10);
        assert_eq!(producers[0].producer_label, "DuPont");
        assert_eq!(producers[1].producer_label, "INEOS");
        assert_eq!(producers[2].producer_label, "Lanxess");
    }

    #[test]
    fn test_get_producers_empty_result() {
        let conn = init_test_producer();

        // Test: Call the function with a search filter that should return no results
        let (producers, count) = get_producers(
            &conn,
            &RequestFilter {
                search: Some("NonExistent".to_string()),
                ..Default::default()
            },
        )
        .unwrap();

        // Assert: Verify the results
        assert!(producers.is_empty());
        assert_eq!(count, 0);
    }
}
