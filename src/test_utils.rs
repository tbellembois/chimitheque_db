use crate::init::{connect_test, create_tables};
use rusqlite::Connection;
use std::sync::Once;

static INIT: Once = Once::new();

#[must_use]
pub fn init_test() -> Connection {
    INIT.call_once(|| {
        let _ = env_logger::builder().is_test(true).try_init();
    });

    unsafe {
        std::env::set_var(
            "SQLITE_EXTENSION_DIR",
            "/home/thbellem/workspace/workspace_rust/chimitheque_db/src/extensions",
        );
    };

    let mut db_connection = connect_test();

    create_tables(&mut db_connection).unwrap();

    db_connection
}
