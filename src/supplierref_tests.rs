#[cfg(test)]
mod tests {
    use crate::supplierref::*;
    use chimitheque_types::requestfilter::RequestFilter;
    use log::info;
    use rusqlite::Connection;

    const TOTAL_SUPPLIERS: usize = 5;
    const TOTAL_SUPPLIER_REFS: usize = 13;

    // Helper function to verify a supplier reference in the database
    fn verify_supplier_ref(
        conn: &Connection,
        ref_id: u64,
        expected_supplier_id: u64,
        expected_label: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut stmt = conn.prepare("SELECT supplier_ref_id, supplier_ref_label, supplier FROM supplier_ref WHERE supplier_ref_id = ?")?;
        let row = stmt.query_row([ref_id], |row| {
            Ok((
                row.get_unwrap::<_, i64>(0),
                row.get_unwrap::<_, String>(1),
                row.get_unwrap::<_, i64>(2),
            ))
        })?;

        let (id, label, supplier) = row;
        assert_eq!(id, i64::try_from(ref_id).unwrap());
        assert_eq!(label, expected_label);
        assert_eq!(supplier, i64::try_from(expected_supplier_id).unwrap());

        Ok(())
    }

    // Helper function to create a supplier reference in the database
    fn insert_supplier_ref(
        conn: &Connection,
        label: &str,
        supplier_id: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        conn.execute(
            "INSERT INTO supplier_ref (supplier_ref_label, supplier) VALUES (?, ?)",
            [label, supplier_id.to_string().as_str()],
        )?;
        Ok(())
    }

    fn init_test_supplierrefs() -> Connection {
        let db = crate::test_utils::init_test();

        // Disable synchronous operations and foreign key constraints for faster test execution
        db.execute("PRAGMA synchronous = OFF", []).unwrap();
        db.execute("PRAGMA foreign_keys = OFF", []).unwrap();

        // Clear existing data
        db.execute("DELETE FROM supplier_ref", []).unwrap();
        db.execute("DELETE FROM supplier", []).unwrap();

        // Insert sample suppliers
        for i in 1..=TOTAL_SUPPLIERS {
            db.execute(
                "INSERT INTO supplier (supplier_id, supplier_label) VALUES (?, ?)",
                [i.to_string(), format!("Supplier {i}")],
            )
            .unwrap();
        }

        // Insert sample supplier references
        // Supplier 1
        insert_supplier_ref(&db, "Supplier 1 Ref 1", 1).unwrap();
        insert_supplier_ref(&db, "Supplier 1 Ref 2", 1).unwrap();
        insert_supplier_ref(&db, "Supplier 1 Ref 3", 1).unwrap();
        insert_supplier_ref(&db, "Supplier 1 Ref 4", 1).unwrap();
        insert_supplier_ref(&db, "Supplier 1 Ref 5", 1).unwrap();

        // Supplier 2
        insert_supplier_ref(&db, "Supplier 2 Ref 1", 2).unwrap();
        insert_supplier_ref(&db, "Supplier 2 Ref 2", 2).unwrap();
        insert_supplier_ref(&db, "Supplier 2 Ref 3", 2).unwrap();

        // Supplier 3
        insert_supplier_ref(&db, "Supplier 3 Ref 1", 3).unwrap();
        insert_supplier_ref(&db, "Supplier 3 Ref 2", 3).unwrap();

        // Supplier 4
        insert_supplier_ref(&db, "Supplier 4 Ref 1", 4).unwrap();

        // Supplier 5
        insert_supplier_ref(&db, "Supplier 5 Ref 1", 5).unwrap();

        // Special ref for exact match testing
        insert_supplier_ref(&db, "Suprapur® grade", 1).unwrap();

        // Enable foreign key constraints back
        db.execute("PRAGMA foreign_keys = ON", []).unwrap();

        db
    }

    #[test]
    fn test_get_supplier_refs_with_not_filter() {
        let db = init_test_supplierrefs();

        info!("testing total result");
        let filter = RequestFilter {
            ..Default::default()
        };
        let (supplier_refs, count) = get_supplier_refs(&db, &filter).unwrap();

        // expected number of results.
        assert_eq!(count, TOTAL_SUPPLIER_REFS);
        assert_eq!(supplier_refs.len(), TOTAL_SUPPLIER_REFS);
        assert_eq!(supplier_refs[0].supplier_ref_label, "Supplier 1 Ref 1"); // Because it's at the beginning
    }

    #[test]
    fn test_get_supplier_refs_with_search_filter() {
        let db = init_test_supplierrefs();

        let filter = RequestFilter {
            search: Some(String::from("Supplier 1 Ref")),
            ..Default::default()
        };
        let (supplier_refs, count) = get_supplier_refs(&db, &filter).unwrap();

        // expected number of results.
        assert_eq!(count, 5);
        // The function should only return 5 records with limit set to 5
        assert_eq!(supplier_refs.len(), 5);
    }

    #[test]
    fn test_get_supplier_refs_with_supplier_filter() {
        let db = init_test_supplierrefs();

        let filter = RequestFilter {
            supplier: Some(1),
            ..Default::default()
        };
        let (supplier_refs, count) = get_supplier_refs(&db, &filter).unwrap();

        // expected number of results.
        assert_eq!(count, 6);
        assert_eq!(supplier_refs.len(), 6);
        assert!(supplier_refs
            .iter()
            .all(|s| s.supplier.supplier_id == Some(1)));
    }

    #[test]
    fn test_get_supplier_refs_with_limit() {
        let db = init_test_supplierrefs();

        let filter = RequestFilter {
            limit: Some(5),
            ..Default::default()
        };
        let (supplier_refs, count) = get_supplier_refs(&db, &filter).unwrap();

        // expected number of results.
        assert_eq!(count, TOTAL_SUPPLIER_REFS);
        assert_eq!(supplier_refs.len(), 5);
    }

    #[test]
    fn test_get_supplier_refs_with_limit_and_offset() {
        let db = init_test_supplierrefs();

        let filter = RequestFilter {
            offset: Some(5),
            limit: Some(5),
            ..Default::default()
        };
        let (supplier_refs, count) = get_supplier_refs(&db, &filter).unwrap();

        // expected number of results.
        assert_eq!(count, TOTAL_SUPPLIER_REFS);
        assert_eq!(supplier_refs.len(), 5);
    }

    #[test]
    fn test_get_supplier_refs_with_exact_search_match() {
        let db = init_test_supplierrefs();

        let filter = RequestFilter {
            search: Some(String::from("Suprapur® grade")),
            ..Default::default()
        };
        let (supplier_refs, count) = get_supplier_refs(&db, &filter).unwrap();

        assert_eq!(count, 1);
        assert_eq!(supplier_refs.len(), 1);
        assert!(supplier_refs[0].match_exact_search);
        assert_eq!(supplier_refs[0].supplier_ref_label, "Suprapur® grade");
    }

    #[test]
    fn test_create_update_supplier_ref_create_success() {
        let conn = init_test_supplierrefs();

        let supplier = SupplierStruct {
            match_exact_search: false,
            supplier_id: Some(1),
            supplier_label: String::default(),
        };

        // Insert a supplier reference
        let ref1 = SupplierRefStruct {
            match_exact_search: false,
            supplier_ref_id: None,
            supplier_ref_label: String::from("My new supplier reference"),
            supplier: supplier.clone(),
        };

        let ref_id1 = create_update_supplier_ref(&conn, &ref1).unwrap();
        assert!(ref_id1 > 0);
        verify_supplier_ref(&conn, ref_id1, 1, "My new supplier reference").unwrap();
    }

    #[test]
    fn test_create_update_supplier_ref_update_success() {
        let conn = init_test_supplierrefs();

        // Test updating an existing supplier reference
        let ref2 = SupplierRefStruct {
            match_exact_search: false,
            supplier_ref_id: Some(1),
            supplier_ref_label: String::from("Supplier 1 Ref 1"),
            supplier: SupplierStruct {
                match_exact_search: false,
                supplier_id: Some(2),
                supplier_label: String::default(),
            },
        };

        let ref_id2 = create_update_supplier_ref(&conn, &ref2).unwrap();
        assert!(ref_id2 == 1);
        verify_supplier_ref(&conn, ref_id2, 2, "Supplier 1 Ref 1").unwrap();
    }

    #[test]
    fn test_create_update_supplier_ref_unicode_chars() {
        let conn = init_test_supplierrefs();

        // Test inserting a reference with Unicode characters
        let unicode_ref = SupplierRefStruct {
            match_exact_search: false,
            supplier_ref_id: None,
            supplier_ref_label: String::from("テストSupplier®"),
            supplier: SupplierStruct {
                match_exact_search: false,
                supplier_id: Some(2),
                supplier_label: String::default(),
            },
        };

        let unicode_ref_id = create_update_supplier_ref(&conn, &unicode_ref).unwrap();
        verify_supplier_ref(&conn, unicode_ref_id, 2, "テストSupplier®").unwrap();
    }

    #[test]
    fn test_clean_supplier_ref_label() {
        let conn = init_test_supplierrefs();

        // Reference with special leading/trailing characters
        let ref1 = SupplierRefStruct {
            match_exact_search: false,
            supplier_ref_id: None,
            supplier_ref_label: "  ^Once$Upon^   aTime@  ".to_string(),
            supplier: SupplierStruct {
                match_exact_search: false,
                supplier_id: Some(1),
                supplier_label: String::from("Supplier 1"),
            },
        };

        // The clean function should remove the leading/trailing special characters
        let expected_label = "^Once$Upon^ aTime@";

        let ref_id1 = create_update_supplier_ref(&conn, &ref1).unwrap();
        verify_supplier_ref(&conn, ref_id1, 1, expected_label).unwrap();
    }
}
