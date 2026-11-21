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
}
