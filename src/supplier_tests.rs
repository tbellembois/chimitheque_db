#[cfg(test)]
mod tests {
    use crate::supplier::*;
    use rusqlite::Connection;

    fn init_test_suppliers() -> Connection {
        let conn = crate::test_utils::init_test();

        // Disable synchronous operations and foreign key constraints for faster test execution
        conn.execute("PRAGMA synchronous = OFF", []).unwrap();
        conn.execute("PRAGMA foreign_keys = OFF", []).unwrap();

        // Clear existing data
        conn.execute("DELETE FROM supplier", []).unwrap();

        // Insert sample suppliers with different patterns
        for i in 1..=20 {
            let label: String;

            if i % 3 == 0 {
                label = format!("Supplier with company {i}");
            } else if i % 5 == 0 {
                label = format!("Another company label {i}");
            } else if i == 10 {
                label = "ExactMatch Supplier".to_string();
            } else {
                label = format!("Supplier {i}");
            }

            conn.execute(
                "INSERT INTO supplier (supplier_id, supplier_label) VALUES (?, ?)",
                [i.to_string(), label],
            )
            .unwrap();
        }

        let label = "Supplier with special ßcharâctérs".to_string();
        conn.execute("INSERT INTO supplier (supplier_label) VALUES (?)", [label])
            .unwrap();

        // Enable foreign key constraints back
        conn.execute("PRAGMA foreign_keys = ON", []).unwrap();

        conn
    }

    #[test]
    fn test_get_all_suppliers() {
        let conn = init_test_suppliers();

        let filter = RequestFilter {
            ..Default::default()
        };
        let (suppliers, count) = get_suppliers(&conn, &filter).unwrap();

        // Test total count and result length
        assert_eq!(count, 21);
        assert_eq!(suppliers.len(), 21);

        // Test sorting by label
        for i in 1..21 {
            if suppliers[i].supplier_label < suppliers[i - 1].supplier_label {
                panic!("Suppliers are not in alphabetical order");
            }
        }

        // Test that no supplier has exact match set
        assert!(suppliers.iter().all(|s| !s.match_exact_search));
    }

    #[test]
    fn test_get_suppliers_search() {
        let conn = init_test_suppliers();

        let search_term = "company";
        let exact_search_term = "ExactMatch Supplier";

        // Test partial match search
        let filter = RequestFilter {
            search: Some(search_term.to_string()),
            ..Default::default()
        };
        let (suppliers, count) = get_suppliers(&conn, &filter).unwrap();
        assert_eq!(count, 7);
        assert_eq!(suppliers.len(), 7);
        assert!(suppliers
            .iter()
            .all(|s| s.supplier_label.contains(search_term)));
        assert!(suppliers.iter().all(|s| !s.match_exact_search));

        // Test exact match search
        let filter = RequestFilter {
            search: Some(exact_search_term.to_string()),
            ..Default::default()
        };
        let (suppliers, count) = get_suppliers(&conn, &filter).unwrap();
        assert_eq!(count, 1);
        assert_eq!(suppliers.len(), 1);
        assert!(suppliers[0].match_exact_search);
        assert_eq!(suppliers[0].supplier_label, exact_search_term);

        // Test case sensitivity
        let case_search_filter = RequestFilter {
            search: Some("ExactMatch SUPPLIER".to_string()),
            ..Default::default()
        };
        let (case_suppliers, case_count) = get_suppliers(&conn, &case_search_filter).unwrap();
        assert_eq!(case_count, 1);
        assert_eq!(case_suppliers.len(), 1);
        assert_eq!(case_suppliers[0].supplier_label, exact_search_term);
        assert!(case_suppliers[0].match_exact_search);

        // Test search term not found
        let filter = RequestFilter {
            search: Some(String::from("Non-existent search term")),
            ..Default::default()
        };
        let (suppliers, count) = get_suppliers(&conn, &filter).unwrap();
        assert_eq!(count, 0);
        assert_eq!(suppliers.len(), 0);
    }

    #[test]
    fn test_get_suppliers_pagination() {
        let conn = init_test_suppliers();

        // Test limit functionality
        let filter = RequestFilter {
            limit: Some(10),
            ..Default::default()
        };
        let (suppliers, count) = get_suppliers(&conn, &filter).unwrap();
        assert_eq!(count, 21);
        assert_eq!(suppliers.len(), 10);
        for i in 0..10 {
            debug!("{}: {:?}", i, suppliers[i].supplier_id);
            assert_eq!(suppliers[i].supplier_id, Some(i as u64 + 1));
        }

        // Test offset functionality
        let filter = RequestFilter {
            offset: Some(5),
            limit: Some(5),
            ..Default::default()
        };
        let (suppliers, count) = get_suppliers(&conn, &filter).unwrap();
        assert_eq!(count, 21);
        assert_eq!(suppliers.len(), 5);
        assert_eq!(suppliers[0].supplier_id, Some(6));

        // Test combining search and pagination
        let search_term = "company";
        let search_filter = RequestFilter {
            search: Some(search_term.to_string()),
            offset: Some(2),
            limit: Some(2),
            ..Default::default()
        };
        let (suppliers, count) = get_suppliers(&conn, &search_filter).unwrap();
        assert_eq!(count, 7);
        assert_eq!(suppliers.len(), 2);
        assert!(suppliers
            .iter()
            .all(|s| s.supplier_label.contains(search_term)));
    }

    #[test]
    fn test_get_suppliers_with_unicode_characters() {
        let conn = init_test_suppliers();

        // Test edge case - Unicode characters
        let filter = RequestFilter {
            search: Some("ß".to_string()),
            ..Default::default()
        };
        let (suppliers, count) = get_suppliers(&conn, &filter).unwrap();
        assert_eq!(count, 1);
        assert_eq!(suppliers.len(), 1);
        assert!(suppliers[0].supplier_label.contains("ß"));
        assert!(suppliers.iter().all(|s| !s.match_exact_search));
    }

    #[test]
    fn test_get_suppliers_with_special_characters() {
        let conn = init_test_suppliers();

        // Test special character search
        let search_term = "ßcharâctérs";
        let filter = RequestFilter {
            search: Some(search_term.to_string()),
            ..Default::default()
        };
        let (suppliers, count) = get_suppliers(&conn, &filter).unwrap();
        assert_eq!(count, 1);
        assert_eq!(suppliers.len(), 1);
        assert!(suppliers[0].match_exact_search);
        assert!(suppliers[0].supplier_label.contains("ßcharâctérs"));

        // Test partial special character search
        let partial_search_term = "charâctérs";
        let filter = RequestFilter {
            search: Some(partial_search_term.to_string()),
            ..Default::default()
        };
        let (suppliers, count) = get_suppliers(&conn, &filter).unwrap();
        assert_eq!(count, 1);
        assert_eq!(suppliers.len(), 1);
        assert!(suppliers[0].supplier_label.contains(partial_search_term));
        assert!(suppliers.iter().all(|s| !s.match_exact_search));
    }

    // Test function to verify data sorting
    #[test]
    fn test_get_suppliers_sorting() {
        let conn = init_test_suppliers();

        // Across different test functions, this ensures that data is returned in alphabetical order
        let filter = RequestFilter {
            ..Default::default()
        };
        let (suppliers, _) = get_suppliers(&conn, &filter).unwrap();

        // Verify alphabetical sorting
        for i in 1..suppliers.len() {
            assert!(suppliers[i].supplier_label >= suppliers[i - 1].supplier_label);
        }
    }
}
