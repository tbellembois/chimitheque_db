#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use crate::bookmark::*;
    use rusqlite::Connection;

    // In the fixture, chemistry.user (3) bookmarked Ethanol (2) and biology.user
    // (5) Bovine serum albumin (5).
    fn is_bookmarked(db: &Connection, person_id: u64, product_id: u64) -> bool {
        db.query_row(
            "SELECT EXISTS (SELECT 1 FROM bookmark WHERE person = ?1 AND product = ?2)",
            [person_id, product_id],
            |row| row.get(0),
        )
        .unwrap()
    }

    fn bookmark_count(db: &Connection) -> i64 {
        db.query_row("SELECT COUNT(*) FROM bookmark", [], |row| row.get(0))
            .unwrap()
    }

    #[test]
    fn test_toggle_product_bookmark_add_bookmark() {
        let mut db = crate::test_utils::init_test_with_fixture();

        // chemistry.user bookmarks Water (1).
        toggle_product_bookmark(&mut db, 3, 1).unwrap();

        assert!(is_bookmarked(&db, 3, 1));
        assert!(is_bookmarked(&db, 3, 2));
        assert_eq!(bookmark_count(&db), 3);
    }

    #[test]
    fn test_toggle_product_bookmark_remove_bookmark() {
        let mut db = crate::test_utils::init_test_with_fixture();

        // chemistry.user removes their Ethanol (2) bookmark.
        toggle_product_bookmark(&mut db, 3, 2).unwrap();

        assert!(!is_bookmarked(&db, 3, 2));
        // The bookmarks of the others are kept.
        assert!(is_bookmarked(&db, 5, 5));
        assert_eq!(bookmark_count(&db), 1);
    }

    #[test]
    fn test_toggle_product_bookmark_twice() {
        let mut db = crate::test_utils::init_test_with_fixture();

        // The admin bookmarks Sodium chloride (3), then removes it.
        toggle_product_bookmark(&mut db, 1, 3).unwrap();
        assert!(is_bookmarked(&db, 1, 3));
        toggle_product_bookmark(&mut db, 1, 3).unwrap();
        assert!(!is_bookmarked(&db, 1, 3));
        assert_eq!(bookmark_count(&db), 2);
    }

    #[test]
    fn test_toggle_with_invalid_person_id() {
        let mut db = crate::test_utils::init_test_with_fixture();

        assert!(toggle_product_bookmark(&mut db, 999, 1).is_err());
    }

    #[test]
    fn test_toggle_with_invalid_product_id() {
        let mut db = crate::test_utils::init_test_with_fixture();

        assert!(toggle_product_bookmark(&mut db, 1, 999).is_err());
    }
}
