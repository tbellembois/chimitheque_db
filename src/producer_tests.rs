#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use crate::producer::*;
    use chimitheque_defines::PRODUCERS;

    // The producers of the base data (`PRODUCERS`), in the order of the list:
    // ignoring the ASCII case, as `COLLATE NOCASE`.
    fn sorted_producers() -> Vec<&'static str> {
        let mut producers = PRODUCERS.to_vec();
        producers.sort_by_key(|producer| producer.to_ascii_lowercase());
        producers
    }

    fn labels(producers: &[ProducerStruct]) -> Vec<&str> {
        producers
            .iter()
            .map(|producer| producer.producer_label.as_str())
            .collect()
    }

    #[test]
    fn test_get_producers_with_no_filter() {
        let db = crate::test_utils::init_test_with_fixture();

        let (producers, count) = get_producers(&db, &RequestFilter::default()).unwrap();

        assert_eq!(count, PRODUCERS.len());
        assert_eq!(labels(&producers), sorted_producers());
    }

    #[test]
    fn test_get_producers_with_search_filter() {
        let db = crate::test_utils::init_test_with_fixture();

        let (producers, count) = get_producers(
            &db,
            &RequestFilter {
                search: Some("chem".to_string()),
                ..Default::default()
            },
        )
        .unwrap();

        // LIKE ignores the ASCII case.
        let expected: Vec<&str> = sorted_producers()
            .into_iter()
            .filter(|producer| producer.to_ascii_lowercase().contains("chem"))
            .collect();
        assert_ne!(expected.len(), 0);
        assert_eq!(count, expected.len());
        assert_eq!(labels(&producers), expected);
    }

    #[test]
    fn test_get_producers_with_limit_and_offset() {
        let db = crate::test_utils::init_test_with_fixture();

        let (producers, count) = get_producers(
            &db,
            &RequestFilter {
                limit: Some(3),
                offset: Some(2),
                ..Default::default()
            },
        )
        .unwrap();

        // The count ignores the limit and offset.
        assert_eq!(count, PRODUCERS.len());
        assert_eq!(labels(&producers), sorted_producers()[2..5]);
    }

    #[test]
    fn test_get_producers_empty_result() {
        let db = crate::test_utils::init_test_with_fixture();

        let (producers, count) = get_producers(
            &db,
            &RequestFilter {
                search: Some("NonExistent".to_string()),
                ..Default::default()
            },
        )
        .unwrap();

        assert_eq!(producers.len(), 0);
        assert_eq!(count, 0);
    }
}
