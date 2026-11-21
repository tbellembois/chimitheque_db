#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::too_many_lines
    )]

    use rusqlite::Connection;

    use crate::init::{connect_test, create_tables};
    use crate::migration::{MigrationError, SCHEMA_VERSION};

    const CHECK: &str = r#" CHECK ("product_type" IN ('chem', 'bio', 'cons'))"#;

    /// An empty in-memory database, as `test_utils::init_test` but without tables.
    fn connect() -> Connection {
        unsafe {
            std::env::set_var("SQLITE_EXTENSION_DIR", "src/extensions");
        };
        connect_test()
    }

    fn user_version(db: &Connection) -> i64 {
        db.pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap()
    }

    fn count(db: &Connection, sql: &str) -> i64 {
        db.query_row(sql, [], |row| row.get(0)).unwrap()
    }

    fn insert_product(
        db: &Connection,
        product_id: i64,
        product_type: &str,
    ) -> rusqlite::Result<usize> {
        db.execute(
            "INSERT INTO product (product_id, product_type, name, person) VALUES (?1, ?2, 1, 1)",
            rusqlite::params![product_id, product_type],
        )
    }

    /// A database created before version 11: product table without the CHECK,
    /// `user_version` 0, with a product, its synonym and a bookmark.
    fn legacy_db(product_type: &str) -> Connection {
        let db = connect();
        let schema = include_str!("resources/shema.sql");
        assert!(schema.contains(CHECK));
        db.execute_batch(&schema.replace(CHECK, "")).unwrap();
        assert_eq!(user_version(&db), 0);

        db.execute_batch(
            "INSERT INTO name (name_id, name_label) VALUES (1, 'ETHANOL'), (2, 'ETOH');
             INSERT INTO person (person_id, person_email) VALUES (1, 'admin@example.com');",
        )
        .unwrap();
        insert_product(&db, 10, product_type).unwrap();
        db.execute_batch(
            "INSERT INTO productsynonyms (productsynonyms_product_id, productsynonyms_name_id) VALUES (10, 2);
             INSERT INTO bookmark (person, product) VALUES (1, 10);",
        )
        .unwrap();
        db
    }

    #[test]
    fn test_new_database_rejects_invalid_product_type() {
        let mut db = connect();
        create_tables(&mut db).unwrap();
        assert_eq!(user_version(&db), SCHEMA_VERSION);

        db.execute_batch(
            "INSERT INTO name (name_id, name_label) VALUES (1, 'ETHANOL');
             INSERT INTO person (person_id, person_email) VALUES (1, 'admin@example.com');",
        )
        .unwrap();
        for (product_id, product_type) in [(1, "chem"), (2, "bio"), (3, "cons")] {
            insert_product(&db, product_id, product_type).unwrap();
        }

        let error = insert_product(&db, 4, "xyz").unwrap_err();
        assert!(
            error.to_string().contains("CHECK constraint failed"),
            "{error}"
        );
        let error = db
            .execute(
                "UPDATE product SET product_type = 'Chem' WHERE product_id = 1",
                [],
            )
            .unwrap_err();
        assert!(
            error.to_string().contains("CHECK constraint failed"),
            "{error}"
        );
    }

    #[test]
    fn test_migration_adds_check_and_keeps_data() {
        let mut db = legacy_db("bio");
        // Without the CHECK, invalid values are accepted.
        insert_product(&db, 11, "cons").unwrap();
        let indexes_before = count(
            &db,
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND tbl_name = 'product'",
        );

        create_tables(&mut db).unwrap();

        assert_eq!(user_version(&db), SCHEMA_VERSION);
        // Rows, references and indexes survive the rebuild.
        assert_eq!(
            count(
                &db,
                "SELECT COUNT(*) FROM product WHERE product_id = 10 AND product_type = 'bio' AND name = 1"
            ),
            1
        );
        assert_eq!(count(&db, "SELECT COUNT(*) FROM product"), 2);
        assert_eq!(
            count(
                &db,
                "SELECT COUNT(*) FROM productsynonyms WHERE productsynonyms_product_id = 10"
            ),
            1
        );
        assert_eq!(
            count(&db, "SELECT COUNT(*) FROM bookmark WHERE product = 10"),
            1
        );
        assert_eq!(
            count(&db, "SELECT COUNT(*) FROM pragma_foreign_key_check"),
            0
        );
        assert_eq!(
            count(
                &db,
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND tbl_name = 'product'"
            ),
            indexes_before
        );
        assert_eq!(
            count(
                &db,
                "SELECT COUNT(*) FROM sqlite_master WHERE name = 'product_new'"
            ),
            0
        );

        // The CHECK is now enforced.
        let error = insert_product(&db, 12, "xyz").unwrap_err();
        assert!(
            error.to_string().contains("CHECK constraint failed"),
            "{error}"
        );

        // Running again is a no-op.
        create_tables(&mut db).unwrap();
        assert_eq!(count(&db, "SELECT COUNT(*) FROM product"), 2);
    }

    #[test]
    fn test_migration_refuses_invalid_product_types() {
        let mut db = legacy_db("xyz");

        let error = create_tables(&mut db).unwrap_err();
        assert_eq!(
            error.downcast_ref::<MigrationError>(),
            Some(&MigrationError::InvalidProductTypes(1))
        );

        // Nothing changed: still version 0, no CHECK, data intact.
        assert_eq!(user_version(&db), 0);
        assert_eq!(
            count(
                &db,
                "SELECT COUNT(*) FROM product WHERE product_type = 'xyz'"
            ),
            1
        );
        insert_product(&db, 11, "still-accepted").unwrap();
    }

    #[test]
    fn test_migration_restores_foreign_keys_setting() {
        let mut db = legacy_db("chem");
        db.pragma_update(None, "foreign_keys", true).unwrap();

        create_tables(&mut db).unwrap();

        let foreign_keys: bool = db
            .pragma_query_value(None, "foreign_keys", |row| row.get(0))
            .unwrap();
        assert!(foreign_keys);
    }
}
