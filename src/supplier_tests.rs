#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use crate::supplier::*;
    use chimitheque_defines::SUPPLIERS;

    // The suppliers of the base data (`SUPPLIERS`), in the order of the list:
    // ignoring the ASCII case, as `COLLATE NOCASE`.
    fn sorted_suppliers() -> Vec<&'static str> {
        let mut suppliers = SUPPLIERS.to_vec();
        suppliers.sort_by_key(|supplier| supplier.to_ascii_lowercase());
        suppliers
    }

    fn labels(suppliers: &[SupplierStruct]) -> Vec<&str> {
        suppliers
            .iter()
            .map(|supplier| supplier.supplier_label.as_str())
            .collect()
    }

    #[test]
    fn test_get_suppliers_with_no_filter() {
        let db = crate::test_utils::init_test_with_fixture();

        let (suppliers, count) = get_suppliers(&db, &RequestFilter::default()).unwrap();

        assert_eq!(count, SUPPLIERS.len());
        assert_eq!(labels(&suppliers), sorted_suppliers());
    }

    #[test]
    fn test_get_suppliers_with_search_filter() {
        let db = crate::test_utils::init_test_with_fixture();

        let (suppliers, count) = get_suppliers(
            &db,
            &RequestFilter {
                search: Some("chem".to_string()),
                ..Default::default()
            },
        )
        .unwrap();

        // LIKE ignores the ASCII case.
        let expected: Vec<&str> = sorted_suppliers()
            .into_iter()
            .filter(|supplier| supplier.to_ascii_lowercase().contains("chem"))
            .collect();
        assert_ne!(expected.len(), 0);
        assert_eq!(count, expected.len());
        assert_eq!(labels(&suppliers), expected);
    }

    #[test]
    fn test_get_suppliers_with_limit_and_offset() {
        let db = crate::test_utils::init_test_with_fixture();

        let (suppliers, count) = get_suppliers(
            &db,
            &RequestFilter {
                limit: Some(3),
                offset: Some(2),
                ..Default::default()
            },
        )
        .unwrap();

        // The count ignores the limit and offset.
        assert_eq!(count, SUPPLIERS.len());
        assert_eq!(labels(&suppliers), sorted_suppliers()[2..5]);
    }

    #[test]
    fn test_get_suppliers_empty_result() {
        let db = crate::test_utils::init_test_with_fixture();

        let (suppliers, count) = get_suppliers(
            &db,
            &RequestFilter {
                search: Some("NonExistent".to_string()),
                ..Default::default()
            },
        )
        .unwrap();

        assert_eq!(suppliers.len(), 0);
        assert_eq!(count, 0);
    }
}
