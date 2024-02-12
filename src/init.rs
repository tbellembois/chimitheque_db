use rusqlite::{Connection, OpenFlags};

pub fn connect(db_path: &str) -> Result<Connection, rusqlite::Error> {
    Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
}

#[cfg(test)]
mod tests {

    use log::info;

    use super::*;

    fn init_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_connect() {
        init_logger();

        let db_path = "/home/thbellem/S3Drive/chimitheque-db/storage.db";
        info!("connecting to {}", db_path);

        assert!(connect(db_path).is_ok());
    }
}
