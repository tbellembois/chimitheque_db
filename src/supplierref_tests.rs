#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use crate::supplierref::*;
    use chimitheque_types::requestfilter::RequestFilter;
    use rusqlite::Connection;

    // The fixture supplier references, in the order of the list: 1-3 of Acros
    // Organics, 4 of Merck / Sigma.
    const SUPPLIER_REFS: [&str; 4] = ["A-1234", "A-5678", "A-9999", "Suprapur® grade"];

    fn supplier_id(db: &Connection, label: &str) -> u64 {
        db.query_row(
            "SELECT supplier_id FROM supplier WHERE supplier_label = ?1",
            [label],
            |row| row.get(0),
        )
        .unwrap()
    }

    // The label and supplier id of a stored supplier reference.
    fn stored(db: &Connection, supplier_ref_id: u64) -> (String, u64) {
        db.query_row(
            "SELECT supplier_ref_label, supplier FROM supplier_ref WHERE supplier_ref_id = ?1",
            [supplier_ref_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap()
    }

    fn labels(supplier_refs: &[SupplierRefStruct]) -> Vec<&str> {
        supplier_refs
            .iter()
            .map(|supplier_ref| supplier_ref.supplier_ref_label.as_str())
            .collect()
    }

    fn new_supplier_ref(label: &str, supplier_id: u64) -> SupplierRefStruct {
        SupplierRefStruct {
            supplier_ref_label: label.to_string(),
            supplier: SupplierStruct {
                supplier_id: Some(supplier_id),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn test_get_supplier_refs_with_no_filter() {
        let db = crate::test_utils::init_test_with_fixture();

        let (supplier_refs, count) = get_supplier_refs(&db, &RequestFilter::default()).unwrap();

        assert_eq!(count, SUPPLIER_REFS.len());
        assert_eq!(labels(&supplier_refs), SUPPLIER_REFS);
        assert_eq!(supplier_refs[0].supplier.supplier_label, "Acros Organics");
    }

    #[test]
    fn test_get_supplier_refs_with_search_filter() {
        let db = crate::test_utils::init_test_with_fixture();

        let filter = RequestFilter {
            search: Some(String::from("a-")),
            ..Default::default()
        };
        let (supplier_refs, count) = get_supplier_refs(&db, &filter).unwrap();

        assert_eq!(count, 3);
        assert_eq!(labels(&supplier_refs), ["A-1234", "A-5678", "A-9999"]);
        assert!(!supplier_refs[0].match_exact_search);
    }

    #[test]
    fn test_get_supplier_refs_with_supplier_filter() {
        let db = crate::test_utils::init_test_with_fixture();
        let acros = supplier_id(&db, "Acros Organics");

        let filter = RequestFilter {
            supplier: Some(acros),
            ..Default::default()
        };
        let (supplier_refs, count) = get_supplier_refs(&db, &filter).unwrap();

        assert_eq!(count, 3);
        assert_eq!(labels(&supplier_refs), ["A-1234", "A-5678", "A-9999"]);
        assert!(
            supplier_refs
                .iter()
                .all(|supplier_ref| supplier_ref.supplier.supplier_id == Some(acros))
        );
    }

    #[test]
    fn test_get_supplier_refs_with_limit_and_offset() {
        let db = crate::test_utils::init_test_with_fixture();

        let filter = RequestFilter {
            limit: Some(2),
            ..Default::default()
        };
        let (supplier_refs, count) = get_supplier_refs(&db, &filter).unwrap();
        // The count ignores the limit and offset.
        assert_eq!(count, SUPPLIER_REFS.len());
        assert_eq!(labels(&supplier_refs), SUPPLIER_REFS[..2]);

        let filter = RequestFilter {
            limit: Some(2),
            offset: Some(2),
            ..Default::default()
        };
        let (supplier_refs, count) = get_supplier_refs(&db, &filter).unwrap();
        assert_eq!(count, SUPPLIER_REFS.len());
        assert_eq!(labels(&supplier_refs), SUPPLIER_REFS[2..]);
    }

    #[test]
    fn test_get_supplier_refs_with_exact_search_match() {
        let db = crate::test_utils::init_test_with_fixture();

        let filter = RequestFilter {
            search: Some(String::from("Suprapur® grade")),
            ..Default::default()
        };
        let (supplier_refs, count) = get_supplier_refs(&db, &filter).unwrap();

        assert_eq!(count, 1);
        assert!(supplier_refs[0].match_exact_search);
        assert_eq!(supplier_refs[0].supplier_ref_label, "Suprapur® grade");
        assert_eq!(supplier_refs[0].supplier.supplier_label, "Merck / Sigma");
    }

    #[test]
    fn test_create_update_supplier_ref_create() {
        let db = crate::test_utils::init_test_with_fixture();
        let acros = supplier_id(&db, "Acros Organics");

        let supplier_ref_id =
            create_update_supplier_ref(&db, &new_supplier_ref("A-7070", acros)).unwrap();

        assert!(supplier_ref_id > 4);
        assert_eq!(stored(&db, supplier_ref_id), ("A-7070".to_string(), acros));
    }

    #[test]
    fn test_create_update_supplier_ref_rejects_empty_label() {
        let db = crate::test_utils::init_test_with_fixture();
        let acros = supplier_id(&db, "Acros Organics");
        let count = |db: &Connection| -> i64 {
            db.query_row("SELECT COUNT(*) FROM supplier_ref", [], |row| row.get(0))
                .unwrap()
        };
        let before = count(&db);

        for label in ["", "  \n "] {
            let error =
                create_update_supplier_ref(&db, &new_supplier_ref(label, acros)).unwrap_err();
            assert_eq!(error.to_string(), "supplier_ref_label is empty");
        }
        assert_eq!(count(&db), before);
    }

    #[test]
    fn test_create_update_supplier_ref_update() {
        let db = crate::test_utils::init_test_with_fixture();
        let merck = supplier_id(&db, "Merck / Sigma");

        // Reference 1 (A-1234 of Acros Organics) moved to Merck / Sigma.
        let mut supplier_ref = new_supplier_ref("A-1234", merck);
        supplier_ref.supplier_ref_id = Some(1);
        let supplier_ref_id = create_update_supplier_ref(&db, &supplier_ref).unwrap();

        assert_eq!(supplier_ref_id, 1);
        assert_eq!(stored(&db, 1), ("A-1234".to_string(), merck));
    }

    #[test]
    fn test_create_update_supplier_ref_unicode_chars() {
        let db = crate::test_utils::init_test_with_fixture();
        let merck = supplier_id(&db, "Merck / Sigma");

        let supplier_ref_id =
            create_update_supplier_ref(&db, &new_supplier_ref("テストSupplier®", merck)).unwrap();

        assert_eq!(
            stored(&db, supplier_ref_id),
            ("テストSupplier®".to_string(), merck)
        );
    }

    #[test]
    fn test_clean_supplier_ref_label() {
        let db = crate::test_utils::init_test_with_fixture();
        let acros = supplier_id(&db, "Acros Organics");

        // Surrounding spaces are trimmed and inner spaces collapsed.
        let supplier_ref_id =
            create_update_supplier_ref(&db, &new_supplier_ref("  ^Once$Upon^   aTime@  ", acros))
                .unwrap();

        assert_eq!(
            stored(&db, supplier_ref_id),
            ("^Once$Upon^ aTime@".to_string(), acros)
        );
    }

    #[test]
    fn test_create_update_supplier_ref_reuses_existing() {
        let db = crate::test_utils::init_test_with_fixture();
        let acros = supplier_id(&db, "Acros Organics");
        let merck = supplier_id(&db, "Merck / Sigma");
        let count = |db: &Connection| -> i64 {
            db.query_row("SELECT COUNT(*) FROM supplier_ref", [], |row| row.get(0))
                .unwrap()
        };

        // An existing label of the same supplier, sent without an id, is reused.
        let reused_id =
            create_update_supplier_ref(&db, &new_supplier_ref("A-1234", acros)).unwrap();
        assert_eq!(reused_id, 1);
        assert_eq!(count(&db), 4);

        // The same label for another supplier is a different reference.
        let other_id = create_update_supplier_ref(&db, &new_supplier_ref("A-1234", merck)).unwrap();
        assert_ne!(other_id, 1);
        assert_eq!(count(&db), 5);
    }
}
