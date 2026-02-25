use rusqlite::Connection;

use crate::init::{connect_test, init_db, insert_fake_values};

pub fn init_test() -> Connection {
    let _ = env_logger::builder().is_test(true).try_init();
    unsafe {
        std::env::set_var(
            "SQLITE_EXTENSION_DIR",
            "/home/thbellem/workspace/workspace_rust/chimitheque_db/src/extensions",
        )
    };
    let mut db_connection = connect_test();
    init_db(&mut db_connection).unwrap();
    insert_fake_values(&mut db_connection).unwrap();
    db_connection
}
