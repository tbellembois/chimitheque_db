#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use crate::stock::compute_stock;
    use rusqlite::{Batch, Connection, fallible_iterator::FallibleIterator};

    // Product 656 is stored in store location 200, of entity 659.
    const PRODUCT_ID: u64 = 656;

    // The test data with only the given storages of product 656, as
    // (quantity, unit id, archived) tuples, in store location 200.
    fn init_test_stock(storages: &[(f64, Option<u64>, bool)]) -> Connection {
        let db = crate::test_utils::init_test();
        let mut batch = Batch::new(&db, include_str!("resources/test.sql"));
        while let Ok(Some(mut stmt)) = batch.next() {
            stmt.execute([]).unwrap();
        }

        db.execute("DELETE FROM storage WHERE product = ?1", [PRODUCT_ID])
            .unwrap();
        for (quantity, unit_id, archive) in storages {
            db.execute(
                "INSERT INTO storage (storage_quantity, unit_quantity, storage_archive, person, product, store_location)
                 VALUES (?1, ?2, ?3, 1, ?4, 200)",
                (quantity, unit_id, archive, PRODUCT_ID),
            )
            .unwrap();
        }

        db
    }

    // Sets the permissions of person 2, as (name, item, entity) tuples.
    fn set_permissions(db: &Connection, permissions: &[(&str, &str, Option<u64>)]) {
        db.execute("DELETE FROM permission WHERE person = 2", [])
            .unwrap();
        for (name, item, entity) in permissions {
            db.execute(
                "INSERT INTO permission (person, permission_name, permission_item, permission_entity)
                 VALUES (2, ?1, ?2, ?3)",
                (name, item, entity),
            )
            .unwrap();
        }
    }

    // The stock lines as (unit label, quantity) tuples, sorted by unit label.
    fn stock_lines(db: &Connection, person_id: u64) -> Vec<(Option<String>, f64)> {
        let mut lines: Vec<(Option<String>, f64)> = compute_stock(db, PRODUCT_ID, person_id)
            .unwrap()
            .into_iter()
            .map(|stock| {
                assert_eq!(stock.store_location.store_location_id, Some(200));
                (stock.unit.map(|unit| unit.unit_label), stock.quantity)
            })
            .collect();
        lines.sort_by(|a, b| a.0.cmp(&b.0));
        lines
    }

    fn assert_stock(lines: &[(Option<String>, f64)], expected: &[(Option<&str>, f64)]) {
        assert_eq!(lines.len(), expected.len(), "{lines:?}");
        for ((unit, quantity), (expected_unit, expected_quantity)) in lines.iter().zip(expected) {
            assert_eq!(unit.as_deref(), *expected_unit, "{lines:?}");
            assert!(
                (quantity - expected_quantity).abs() < 1e-9,
                "{lines:?} expected {expected:?}"
            );
        }
    }

    #[test]
    fn test_compute_stock_sums_by_base_unit() {
        // 1 L + 500 mL, and 200 g + 3 kg.
        let db = init_test_stock(&[
            (1.0, Some(1), false),
            (500.0, Some(2), false),
            (200.0, Some(4), false),
            (3.0, Some(5), false),
        ]);

        assert_stock(
            &stock_lines(&db, 1),
            &[(Some("L"), 1.5), (Some("g"), 3200.0)],
        );
    }

    #[test]
    fn test_compute_stock_without_unit_and_archived() {
        let db = init_test_stock(&[
            (7.0, None, false),
            (2.0, Some(1), false),
            // Archived: not counted.
            (100.0, Some(1), true),
            (100.0, None, true),
        ]);

        assert_stock(&stock_lines(&db, 1), &[(None, 7.0), (Some("L"), 2.0)]);
    }

    #[test]
    fn test_compute_stock_without_history() {
        let mut db = init_test_stock(&[]);
        let mut storage = chimitheque_types::storage::Storage::default();
        storage.product.product_id = Some(PRODUCT_ID);
        storage.person.person_id = Some(1);
        storage.store_location.store_location_id = Some(200);
        storage.storage_quantity = Some(1.0);
        storage.unit_quantity = Some(chimitheque_types::unit::Unit {
            unit_id: Some(1),
            ..Default::default()
        });

        // Created, then updated twice: two history rows.
        let storage_ids =
            crate::storage::create_update_storage(&mut db, storage.clone(), 1, false).unwrap();
        storage.storage_id = Some(storage_ids[0]);
        for quantity in [2.0, 3.0] {
            storage.storage_quantity = Some(quantity);
            crate::storage::create_update_storage(&mut db, storage.clone(), 1, false).unwrap();
        }

        assert_stock(&stock_lines(&db, 1), &[(Some("L"), 3.0)]);
    }

    #[test]
    fn test_compute_stock_with_several_matching_permissions() {
        let db = init_test_stock(&[(1.0, Some(1), false), (500.0, Some(2), false)]);
        set_permissions(&db, &[("w", "storages", Some(659)), ("r", "all", None)]);

        // Counted once.
        assert_stock(&stock_lines(&db, 2), &[(Some("L"), 1.5)]);
    }

    #[test]
    fn test_compute_stock_permissions() {
        let db = init_test_stock(&[(1.0, Some(1), false)]);

        // A permission on another entity, or on another item.
        set_permissions(
            &db,
            &[("all", "storages", Some(1)), ("all", "products", None)],
        );
        assert_stock(&stock_lines(&db, 2), &[]);

        // No read permission.
        set_permissions(&db, &[("n", "storages", None)]);
        assert_stock(&stock_lines(&db, 2), &[]);

        // A permission on the entity.
        set_permissions(&db, &[("r", "storages", Some(659))]);
        assert_stock(&stock_lines(&db, 2), &[(Some("L"), 1.0)]);
    }
}
