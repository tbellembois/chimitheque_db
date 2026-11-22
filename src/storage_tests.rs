#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::too_many_lines
    )]

    use rusqlite::{Batch, Connection, fallible_iterator::FallibleIterator};

    use crate::storage::*;
    use chimitheque_types::requestfilter::RequestFilter;

    fn init_test_storages() -> Connection {
        let db = crate::test_utils::init_test();
        let sql = include_str!("resources/test.sql");
        let mut batch = Batch::new(&db, sql);
        while let Ok(Some(mut stmt)) = batch.next() {
            stmt.execute([]).unwrap();
        }
        db
    }

    #[test]
    fn test_get_storages_with_storages_filter() {
        let db = init_test_storages();
        let filter = RequestFilter {
            storages: Some(vec![3, 7, 42]),
            ..Default::default()
        };

        let (storages, count) = get_storages(&db, filter, 1).unwrap();

        assert_eq!(count, 3);
        let mut ids: Vec<u64> = storages
            .iter()
            .map(|storage| storage.storage_id.unwrap())
            .collect();
        ids.sort_unstable();
        assert_eq!(ids, [3, 7, 42]);
    }

    #[test]
    fn test_get_storages_with_product_specificity_filter() {
        let db = init_test_storages();
        let filter = RequestFilter {
            product_specificity: Some("ab".to_string()),
            ..Default::default()
        };

        let (storages, count) = get_storages(&db, filter, 1).unwrap();

        // Current storages of the products whose specificity contains "ab".
        assert_eq!(count, 26);
        assert!(storages.iter().all(|storage| {
            storage
                .product
                .product_specificity
                .as_deref()
                .is_some_and(|specificity| specificity.to_lowercase().contains("ab"))
        }));
        let mut ids: Vec<u64> = storages
            .iter()
            .map(|storage| storage.storage_id.unwrap())
            .collect();
        ids.sort_unstable();
        assert_eq!(
            ids,
            [
                5, 37, 40, 42, 73, 171, 181, 397, 437, 471, 517, 538, 672, 694, 717, 750, 781, 824,
                826, 832, 833, 920, 927, 954, 962, 965
            ]
        );
    }

    #[test]
    fn test_get_storages_loads_product_specificity() {
        let db = init_test_storages();
        // Storage 5 holds product 863.
        let filter = RequestFilter {
            id: Some(5),
            ..Default::default()
        };

        let (storages, _) = get_storages(&db, filter, 1).unwrap();

        let storage = storages
            .iter()
            .find(|storage| storage.storage_id == Some(5))
            .unwrap();
        assert_eq!(
            storage.product.product_specificity.as_deref(),
            Some("FjEZSkHdQJmOG4b97pJl7nA9czz6mHZVabIrCxdr5kzJ1j")
        );
    }

    // The test data where product 656 has only the given barcodes, in store
    // location 200 whose full path is `room/[AB]shelf`.
    fn init_test_barecodes(barecodes: &[&str]) -> Connection {
        let db = init_test_storages();
        db.execute_batch(
            "DELETE FROM storage WHERE product = 656;
             UPDATE store_location SET store_location_full_path = 'room/[AB]shelf' WHERE store_location_id = 200;",
        )
        .unwrap();
        for barecode in barecodes {
            db.execute(
                "INSERT INTO storage (storage_barecode, person, product, store_location) VALUES (?1, 1, 656, 200)",
                [barecode],
            )
            .unwrap();
        }
        db
    }

    fn barecode_parts(db: &mut Connection) -> Result<(String, u64, u64), String> {
        let mut storage = StorageStruct::default();
        storage.store_location.store_location_id = Some(200);
        let db_transaction = db.transaction().unwrap();
        compute_storage_barecode_parts(&db_transaction, &storage, 656, 1)
            .map_err(|error| error.to_string())
    }

    #[test]
    fn test_compute_storage_barecode_parts() {
        // First storage of the product.
        let mut db = init_test_barecodes(&[]);
        assert_eq!(barecode_parts(&mut db), Ok(("AB".to_string(), 656, 1)));

        // Next minor number; a barcode not generated is ignored.
        let mut db = init_test_barecodes(&["AB656.1", "AB656.3", "custom"]);
        assert_eq!(barecode_parts(&mut db), Ok(("AB".to_string(), 656, 4)));
    }

    #[test]
    fn test_compute_storage_barecode_parts_returns_database_errors() {
        let mut db = init_test_barecodes(&["AB656.1"]);
        db.execute(
            "ALTER TABLE storage RENAME COLUMN storage_barecode TO renamed",
            [],
        )
        .unwrap();

        // Not a first barcode as if the product had no storage.
        let error = barecode_parts(&mut db).unwrap_err();
        assert!(error.contains("storage_barecode"), "{error}");
    }

    #[test]
    fn test_export_storages() {
        let db = init_test_storages();
        // A person sees the borrowings they made: make storage 2's one person 1's.
        db.execute("UPDATE borrowing SET person = 1 WHERE storage = 2", [])
            .unwrap();
        let filter = RequestFilter {
            id: Some(2),
            ..Default::default()
        };

        let csv = export_storages(&db, filter, 1).unwrap();
        let mut reader = csv::Reader::from_reader(csv.as_bytes());
        let headers = reader.headers().unwrap().clone();
        let records: Vec<csv::StringRecord> = reader.records().collect::<Result<_, _>>().unwrap();
        assert_eq!(headers.len(), 29);
        assert_eq!(records.len(), 1);
        let value = |column: &str| {
            let index = headers.iter().position(|header| header == column).unwrap();
            records[0][index].to_string()
        };

        assert_eq!(value("ID"), "2");
        assert_eq!(value("PRODUCT_ID"), "215");
        assert_eq!(
            value("BARECODE"),
            "1aniTlk3bWjZdPH8eQ1Sdu1isVEkhXNsLXVqRpvX3hr6c"
        );
        assert_eq!(value("QUANTITY"), "1.5");
        assert_eq!(value("UNIT"), "X");
        assert_eq!(value("CONCENTRATION"), "788162.59");
        assert_eq!(value("CONCENTRATION_UNIT"), "mg");
        assert_eq!(
            value("STORE_LOCATION"),
            "Innovative modular Local Area Network"
        );
        assert_eq!(value("ENTITY"), "ENTITY_N-00685");
        assert_eq!(value("NUMBER_OF_BAG"), "46274");
        assert_eq!(value("ENTRY_DATE"), "2022-09-06T11:43:28+00:00");
        assert_eq!(value("ARCHIVE"), "true");
        assert_eq!(value("TO_DESTROY"), "true");
        assert_eq!(value("BORROWER"), "lblair@example.net");
        assert_eq!(value("PERSON"), "ortizandrew@example.org");
    }

    #[test]
    fn test_create_update_storage_ignores_the_history_link() {
        let mut db = init_test_storages();
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
        storage.product.product_id = Some(656);
        storage.person.person_id = Some(1);
        storage.store_location.store_location_id = Some(200);
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
        let db = init_test_storages();
        db.execute(
            "INSERT INTO storage (storage_id, person, product, store_location, storage) VALUES (5000, 1, 656, 200, 2)",
            [],
        )
        .unwrap();

        assert!(storage_is_history(&db, 5000).unwrap());
        assert!(!storage_is_history(&db, 2).unwrap());
        assert!(!storage_is_history(&db, 999_999).unwrap());
    }
}
