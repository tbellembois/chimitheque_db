#[cfg(test)]
mod tests {

    use crate::supplier::*;

    fn init_test_supplier() -> Connection {
        let db = crate::test_utils::init_test();

        // Disable synchronous operations and foreign key constraints for faster test execution
        db.execute("PRAGMA synchronous = OFF", []).unwrap();
        db.execute("PRAGMA foreign_keys = OFF", []).unwrap();

        // Enable foreign key constraints back
        db.execute("PRAGMA foreign_keys = ON", []).unwrap();

        db.execute(
            "INSERT INTO supplier (supplier_id, supplier_label) VALUES (1, 'BASF')",
            [],
        )
        .unwrap();

        db.execute(
            "INSERT INTO supplier (supplier_id, supplier_label) VALUES (2, 'Dow Chemical Company')",
            [],
        )
        .unwrap();

        db.execute(
            "INSERT INTO supplier (supplier_id, supplier_label) VALUES (3, 'DuPont')",
            [],
        )
        .unwrap();

        db.execute(
            "INSERT INTO supplier (supplier_id, supplier_label) VALUES (4, 'LyondellBasell')",
            [],
        )
        .unwrap();

        db.execute(
            "INSERT INTO supplier (supplier_id, supplier_label) VALUES (5, 'Mitsubishi Chemical Holdings')",
            []
        ).unwrap();

        db.execute(
            "INSERT INTO supplier (supplier_id, supplier_label) VALUES (6, 'SABIC')",
            [],
        )
        .unwrap();

        db.execute(
            "INSERT INTO supplier (supplier_id, supplier_label) VALUES (7, 'INEOS')",
            [],
        )
        .unwrap();

        db.execute(
            "INSERT INTO supplier (supplier_id, supplier_label) VALUES (8, 'Sumitomo Chemical')",
            [],
        )
        .unwrap();

        db.execute(
            "INSERT INTO supplier (supplier_id, supplier_label) VALUES (9, 'Lanxess')",
            [],
        )
        .unwrap();

        db.execute(
            "INSERT INTO supplier (supplier_id, supplier_label) VALUES (10, 'Solvay')",
            [],
        )
        .unwrap();

        // Enable foreign key constraints back
        db.execute("PRAGMA foreign_keys = ON", []).unwrap();

        db
    }

    #[test]
    fn test_get_suppliers_with_no_filter() {
        let conn = init_test_supplier();

        // Test: Call the function with no filter
        let (suppliers, count) = get_suppliers(&conn, &RequestFilter::default()).unwrap();

        // Assert: Verify the results
        assert_eq!(count, 10);
        assert_eq!(suppliers[0].supplier_label, "BASF");
        assert_eq!(suppliers[1].supplier_label, "Dow Chemical Company");
        assert_eq!(suppliers[2].supplier_label, "DuPont");
        assert_eq!(suppliers[3].supplier_label, "INEOS");
        assert_eq!(suppliers[4].supplier_label, "Lanxess");
        assert_eq!(suppliers[5].supplier_label, "LyondellBasell");
        assert_eq!(suppliers[6].supplier_label, "Mitsubishi Chemical Holdings");
        assert_eq!(suppliers[7].supplier_label, "SABIC");
        assert_eq!(suppliers[8].supplier_label, "Solvay");
        assert_eq!(suppliers[9].supplier_label, "Sumitomo Chemical");
    }

    #[test]
    fn test_get_suppliers_with_search_filter() {
        let conn = init_test_supplier();

        // Test: Call the function with a search filter
        let (suppliers, count) = get_suppliers(
            &conn,
            &RequestFilter {
                search: Some("Chemical".to_string()),
                ..Default::default()
            },
        )
        .unwrap();

        // Assert: Verify the results
        assert_eq!(count, 3);
        assert_eq!(suppliers[0].supplier_label, "Dow Chemical Company");
        assert_eq!(suppliers[1].supplier_label, "Mitsubishi Chemical Holdings");
        assert_eq!(suppliers[2].supplier_label, "Sumitomo Chemical");
    }

    #[test]
    fn test_get_suppliers_with_limit_and_offset() {
        let conn = init_test_supplier();

        // Test: Call the function with limit and offset
        let (suppliers, count) = get_suppliers(
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
        assert_eq!(suppliers[0].supplier_label, "DuPont");
        assert_eq!(suppliers[1].supplier_label, "INEOS");
        assert_eq!(suppliers[2].supplier_label, "Lanxess");
    }

    #[test]
    fn test_get_suppliers_empty_result() {
        let conn = init_test_supplier();

        // Test: Call the function with a search filter that should return no results
        let (suppliers, count) = get_suppliers(
            &conn,
            &RequestFilter {
                search: Some("NonExistent".to_string()),
                ..Default::default()
            },
        )
        .unwrap();

        // Assert: Verify the results
        assert!(suppliers.is_empty());
        assert_eq!(count, 0);
    }
}
