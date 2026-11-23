#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use crate::borrowing::*;
    use rusqlite::Connection;

    // In the fixture, chemistry.manager (2) lent storage 1 to chemistry.user (3).
    fn is_borrowed(db: &Connection, storage_id: u64) -> bool {
        db.query_row(
            "SELECT EXISTS (SELECT 1 FROM borrowing WHERE storage = ?1)",
            [storage_id],
            |row| row.get(0),
        )
        .unwrap()
    }

    // The comment of a borrowed storage.
    fn borrowing_comment(db: &Connection, storage_id: u64) -> Option<String> {
        db.query_row(
            "SELECT borrowing_comment FROM borrowing WHERE storage = ?1",
            [storage_id],
            |row| row.get(0),
        )
        .unwrap()
    }

    #[test]
    fn test_toggle_storage_borrowing_add_borrowing() {
        let mut db = crate::test_utils::init_test_with_fixture();

        // chemistry.manager lends storage 2 to chemistry.user.
        toggle_storage_borrowing(&mut db, 2, 2, 3, Some("foo".to_string())).unwrap();

        assert_eq!(borrowing_comment(&db, 2).as_deref(), Some("foo"));
        // The other borrowing is kept.
        assert!(is_borrowed(&db, 1));
    }

    #[test]
    fn test_toggle_storage_borrowing_remove_borrowing() {
        let mut db = crate::test_utils::init_test_with_fixture();

        // chemistry.manager takes storage 1 back from chemistry.user.
        toggle_storage_borrowing(&mut db, 2, 1, 3, None).unwrap();

        assert!(!is_borrowed(&db, 1));
    }

    #[test]
    fn test_toggle_storage_borrowing_twice() {
        let mut db = crate::test_utils::init_test_with_fixture();

        // The admin lends storage 5 to chemistry.user, then takes it back.
        toggle_storage_borrowing(&mut db, 1, 5, 3, None).unwrap();
        assert!(is_borrowed(&db, 5));
        toggle_storage_borrowing(&mut db, 1, 5, 3, None).unwrap();
        assert!(!is_borrowed(&db, 5));
    }

    #[test]
    fn test_borrow_nonexistent_storage() {
        let mut db = crate::test_utils::init_test_with_fixture();

        assert!(toggle_storage_borrowing(&mut db, 1, 999, 3, None).is_err());
    }

    #[test]
    fn test_borrow_with_invalid_borrower() {
        let mut db = crate::test_utils::init_test_with_fixture();

        assert!(toggle_storage_borrowing(&mut db, 1, 2, 999, None).is_err());
    }

    #[test]
    fn test_toggle_storage_borrowing_without_comment_stores_null() {
        let mut db = crate::test_utils::init_test_with_fixture();

        toggle_storage_borrowing(&mut db, 2, 2, 3, None).unwrap();

        assert!(is_borrowed(&db, 2));
        assert_eq!(borrowing_comment(&db, 2), None);
    }
}
