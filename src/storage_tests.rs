#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use rusqlite::Connection;

    use crate::storage::*;
    use crate::test_utils::profiles::*;
    use chimitheque_types::requestfilter::RequestFilter;

    // See the fixture description in resources/test_fixture.sql: storages 1-3
    // and 5 in Chemistry, 4 the history row of 2, 6-7 in Biology; storage 5
    // holds the restricted Acetone.

    // The sorted ids of the storages the person gets with the filter.
    fn storage_ids(db: &Connection, filter: RequestFilter, person_id: u64) -> Vec<u64> {
        let (storages, count) = get_storages(db, filter, person_id).unwrap();
        assert_eq!(count, storages.len());
        let mut ids: Vec<u64> = storages
            .iter()
            .filter_map(|storage| storage.storage_id)
            .collect();
        ids.sort_unstable();
        ids
    }

    #[test]
    fn test_get_storages_permissions() {
        let db = crate::test_utils::init_test_with_fixture();

        // The current storages (not the history row 4) of the entities the
        // person reads the storages of; storage 5 holds the restricted Acetone.
        for (person_id, expected) in [
            (ADMIN, vec![1, 2, 3, 5, 6, 7]),
            (CHEMISTRY_MANAGER, vec![1, 2, 3, 5]),
            // Not the restricted product.
            (CHEMISTRY_USER, vec![1, 2, 3]),
            (BIOLOGY_MANAGER, vec![6, 7]),
            (BIOLOGY_USER, vec![6, 7]),
            (NEW_USER, vec![]),
            // All the entities, but not the restricted product.
            (ALL_READER, vec![1, 2, 3, 6, 7]),
            // Reads the restricted products, but no storage.
            (PRODUCT_EDITOR, vec![]),
        ] {
            assert_eq!(
                storage_ids(&db, RequestFilter::default(), person_id),
                expected,
                "{person_id}"
            );
        }

        // A filter does not give more than what the person can read.
        let filter = RequestFilter {
            storages: Some(vec![1, 6]),
            ..Default::default()
        };
        assert_eq!(storage_ids(&db, filter, BIOLOGY_USER), [6]);
    }

    #[test]
    fn test_export_storages_permissions() {
        let db = crate::test_utils::init_test_with_fixture();

        // A header, and a row per storage the person reads.
        for (person_id, rows) in [
            (ADMIN, 6),
            (CHEMISTRY_USER, 3),
            (BIOLOGY_USER, 2),
            (NEW_USER, 0),
        ] {
            let csv = export_storages(&db, RequestFilter::default(), person_id).unwrap();
            assert_eq!(csv.lines().count(), rows + 1, "{person_id}");
        }
    }

    #[test]
    fn test_get_storages_with_storages_filter() {
        let db = crate::test_utils::init_test_with_fixture();

        let filter = RequestFilter {
            storages: Some(vec![1, 5, 6]),
            ..Default::default()
        };
        assert_eq!(storage_ids(&db, filter, 1), [1, 5, 6]);
    }

    #[test]
    fn test_get_storages_with_product_specificity_filter() {
        let db = crate::test_utils::init_test_with_fixture();

        // The storages of the "absolute" Ethanol.
        let filter = RequestFilter {
            product_specificity: Some("abs".to_string()),
            ..Default::default()
        };
        assert_eq!(storage_ids(&db, filter, 1), [2, 3, 7]);
    }

    #[test]
    fn test_get_storages_history() {
        let db = crate::test_utils::init_test_with_fixture();

        // Storage 2 and its history row.
        let filter = RequestFilter {
            id: Some(2),
            history: true,
            ..Default::default()
        };
        assert_eq!(storage_ids(&db, filter, 1), [2, 4]);

        let (storages, _) = get_storages(
            &db,
            RequestFilter {
                id: Some(2),
                ..Default::default()
            },
            1,
        )
        .unwrap();
        assert_eq!(storages[0].storage_hc, 1);
    }

    #[test]
    fn test_get_storages_loads_the_storage() {
        let db = crate::test_utils::init_test_with_fixture();

        let filter = RequestFilter {
            id: Some(2),
            ..Default::default()
        };
        let (storages, _) = get_storages(&db, filter, 1).unwrap();
        let storage = &storages[0];

        assert_eq!(storage.product.name.name_label, "ETHANOL");
        assert_eq!(
            storage.product.product_specificity.as_deref(),
            Some("absolute")
        );
        assert_eq!(storage.storage_quantity, Some(500.0));
        assert_eq!(storage.unit_quantity.as_ref().unwrap().unit_label, "mL");
        assert_eq!(storage.store_location.store_location_name, "Shelf");
        assert_eq!(
            storage.supplier.as_ref().unwrap().supplier_label,
            "Acros Organics"
        );
        assert_eq!(storage.storage_barecode.as_deref(), Some("CA2.1"));
    }

    fn barecode_parts(
        db: &mut Connection,
        product_id: u64,
        store_location_id: u64,
    ) -> Result<(String, u64, u64), String> {
        let mut storage = StorageStruct::default();
        storage.store_location.store_location_id = Some(store_location_id);
        let db_transaction = db.transaction().unwrap();
        compute_storage_barecode_parts(&db_transaction, &storage, product_id, 1)
            .map_err(|error| error.to_string())
    }

    #[test]
    fn test_compute_storage_barecode_parts() {
        let mut db = crate::test_utils::init_test_with_fixture();

        // Ethanol (2) in Chemistry has CA2.1 and CA2.2 (and the history row
        // CA2.1): the next one is CA2.3. The code comes from Cabinet [CA].
        assert_eq!(barecode_parts(&mut db, 2, 3), Ok(("CA".to_string(), 2, 3)));
        // In Biology, it has CB2.1.
        assert_eq!(barecode_parts(&mut db, 2, 6), Ok(("CB".to_string(), 2, 2)));

        // Sodium chloride (3) has no storage: its first one.
        assert_eq!(barecode_parts(&mut db, 3, 2), Ok(("CA".to_string(), 3, 1)));

        // A barcode not generated is ignored.
        db.execute_batch(
            "INSERT INTO storage (storage_barecode, person, product, store_location) VALUES
               ('CA3.1', 3, 3, 3), ('CA3.3', 3, 3, 3), ('custom', 3, 3, 3);",
        )
        .unwrap();
        assert_eq!(barecode_parts(&mut db, 3, 2), Ok(("CA".to_string(), 3, 4)));
    }

    #[test]
    fn test_compute_storage_barecode_parts_returns_database_errors() {
        let mut db = crate::test_utils::init_test_with_fixture();
        db.execute(
            "ALTER TABLE storage RENAME COLUMN storage_barecode TO renamed",
            [],
        )
        .unwrap();

        // Not a first barcode as if the product had no storage.
        let error = barecode_parts(&mut db, 2, 3).unwrap_err();
        assert!(error.contains("storage_barecode"), "{error}");
    }

    #[test]
    fn test_export_storages() {
        let db = crate::test_utils::init_test_with_fixture();
        let filter = RequestFilter {
            id: Some(1),
            ..Default::default()
        };

        // Exported by chemistry.manager, who lent it: the borrowing is theirs.
        let csv = export_storages(&db, filter, 2).unwrap();
        let mut reader = csv::Reader::from_reader(csv.as_bytes());
        let headers = reader.headers().unwrap().clone();
        let records: Vec<csv::StringRecord> = reader.records().collect::<Result<_, _>>().unwrap();
        assert_eq!(headers.len(), 29);
        assert_eq!(records.len(), 1);
        let value = |column: &str| {
            let index = headers.iter().position(|header| header == column).unwrap();
            records[0][index].to_string()
        };

        assert_eq!(value("ID"), "1");
        assert_eq!(value("PRODUCT_NAME"), "WATER");
        assert_eq!(value("PRODUCT_ID"), "1");
        assert_eq!(value("BARECODE"), "CA1.1");
        assert_eq!(value("QUANTITY"), "1");
        assert_eq!(value("UNIT"), "L");
        assert_eq!(value("STORE_LOCATION"), "Shelf");
        assert_eq!(
            value("STORE_LOCATION_FULL_PATH"),
            "Room 101/Cabinet [CA]/Shelf"
        );
        assert_eq!(value("ENTITY"), "Chemistry");
        assert_eq!(value("BATCH_NUMBER"), "B-001");
        assert_eq!(value("ENTRY_DATE"), "2024-01-15T00:00:00+00:00");
        assert_eq!(value("ARCHIVE"), "false");
        assert_eq!(value("BORROWER"), "chemistry.user@example.com");
        assert_eq!(value("BORROWING_COMMENT"), "practical work");
        assert_eq!(value("PERSON"), "chemistry.user@example.com");
    }

    #[test]
    fn test_export_storages_shows_only_own_borrowings() {
        let db = crate::test_utils::init_test_with_fixture();
        let filter = RequestFilter {
            id: Some(1),
            ..Default::default()
        };

        // The admin did not lend storage 1.
        let csv = export_storages(&db, filter, 1).unwrap();
        let mut reader = csv::Reader::from_reader(csv.as_bytes());
        let headers = reader.headers().unwrap().clone();
        let record = reader.records().next().unwrap().unwrap();
        let borrower = headers
            .iter()
            .position(|header| header == "BORROWER")
            .unwrap();
        assert_eq!(&record[borrower], "");
    }

    #[test]
    fn test_create_update_storage_ignores_the_history_link() {
        let mut db = crate::test_utils::init_test_with_fixture();
        let history_link = |db: &Connection, storage_id: u64| -> Option<u64> {
            db.query_row(
                "SELECT storage FROM storage WHERE storage_id = ?1",
                [storage_id],
                |row| row.get(0),
            )
            .unwrap()
        };

        // A storage claiming to be a history row of storage 2.
        let mut storage = StorageStruct::default();
        storage.product.product_id = Some(3);
        storage.person.person_id = Some(3);
        storage.store_location.store_location_id = Some(3);
        storage.storage = Some(Box::new(StorageStruct {
            storage_id: Some(2),
            ..Default::default()
        }));

        let storage_id = create_update_storage(&mut db, storage.clone(), 1, false).unwrap()[0];
        assert_eq!(history_link(&db, storage_id), None);

        // Same on update.
        storage.storage_id = Some(storage_id);
        create_update_storage(&mut db, storage, 1, false).unwrap();
        assert_eq!(history_link(&db, storage_id), None);
    }

    #[test]
    fn test_storage_is_history() {
        let db = crate::test_utils::init_test_with_fixture();

        assert!(storage_is_history(&db, 4).unwrap());
        assert!(!storage_is_history(&db, 2).unwrap());
        assert!(!storage_is_history(&db, 999_999).unwrap());
    }

    // A new liter of sodium chloride (3) on Shelf (3), saved by chemistry.user.
    fn new_storage() -> StorageStruct {
        let mut storage = StorageStruct::default();
        storage.product.product_id = Some(3);
        storage.person.person_id = Some(CHEMISTRY_USER);
        storage.store_location.store_location_id = Some(3);
        storage.storage_quantity = Some(1.0);
        storage.unit_quantity = Some(chimitheque_types::unit::Unit {
            unit_id: Some(1),
            ..Default::default()
        });
        storage
    }

    fn barecode(db: &Connection, storage_id: u64) -> Option<String> {
        db.query_row(
            "SELECT storage_barecode FROM storage WHERE storage_id = ?1",
            [storage_id],
            |row| row.get(0),
        )
        .unwrap()
    }

    #[test]
    fn test_create_storage_generates_the_barecode_and_qrcode() {
        let mut db = crate::test_utils::init_test_with_fixture();

        let storage_ids = create_update_storage(&mut db, new_storage(), 1, false).unwrap();

        assert_eq!(storage_ids, [8]);
        // The code of Cabinet [CA], the product id, the first number.
        assert_eq!(barecode(&db, 8).as_deref(), Some("CA3.1"));
        let qrcode: Option<Vec<u8>> = db
            .query_row(
                "SELECT storage_qrcode FROM storage WHERE storage_id = 8",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(qrcode.is_some_and(|qrcode| qrcode.starts_with(b"\x89PNG")));
    }

    #[test]
    fn test_create_storage_copies() {
        let mut db = crate::test_utils::init_test_with_fixture();

        // Three copies: the next numbers.
        let storage_ids = create_update_storage(&mut db, new_storage(), 3, false).unwrap();
        assert_eq!(storage_ids, [8, 9, 10]);
        let barecodes: Vec<Option<String>> =
            storage_ids.iter().map(|id| barecode(&db, *id)).collect();
        assert_eq!(
            barecodes,
            [
                Some("CA3.1".to_string()),
                Some("CA3.2".to_string()),
                Some("CA3.3".to_string())
            ]
        );

        // Two copies with the same barcode.
        let storage_ids = create_update_storage(&mut db, new_storage(), 2, true).unwrap();
        assert_eq!(barecode(&db, storage_ids[0]).as_deref(), Some("CA3.4"));
        assert_eq!(barecode(&db, storage_ids[1]).as_deref(), Some("CA3.4"));
    }

    #[test]
    fn test_create_storage_keeps_a_given_barecode() {
        let mut db = crate::test_utils::init_test_with_fixture();
        let mut storage = new_storage();
        storage.storage_barecode = Some("MY-CODE".to_string());

        let storage_ids = create_update_storage(&mut db, storage, 1, false).unwrap();

        assert_eq!(barecode(&db, storage_ids[0]).as_deref(), Some("MY-CODE"));
    }

    #[test]
    fn test_update_storage_keeps_the_previous_version() {
        let mut db = crate::test_utils::init_test_with_fixture();
        let filter = RequestFilter {
            id: Some(1),
            ..Default::default()
        };
        let mut storage = get_storages(&db, filter.clone(), ADMIN)
            .unwrap()
            .0
            .remove(0);

        // Water (storage 1): 1 L, then 2 L.
        storage.storage_quantity = Some(2.0);
        assert_eq!(
            create_update_storage(&mut db, storage, 1, false).unwrap(),
            [1]
        );

        let storage = get_storages(&db, filter, ADMIN).unwrap().0.remove(0);
        assert_eq!(storage.storage_quantity, Some(2.0));
        assert_eq!(storage.storage_barecode.as_deref(), Some("CA1.1"));
        assert_eq!(storage.storage_hc, 1);
        let history_quantity: f64 = db
            .query_row(
                "SELECT storage_quantity FROM storage WHERE storage = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!((history_quantity - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_archive_and_unarchive_storage() {
        let mut db = crate::test_utils::init_test_with_fixture();
        let archived = |db: &Connection, storage_id: u64| -> bool {
            db.query_row(
                "SELECT storage_archive FROM storage WHERE storage_id = ?1",
                [storage_id],
                |row| row.get(0),
            )
            .unwrap()
        };

        // Storage 2, with its history row 4.
        archive_storage(&mut db, 2).unwrap();
        assert!(archived(&db, 2));
        assert!(archived(&db, 4));

        unarchive_storage(&mut db, 2).unwrap();
        assert!(!archived(&db, 2));
        assert!(!archived(&db, 4));
    }

    #[test]
    fn test_delete_storage() {
        let mut db = crate::test_utils::init_test_with_fixture();
        let count = |db: &Connection, sql: &str| -> i64 {
            db.query_row(sql, [], |row| row.get(0)).unwrap()
        };

        // Storage 2, with its history row 4; storage 1, borrowed.
        delete_storage(&mut db, 2).unwrap();
        delete_storage(&mut db, 1).unwrap();

        assert_eq!(
            count(
                &db,
                "SELECT COUNT(*) FROM storage WHERE storage_id IN (1, 2, 4)"
            ),
            0
        );
        assert_eq!(count(&db, "SELECT COUNT(*) FROM borrowing"), 0);
    }

    #[test]
    fn test_storage_entity_id() {
        let db = crate::test_utils::init_test_with_fixture();

        assert_eq!(storage_entity_id(&db, 1).unwrap(), Some(1));
        assert_eq!(storage_entity_id(&db, 6).unwrap(), Some(2));
        assert_eq!(storage_entity_id(&db, 999).unwrap(), None);
    }
}
