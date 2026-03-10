#[cfg(test)]
mod tests {
    use crate::producerref::*;
    use chimitheque_types::requestfilter::RequestFilter;
    use log::info;
    use rusqlite::Connection;

    const TOTAL_PRODUCERS: usize = 5;
    const TOTAL_SUPPLIER_REFS: usize = 13;

    // Helper function to verify a producer reference in the database
    fn verify_producer_ref(
        conn: &Connection,
        ref_id: u64,
        expected_producer_id: u64,
        expected_label: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut stmt = conn.prepare("SELECT producer_ref_id, producer_ref_label, producer FROM producer_ref WHERE producer_ref_id = ?")?;
        let row = stmt.query_row([ref_id], |row| {
            Ok((
                row.get_unwrap::<_, i64>(0),
                row.get_unwrap::<_, String>(1),
                row.get_unwrap::<_, i64>(2),
            ))
        })?;

        let (id, label, producer) = row;
        assert_eq!(id, i64::try_from(ref_id).unwrap());
        assert_eq!(label, expected_label);
        assert_eq!(producer, i64::try_from(expected_producer_id).unwrap());

        Ok(())
    }

    // Helper function to create a producer reference in the database
    fn insert_producer_ref(
        conn: &Connection,
        label: &str,
        producer_id: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        conn.execute(
            "INSERT INTO producer_ref (producer_ref_label, producer) VALUES (?, ?)",
            [label, producer_id.to_string().as_str()],
        )?;
        Ok(())
    }

    fn init_test_producerrefs() -> Connection {
        let db = crate::test_utils::init_test();

        // Disable synchronous operations and foreign key constraints for faster test execution
        db.execute("PRAGMA synchronous = OFF", []).unwrap();
        db.execute("PRAGMA foreign_keys = OFF", []).unwrap();

        // Clear existing data
        db.execute("DELETE FROM producer_ref", []).unwrap();
        db.execute("DELETE FROM producer", []).unwrap();

        // Insert sample producers
        for i in 1..=TOTAL_PRODUCERS {
            db.execute(
                "INSERT INTO producer (producer_id, producer_label) VALUES (?, ?)",
                [i.to_string(), format!("Producer {i}")],
            )
            .unwrap();
        }

        // Insert sample producer references
        // Producer 1
        insert_producer_ref(&db, "Producer 1 Ref 1", 1).unwrap();
        insert_producer_ref(&db, "Producer 1 Ref 2", 1).unwrap();
        insert_producer_ref(&db, "Producer 1 Ref 3", 1).unwrap();
        insert_producer_ref(&db, "Producer 1 Ref 4", 1).unwrap();
        insert_producer_ref(&db, "Producer 1 Ref 5", 1).unwrap();

        // Producer 2
        insert_producer_ref(&db, "Producer 2 Ref 1", 2).unwrap();
        insert_producer_ref(&db, "Producer 2 Ref 2", 2).unwrap();
        insert_producer_ref(&db, "Producer 2 Ref 3", 2).unwrap();

        // Producer 3
        insert_producer_ref(&db, "Producer 3 Ref 1", 3).unwrap();
        insert_producer_ref(&db, "Producer 3 Ref 2", 3).unwrap();

        // Producer 4
        insert_producer_ref(&db, "Producer 4 Ref 1", 4).unwrap();

        // Producer 5
        insert_producer_ref(&db, "Producer 5 Ref 1", 5).unwrap();

        // Special ref for exact match testing
        insert_producer_ref(&db, "Suprapur® grade", 1).unwrap();

        // Enable foreign key constraints back
        db.execute("PRAGMA foreign_keys = ON", []).unwrap();

        db
    }

    #[test]
    fn test_get_producer_refs_with_not_filter() {
        let db = init_test_producerrefs();

        info!("testing total result");
        let filter = RequestFilter {
            ..Default::default()
        };
        let (producer_refs, count) = get_producer_refs(&db, &filter).unwrap();

        // expected number of results.
        assert_eq!(count, TOTAL_SUPPLIER_REFS);
        assert_eq!(producer_refs.len(), TOTAL_SUPPLIER_REFS);
        assert_eq!(producer_refs[0].producer_ref_label, "Producer 1 Ref 1"); // Because it's at the beginning
    }

    #[test]
    fn test_get_producer_refs_with_search_filter() {
        let db = init_test_producerrefs();

        let filter = RequestFilter {
            search: Some(String::from("Producer 1 Ref")),
            ..Default::default()
        };
        let (producer_refs, count) = get_producer_refs(&db, &filter).unwrap();

        // expected number of results.
        assert_eq!(count, 5);
        // The function should only return 5 records with limit set to 5
        assert_eq!(producer_refs.len(), 5);
    }

    #[test]
    fn test_get_producer_refs_with_producer_filter() {
        let db = init_test_producerrefs();

        let filter = RequestFilter {
            producer: Some(1),
            ..Default::default()
        };
        let (producer_refs, count) = get_producer_refs(&db, &filter).unwrap();

        // expected number of results.
        assert_eq!(count, 6);
        assert_eq!(producer_refs.len(), 6);
        assert!(producer_refs
            .iter()
            .all(|s| s.producer.producer_id == Some(1)));
    }

    #[test]
    fn test_get_producer_refs_with_limit() {
        let db = init_test_producerrefs();

        let filter = RequestFilter {
            limit: Some(5),
            ..Default::default()
        };
        let (producer_refs, count) = get_producer_refs(&db, &filter).unwrap();

        // expected number of results.
        assert_eq!(count, TOTAL_SUPPLIER_REFS);
        assert_eq!(producer_refs.len(), 5);
    }

    #[test]
    fn test_get_producer_refs_with_limit_and_offset() {
        let db = init_test_producerrefs();

        let filter = RequestFilter {
            offset: Some(5),
            limit: Some(5),
            ..Default::default()
        };
        let (producer_refs, count) = get_producer_refs(&db, &filter).unwrap();

        // expected number of results.
        assert_eq!(count, TOTAL_SUPPLIER_REFS);
        assert_eq!(producer_refs.len(), 5);
    }

    #[test]
    fn test_get_producer_refs_with_exact_search_match() {
        let db = init_test_producerrefs();

        let filter = RequestFilter {
            search: Some(String::from("Suprapur® grade")),
            ..Default::default()
        };
        let (producer_refs, count) = get_producer_refs(&db, &filter).unwrap();

        assert_eq!(count, 1);
        assert_eq!(producer_refs.len(), 1);
        assert!(producer_refs[0].match_exact_search);
        assert_eq!(producer_refs[0].producer_ref_label, "Suprapur® grade");
    }

    #[test]
    fn test_create_update_producer_ref_create_success() {
        let conn = init_test_producerrefs();

        let producer = ProducerStruct {
            match_exact_search: false,
            producer_id: Some(1),
            producer_label: String::default(),
        };

        // Insert a producer reference
        let ref1 = ProducerRefStruct {
            match_exact_search: false,
            producer_ref_id: None,
            producer_ref_label: String::from("My new producer reference"),
            producer: producer.clone(),
        };

        let ref_id1 = create_update_producer_ref(&conn, &ref1).unwrap();
        assert!(ref_id1 > 0);
        verify_producer_ref(&conn, ref_id1, 1, "My new producer reference").unwrap();
    }

    #[test]
    fn test_create_update_producer_ref_update_success() {
        let conn = init_test_producerrefs();

        // Test updating an existing producer reference
        let ref2 = ProducerRefStruct {
            match_exact_search: false,
            producer_ref_id: Some(1),
            producer_ref_label: String::from("Producer 1 Ref 1"),
            producer: ProducerStruct {
                match_exact_search: false,
                producer_id: Some(2),
                producer_label: String::default(),
            },
        };

        let ref_id2 = create_update_producer_ref(&conn, &ref2).unwrap();
        assert!(ref_id2 == 1);
        verify_producer_ref(&conn, ref_id2, 2, "Producer 1 Ref 1").unwrap();
    }

    #[test]
    fn test_create_update_producer_ref_unicode_chars() {
        let conn = init_test_producerrefs();

        // Test inserting a reference with Unicode characters
        let unicode_ref = ProducerRefStruct {
            match_exact_search: false,
            producer_ref_id: None,
            producer_ref_label: String::from("テストProducer®"),
            producer: ProducerStruct {
                match_exact_search: false,
                producer_id: Some(2),
                producer_label: String::default(),
            },
        };

        let unicode_ref_id = create_update_producer_ref(&conn, &unicode_ref).unwrap();
        verify_producer_ref(&conn, unicode_ref_id, 2, "テストProducer®").unwrap();
    }

    #[test]
    fn test_clean_producer_ref_label() {
        let conn = init_test_producerrefs();

        // Reference with special leading/trailing characters
        let ref1 = ProducerRefStruct {
            match_exact_search: false,
            producer_ref_id: None,
            producer_ref_label: "  ^Once$Upon^   aTime@  ".to_string(),
            producer: ProducerStruct {
                match_exact_search: false,
                producer_id: Some(1),
                producer_label: String::from("Producer 1"),
            },
        };

        // The clean function should remove the leading/trailing special characters
        let expected_label = "^Once$Upon^ aTime@";

        let ref_id1 = create_update_producer_ref(&conn, &ref1).unwrap();
        verify_producer_ref(&conn, ref_id1, 1, expected_label).unwrap();
    }
}
