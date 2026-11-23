#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::too_many_lines
)]

use crate::init::{connect_test, create_tables};
use rusqlite::Connection;
use std::sync::Once;

static INIT: Once = Once::new();

/// Returns an in-memory database with the tables, for the tests.
#[must_use]
pub fn init_test() -> Connection {
    INIT.call_once(|| {
        let _ = env_logger::builder().is_test(true).try_init();
    });

    unsafe {
        std::env::set_var("SQLITE_EXTENSION_DIR", "src/extensions");
    };

    let mut db_connection = connect_test();

    create_tables(&mut db_connection).unwrap();

    db_connection
}

/// Returns `init_test` with the base data and the test data of
/// `resources/test_fixture.sql`, described at the top of that file. The
/// foreign keys are on.
///
/// # Panics
///
/// Panics if the base data or a statement of the test data fails.
#[cfg(test)]
#[must_use]
pub fn init_test_with_fixture() -> Connection {
    let mut db_connection = init_test();
    crate::init::populate_db_with_base_data(&mut db_connection).unwrap();
    db_connection
        .execute_batch(include_str!("resources/test_fixture.sql"))
        .unwrap();
    db_connection
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixture_labels_are_found() {
        let db = init_test_with_fixture();
        let count = |sql: &str| -> i64 { db.query_row(sql, [], |row| row.get(0)).unwrap() };

        // The base data rows are referred to by label: none must be missing.
        assert_eq!(
            count(
                "SELECT COUNT(*) FROM product WHERE product_id = 2 AND (cas_number IS NULL OR physical_state IS NULL OR signal_word IS NULL OR producer_ref IS NULL OR unit_molecular_weight IS NULL OR unit_temperature IS NULL OR unit_concentration IS NULL)"
            ),
            0
        );
        assert_eq!(
            count("SELECT COUNT(*) FROM product WHERE product_id = 5 AND category IS NULL"),
            0
        );
        assert_eq!(
            count("SELECT COUNT(*) FROM producer_ref WHERE producer IS NULL"),
            0
        );
        assert_eq!(
            count("SELECT COUNT(*) FROM supplier_ref WHERE supplier IS NULL"),
            0
        );
        assert_eq!(
            count("SELECT COUNT(*) FROM storage WHERE unit_quantity IS NULL"),
            0
        );
        assert_eq!(
            count("SELECT COUNT(*) FROM storage WHERE storage_id = 2 AND supplier IS NULL"),
            0
        );
        for (table, column, expected) in [
            ("productsymbols", "productsymbols_symbol_id", 3),
            (
                "producthazardstatements",
                "producthazardstatements_hazard_statement_id",
                3,
            ),
            (
                "productprecautionarystatements",
                "productprecautionarystatements_precautionary_statement_id",
                1,
            ),
            (
                "productclassesofcompounds",
                "productclassesofcompounds_class_of_compound_id",
                1,
            ),
            ("producttags", "producttags_tag_id", 1),
        ] {
            assert_eq!(
                count(&format!("SELECT COUNT({column}) FROM {table}")),
                expected,
                "{table}"
            );
        }

        // The foreign keys hold.
        assert_eq!(count("SELECT COUNT(*) FROM pragma_foreign_key_check"), 0);
    }
}
