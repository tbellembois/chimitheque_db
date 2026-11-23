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

/// The people of the test fixture (`resources/test_fixture.sql`), by profile.
#[cfg(test)]
pub mod profiles {
    /// The admin: all on all the entities.
    pub const ADMIN: u64 = 1;
    /// Manager of Chemistry (1): all on Chemistry.
    pub const CHEMISTRY_MANAGER: u64 = 2;
    /// Member of Chemistry: reads its entity, writes its storages.
    pub const CHEMISTRY_USER: u64 = 3;
    /// Manager of Biology (2): all on Biology.
    pub const BIOLOGY_MANAGER: u64 = 4;
    /// Member of Biology: reads its entity and storages, and the restricted
    /// products.
    pub const BIOLOGY_USER: u64 = 5;
    /// No entity: reads the products only, as a new user.
    pub const NEW_USER: u64 = 6;
    /// No entity: reads the products, and the storages of all the entities.
    pub const ALL_READER: u64 = 7;
    /// No entity: writes the products, and so reads the restricted ones.
    pub const PRODUCT_EDITOR: u64 = 8;

    /// All the profiles.
    pub const ALL: [u64; 8] = [
        ADMIN,
        CHEMISTRY_MANAGER,
        CHEMISTRY_USER,
        BIOLOGY_MANAGER,
        BIOLOGY_USER,
        NEW_USER,
        ALL_READER,
        PRODUCT_EDITOR,
    ];
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
