#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use crate::stock::compute_stock;
    use crate::test_utils::profiles::*;
    use rusqlite::Connection;

    // See the fixture description in resources/test_fixture.sql. Ethanol (2) is
    // stored on Shelf (3, Chemistry): 500 mL, with a history row of 1 L; in
    // Fridge (4, Chemistry): 2 L archived; in Cabinet [CB] (6, Biology): 1 L.
    const ETHANOL: u64 = 2;
    // Sodium chloride (3) has no storage: the tests give it some.
    const SODIUM_CHLORIDE: u64 = 3;

    // Base data unit ids.
    const L: u64 = 1;
    const ML: u64 = 2;
    const G: u64 = 4;
    const KG: u64 = 5;

    // The fixture, with the given storages of sodium chloride on Shelf (3), as
    // (quantity, unit id, archived) tuples.
    fn init_test_stock(storages: &[(f64, Option<u64>, bool)]) -> Connection {
        let db = crate::test_utils::init_test_with_fixture();
        for (quantity, unit_id, archive) in storages {
            db.execute(
                "INSERT INTO storage (storage_quantity, unit_quantity, storage_archive, person, product, store_location)
                 VALUES (?1, ?2, ?3, 3, ?4, 3)",
                (quantity, unit_id, archive, SODIUM_CHLORIDE),
            )
            .unwrap();
        }
        db
    }

    // Gives new.user (6) only the given permissions, as (name, item, entity)
    // tuples.
    fn set_permissions(db: &Connection, permissions: &[(&str, &str, Option<u64>)]) {
        db.execute("DELETE FROM permission WHERE person = 6", [])
            .unwrap();
        for (name, item, entity) in permissions {
            db.execute(
                "INSERT INTO permission (person, permission_name, permission_item, permission_entity)
                 VALUES (6, ?1, ?2, ?3)",
                (name, item, entity),
            )
            .unwrap();
        }
    }

    // The stock lines as (store location, unit label, quantity) tuples, sorted.
    fn stock_lines(
        db: &Connection,
        product_id: u64,
        person_id: u64,
    ) -> Vec<(String, Option<String>, f64)> {
        let mut lines: Vec<(String, Option<String>, f64)> =
            compute_stock(db, product_id, person_id)
                .unwrap()
                .into_iter()
                .map(|stock| {
                    (
                        stock.store_location.store_location_name,
                        stock.unit.map(|unit| unit.unit_label),
                        stock.quantity,
                    )
                })
                .collect();
        lines.sort_by(|a, b| (&a.0, &a.1).cmp(&(&b.0, &b.1)));
        lines
    }

    fn assert_stock(
        lines: &[(String, Option<String>, f64)],
        expected: &[(&str, Option<&str>, f64)],
    ) {
        assert_eq!(lines.len(), expected.len(), "{lines:?}");
        for ((location, unit, quantity), (expected_location, expected_unit, expected_quantity)) in
            lines.iter().zip(expected)
        {
            assert_eq!(location, expected_location, "{lines:?}");
            assert_eq!(unit.as_deref(), *expected_unit, "{lines:?}");
            assert!(
                (quantity - expected_quantity).abs() < 1e-9,
                "{lines:?} expected {expected:?}"
            );
        }
    }

    #[test]
    fn test_compute_stock_of_the_fixture() {
        let db = crate::test_utils::init_test_with_fixture();

        // The archived storage and the history row are not counted.
        assert_stock(
            &stock_lines(&db, ETHANOL, 1),
            &[("Cabinet [CB]", Some("L"), 1.0), ("Shelf", Some("L"), 0.5)],
        );
    }

    #[test]
    fn test_compute_stock_sums_by_base_unit() {
        // 1 L + 500 mL, and 200 g + 3 kg.
        let db = init_test_stock(&[
            (1.0, Some(L), false),
            (500.0, Some(ML), false),
            (200.0, Some(G), false),
            (3.0, Some(KG), false),
        ]);

        assert_stock(
            &stock_lines(&db, SODIUM_CHLORIDE, 1),
            &[("Shelf", Some("L"), 1.5), ("Shelf", Some("g"), 3200.0)],
        );
    }

    #[test]
    fn test_compute_stock_without_unit_and_archived() {
        let db = init_test_stock(&[
            (7.0, None, false),
            (2.0, Some(L), false),
            // Archived: not counted.
            (100.0, Some(L), true),
            (100.0, None, true),
        ]);

        assert_stock(
            &stock_lines(&db, SODIUM_CHLORIDE, 1),
            &[("Shelf", None, 7.0), ("Shelf", Some("L"), 2.0)],
        );
    }

    #[test]
    fn test_compute_stock_without_history() {
        let mut db = crate::test_utils::init_test_with_fixture();
        let mut storage = chimitheque_types::storage::Storage::default();
        storage.product.product_id = Some(SODIUM_CHLORIDE);
        storage.person.person_id = Some(3);
        storage.store_location.store_location_id = Some(3);
        storage.storage_quantity = Some(1.0);
        storage.unit_quantity = Some(chimitheque_types::unit::Unit {
            unit_id: Some(L),
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

        assert_stock(
            &stock_lines(&db, SODIUM_CHLORIDE, 1),
            &[("Shelf", Some("L"), 3.0)],
        );
    }

    #[test]
    fn test_compute_stock_permissions() {
        let db = crate::test_utils::init_test_with_fixture();
        let chemistry = [("Shelf", Some("L"), 0.5)];
        let biology = [("Cabinet [CB]", Some("L"), 1.0)];
        let all = [("Cabinet [CB]", Some("L"), 1.0), ("Shelf", Some("L"), 0.5)];

        // The stock of the entities the person reads the storages of.
        for (person_id, expected) in [
            (ADMIN, &all[..]),
            (CHEMISTRY_MANAGER, &chemistry[..]),
            (CHEMISTRY_USER, &chemistry[..]),
            (BIOLOGY_MANAGER, &biology[..]),
            (BIOLOGY_USER, &biology[..]),
            (NEW_USER, &[][..]),
            (ALL_READER, &all[..]),
            (PRODUCT_EDITOR, &[][..]),
        ] {
            assert_stock(&stock_lines(&db, ETHANOL, person_id), expected);
        }

        // A permission on another item, or a n permission, gives nothing.
        set_permissions(&db, &[("all", "products", None), ("n", "storages", None)]);
        assert_stock(&stock_lines(&db, ETHANOL, NEW_USER), &[]);
    }

    #[test]
    fn test_compute_stock_with_several_matching_permissions() {
        let db = crate::test_utils::init_test_with_fixture();

        // Two permissions on Chemistry, one on all the entities: each storage
        // is counted once.
        set_permissions(
            &db,
            &[
                ("w", "storages", Some(1)),
                ("r", "all", Some(1)),
                ("r", "storages", None),
            ],
        );
        assert_stock(
            &stock_lines(&db, ETHANOL, 6),
            &[("Cabinet [CB]", Some("L"), 1.0), ("Shelf", Some("L"), 0.5)],
        );
    }
}
