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

/// Returns `init_test` with the test data of `resources/test.sql`.
///
/// `test.sql` turns the foreign keys off to insert its tables in any order:
/// they are turned on again after, and the data checked.
///
/// # Panics
///
/// Panics if a statement of the test data fails, or if the data breaks a
/// foreign key.
#[cfg(test)]
#[must_use]
pub fn init_test_with_data() -> Connection {
    let db_connection = init_test();
    db_connection
        .execute_batch(include_str!("resources/test.sql"))
        .unwrap();
    db_connection
        .execute("PRAGMA foreign_keys = ON", [])
        .unwrap();
    let violations: i64 = db_connection
        .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(violations, 0, "foreign key violations in test.sql");
    db_connection
}
