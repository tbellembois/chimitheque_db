#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use crate::producerref::*;
    use chimitheque_types::requestfilter::RequestFilter;
    use rusqlite::Connection;

    // The fixture producer references, in the order of the list: 1-3 of Acros
    // Organics, 4 of Merck / Sigma.
    const PRODUCER_REFS: [&str; 4] = ["ACS reagent", "ETOH-96", "ETOH-99", "Suprapur® grade"];

    fn producer_id(db: &Connection, label: &str) -> u64 {
        db.query_row(
            "SELECT producer_id FROM producer WHERE producer_label = ?1",
            [label],
            |row| row.get(0),
        )
        .unwrap()
    }

    // The label and producer id of a stored producer reference.
    fn stored(db: &Connection, producer_ref_id: u64) -> (String, u64) {
        db.query_row(
            "SELECT producer_ref_label, producer FROM producer_ref WHERE producer_ref_id = ?1",
            [producer_ref_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap()
    }

    fn labels(producer_refs: &[ProducerRefStruct]) -> Vec<&str> {
        producer_refs
            .iter()
            .map(|producer_ref| producer_ref.producer_ref_label.as_str())
            .collect()
    }

    fn new_producer_ref(label: &str, producer_id: u64) -> ProducerRefStruct {
        ProducerRefStruct {
            producer_ref_label: label.to_string(),
            producer: ProducerStruct {
                producer_id: Some(producer_id),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn test_get_producer_refs_with_no_filter() {
        let db = crate::test_utils::init_test_with_fixture();

        let (producer_refs, count) = get_producer_refs(&db, &RequestFilter::default()).unwrap();

        assert_eq!(count, PRODUCER_REFS.len());
        assert_eq!(labels(&producer_refs), PRODUCER_REFS);
        assert_eq!(producer_refs[0].producer.producer_label, "Acros Organics");
    }

    #[test]
    fn test_get_producer_refs_with_search_filter() {
        let db = crate::test_utils::init_test_with_fixture();

        let filter = RequestFilter {
            search: Some(String::from("etoh")),
            ..Default::default()
        };
        let (producer_refs, count) = get_producer_refs(&db, &filter).unwrap();

        assert_eq!(count, 2);
        assert_eq!(labels(&producer_refs), ["ETOH-96", "ETOH-99"]);
        assert!(!producer_refs[0].match_exact_search);
    }

    #[test]
    fn test_get_producer_refs_with_producer_filter() {
        let db = crate::test_utils::init_test_with_fixture();
        let acros = producer_id(&db, "Acros Organics");

        let filter = RequestFilter {
            producer: Some(acros),
            ..Default::default()
        };
        let (producer_refs, count) = get_producer_refs(&db, &filter).unwrap();

        assert_eq!(count, 3);
        assert_eq!(
            labels(&producer_refs),
            ["ACS reagent", "ETOH-96", "ETOH-99"]
        );
        assert!(
            producer_refs
                .iter()
                .all(|producer_ref| producer_ref.producer.producer_id == Some(acros))
        );
    }

    #[test]
    fn test_get_producer_refs_with_limit_and_offset() {
        let db = crate::test_utils::init_test_with_fixture();

        let filter = RequestFilter {
            limit: Some(2),
            ..Default::default()
        };
        let (producer_refs, count) = get_producer_refs(&db, &filter).unwrap();
        // The count ignores the limit and offset.
        assert_eq!(count, PRODUCER_REFS.len());
        assert_eq!(labels(&producer_refs), PRODUCER_REFS[..2]);

        let filter = RequestFilter {
            limit: Some(2),
            offset: Some(2),
            ..Default::default()
        };
        let (producer_refs, count) = get_producer_refs(&db, &filter).unwrap();
        assert_eq!(count, PRODUCER_REFS.len());
        assert_eq!(labels(&producer_refs), PRODUCER_REFS[2..]);
    }

    #[test]
    fn test_get_producer_refs_with_exact_search_match() {
        let db = crate::test_utils::init_test_with_fixture();

        let filter = RequestFilter {
            search: Some(String::from("Suprapur® grade")),
            ..Default::default()
        };
        let (producer_refs, count) = get_producer_refs(&db, &filter).unwrap();

        assert_eq!(count, 1);
        assert!(producer_refs[0].match_exact_search);
        assert_eq!(producer_refs[0].producer_ref_label, "Suprapur® grade");
        assert_eq!(producer_refs[0].producer.producer_label, "Merck / Sigma");
    }

    #[test]
    fn test_create_update_producer_ref_create() {
        let db = crate::test_utils::init_test_with_fixture();
        let acros = producer_id(&db, "Acros Organics");

        let producer_ref_id =
            create_update_producer_ref(&db, &new_producer_ref("ETOH-70", acros)).unwrap();

        assert!(producer_ref_id > 4);
        assert_eq!(stored(&db, producer_ref_id), ("ETOH-70".to_string(), acros));
    }

    #[test]
    fn test_create_update_producer_ref_update() {
        let db = crate::test_utils::init_test_with_fixture();
        let merck = producer_id(&db, "Merck / Sigma");

        // Reference 1 (ETOH-96 of Acros Organics) moved to Merck / Sigma.
        let mut producer_ref = new_producer_ref("ETOH-96", merck);
        producer_ref.producer_ref_id = Some(1);
        let producer_ref_id = create_update_producer_ref(&db, &producer_ref).unwrap();

        assert_eq!(producer_ref_id, 1);
        assert_eq!(stored(&db, 1), ("ETOH-96".to_string(), merck));
    }

    #[test]
    fn test_create_update_producer_ref_unicode_chars() {
        let db = crate::test_utils::init_test_with_fixture();
        let merck = producer_id(&db, "Merck / Sigma");

        let producer_ref_id =
            create_update_producer_ref(&db, &new_producer_ref("テストProducer®", merck)).unwrap();

        assert_eq!(
            stored(&db, producer_ref_id),
            ("テストProducer®".to_string(), merck)
        );
    }

    #[test]
    fn test_clean_producer_ref_label() {
        let db = crate::test_utils::init_test_with_fixture();
        let acros = producer_id(&db, "Acros Organics");

        // Surrounding spaces are trimmed and inner spaces collapsed.
        let producer_ref_id =
            create_update_producer_ref(&db, &new_producer_ref("  ^Once$Upon^   aTime@  ", acros))
                .unwrap();

        assert_eq!(
            stored(&db, producer_ref_id),
            ("^Once$Upon^ aTime@".to_string(), acros)
        );
    }

    #[test]
    fn test_create_update_producer_ref_reuses_existing() {
        let db = crate::test_utils::init_test_with_fixture();
        let acros = producer_id(&db, "Acros Organics");
        let merck = producer_id(&db, "Merck / Sigma");
        let count = |db: &Connection| -> i64 {
            db.query_row("SELECT COUNT(*) FROM producer_ref", [], |row| row.get(0))
                .unwrap()
        };

        // An existing label of the same producer, sent without an id, is reused.
        let reused_id =
            create_update_producer_ref(&db, &new_producer_ref("ETOH-96", acros)).unwrap();
        assert_eq!(reused_id, 1);
        assert_eq!(count(&db), 4);

        // The same label for another producer is a different reference.
        let other_id =
            create_update_producer_ref(&db, &new_producer_ref("ETOH-96", merck)).unwrap();
        assert_ne!(other_id, 1);
        assert_eq!(count(&db), 5);
    }
}
