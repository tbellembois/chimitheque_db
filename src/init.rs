use chimitheque_defines::{
    CATEGORIES, CLASSES_OF_COMPOUNDS, CMR_CAS, HAZARD_STATEMENT_RE, PHYSICAL_STATES,
    PRECAUTIONARY_STATEMENT_RE, PRODUCERS, SIGNAL_WORDS, SUPPLIERS, SYMBOLS, TAGS, UNITS,
};
use chimitheque_types::{
    casnumber::CasNumber, cenumber::CeNumber, empiricalformula::EmpiricalFormula,
    linearformula::LinearFormula, name::Name, requestfilter::RequestFilter,
};
use log::{debug, error, info};
use rusqlite::{
    Batch, Connection, OpenFlags, Transaction, fallible_iterator::FallibleIterator, params,
};
use std::env;
use std::path::Path;

use crate::searchable::{create_update, get_many};

#[must_use]
pub fn connect_test() -> Connection {
    let sql_extension_dir = env::var("SQLITE_EXTENSION_DIR")
        .expect("Missing SQLITE_EXTENSION_DIR environment variable.");
    let sql_extension_regex = Path::new(sql_extension_dir.as_str()).join("regex0.so");

    let db_connection = Connection::open_in_memory().unwrap();
    unsafe {
        db_connection
            .load_extension(sql_extension_regex, None::<&str>)
            .expect("Unable to load regexp extension.");
    };

    db_connection
}

pub fn connect(db_path: &str) -> Result<Connection, rusqlite::Error> {
    let sql_extension_dir = env::var("SQLITE_EXTENSION_DIR")
        .expect("Missing SQLITE_EXTENSION_DIR environment variable.");
    let sql_extension_regex = Path::new(sql_extension_dir.as_str()).join("regex0.so");

    let db_connection = Connection::open_with_flags(
        db_path,
        OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_FULL_MUTEX,
    )?;
    unsafe {
        db_connection
            .load_extension(sql_extension_regex, None::<&str>)
            .expect("Unable to load regexp extension.");
    };

    // Set journal mode to WAL and verify the change.
    db_connection
        .pragma_update_and_check(None, "journal_mode", "WAL", |mode| {
            if mode.get::<_, String>(0) == Ok("wal".to_string()) {
                Ok(())
            } else {
                Err(rusqlite::Error::SqliteFailure(
                    rusqlite::ffi::Error {
                        code: rusqlite::ffi::ErrorCode::Unknown,
                        extended_code: 0,
                    },
                    Some(format!("Failed to set WAL mode, got: {mode:?}")),
                ))
            }
        })
        .expect("Failed to set WAL mode");

    // Enable foreign keys.
    db_connection
        .execute("PRAGMA foreign_keys = ON", [])
        .expect("Failed to enable foreign keys");

    // Set synchronous mode to NORMAL.
    db_connection
        .execute("PRAGMA synchronous = NORMAL", [])
        .expect("Failed to set synchronous mode");

    // Set cache size to -65536 (64MB).
    db_connection
        .execute("PRAGMA cache_size = -65536", [])
        .expect("Failed to set cache size");

    // Set temp store to MEMORY to avoid disk I/O.
    db_connection
        .execute("PRAGMA temp_store = MEMORY", [])
        .expect("Failed to set temp store to MEMORY");

    // Vacuum and analyze.
    db_connection
        .execute("VACUUM", [])
        .expect("Failed to vacuum database");
    db_connection
        .execute("ANALYZE", [])
        .expect("Failed to analyze database");

    Ok(db_connection)
}

pub fn sanitize(
    db_connection: &mut Connection,
    skip_errors: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // CAS numbers.
    match get_many(
        &CasNumber {
            ..Default::default()
        },
        db_connection,
        &RequestFilter::default(),
    ) {
        Ok((cas_numbers, _count)) => {
            for mut cas_number in cas_numbers {
                let mayerr = cas_number.sanitize_and_validate();

                if let Err(err) = mayerr {
                    if !skip_errors {
                        return Err(err.into());
                    }

                    error!("skipping error: {err} for {cas_number:?}");
                    debug!("create_update {cas_number:?}");

                    // Even on error, we still want to update the database will sanitized entry.
                    create_update(
                        &CasNumber::default(),
                        cas_number.cas_number_id,
                        db_connection,
                        &cas_number.cas_number_label,
                    )?;
                }
            }
        }
        Err(err) => {
            if !skip_errors {
                return Err(err);
            }

            error!("skipping error: {err}");
        }
    }

    // CE numbers.
    match get_many(
        &CeNumber {
            ..Default::default()
        },
        db_connection,
        &RequestFilter::default(),
    ) {
        Ok((ce_numbers, _count)) => {
            for mut ce_number in ce_numbers {
                let mayerr = ce_number.sanitize_and_validate();

                if let Err(err) = mayerr {
                    if !skip_errors {
                        return Err(err.into());
                    }

                    error!("skipping error: {err} for {ce_number:?}");
                    debug!("create_update {ce_number:?}");

                    // Even on error, we still want to update the database will sanitized entry.
                    create_update(
                        &CeNumber::default(),
                        ce_number.ce_number_id,
                        db_connection,
                        &ce_number.ce_number_label,
                    )?;
                }
            }
        }
        Err(err) => {
            if !skip_errors {
                return Err(err);
            }

            error!("skipping error: {err}");
        }
    }

    // Names.
    match get_many(
        &Name {
            ..Default::default()
        },
        db_connection,
        &RequestFilter::default(),
    ) {
        Ok((names, _count)) => {
            for mut name in names {
                let mayerr = name.sanitize_and_validate();

                if let Err(err) = mayerr {
                    if !skip_errors {
                        return Err(err.into());
                    }

                    error!("skipping error: {err} for {name:?}");
                    debug!("create_update {name:?}");

                    // Even on error, we still want to update the database will sanitized entry.
                    create_update(
                        &Name::default(),
                        name.name_id,
                        db_connection,
                        &name.name_label,
                    )?;
                }
            }
        }
        Err(err) => {
            if !skip_errors {
                return Err(err);
            }

            error!("skipping error: {err}");
        }
    }

    // Empirical formulas.
    match get_many(
        &EmpiricalFormula {
            ..Default::default()
        },
        db_connection,
        &RequestFilter::default(),
    ) {
        Ok((empirical_formulas, _count)) => {
            for mut empirical_formula in empirical_formulas {
                let mayerr = empirical_formula.sanitize_and_validate();

                if let Err(err) = mayerr {
                    if !skip_errors {
                        return Err(err.into());
                    }

                    error!("skipping error: {err} for {empirical_formula:?}");
                    debug!("create_update {empirical_formula:?}");

                    // Even on error, we still want to update the database will sanitized entry.
                    create_update(
                        &EmpiricalFormula::default(),
                        empirical_formula.empirical_formula_id,
                        db_connection,
                        &empirical_formula.empirical_formula_label,
                    )?;
                }
            }
        }
        Err(err) => {
            if !skip_errors {
                return Err(err);
            }

            error!("skipping error: {err}");
        }
    }

    // Linear formulas.
    match get_many(
        &LinearFormula {
            ..Default::default()
        },
        db_connection,
        &RequestFilter::default(),
    ) {
        Ok((linear_formulas, _count)) => {
            for mut linear_formula in linear_formulas {
                let mayerr = linear_formula.sanitize_and_validate();

                if let Err(err) = mayerr {
                    if !skip_errors {
                        return Err(err.into());
                    }

                    error!("skipping error: {err} for {linear_formula:?}");
                    debug!("create_update {linear_formula:?}");

                    // Even on error, we still want to update the database will sanitized entry.
                    create_update(
                        &LinearFormula::default(),
                        linear_formula.linear_formula_id,
                        db_connection,
                        &linear_formula.linear_formula_label,
                    )?;
                }
            }
        }
        Err(err) => {
            if !skip_errors {
                return Err(err);
            }

            error!("skipping error: {err}");
        }
    }

    Ok(())
}

pub fn create_tables(
    db_connection: &mut Connection,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // https://sqlite.org/stricttables.html
    // INTEGER
    // REAL
    // TEXT
    // BLOB
    // ANY

    let sql = include_str!("resources/shema.sql");

    info!("creating database structure");

    let mut batch = Batch::new(db_connection, sql);
    while let Some(mut stmt) = batch.next()? {
        stmt.execute([])?;
    }

    Ok(())
}

pub fn populate_db_with_base_data(
    db_connection: &mut Connection,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let tx = db_connection.transaction()?;

    info!("- adding tags");
    for tag in TAGS {
        tx.execute("INSERT OR IGNORE INTO tag (tag_label) VALUES (?1)", [tag])?;
    }

    info!("- adding categories");
    for category in CATEGORIES {
        tx.execute(
            "INSERT OR IGNORE INTO category (category_label) VALUES (?1)",
            [category],
        )?;
    }

    info!("- adding suppliers");
    for supplier in SUPPLIERS {
        tx.execute(
            "INSERT OR IGNORE INTO supplier (supplier_label) VALUES (?1)",
            [supplier],
        )?;
    }

    info!("- adding producers");
    for producer in PRODUCERS {
        tx.execute(
            "INSERT OR IGNORE INTO producer (producer_label) VALUES (?1)",
            [producer],
        )?;
    }

    info!("- adding signal words");
    for signal_word in SIGNAL_WORDS {
        tx.execute(
            "INSERT OR IGNORE INTO signal_word (signal_word_label) VALUES (?1)",
            [signal_word],
        )?;
    }

    info!("- adding symbols");
    for symbol in SYMBOLS {
        tx.execute(
            "INSERT OR IGNORE INTO symbol (symbol_label) VALUES (?1)",
            [symbol],
        )?;
    }

    info!("- adding physical states");
    for physical_state in PHYSICAL_STATES {
        tx.execute(
            "INSERT OR IGNORE INTO physical_state (physical_state_label) VALUES (?1)",
            [physical_state],
        )?;
    }

    info!("- adding classes of compounds");
    for class_of_compound in CLASSES_OF_COMPOUNDS {
        tx.execute(
            "INSERT OR IGNORE INTO class_of_compound (class_of_compound_label) VALUES (?1)",
            [class_of_compound],
        )?;
    }

    info!("- adding CMR CAS numbers");
    for cmr_cas in CMR_CAS {
        let (cas, cmr) = cmr_cas;
        tx.execute(
            "INSERT OR IGNORE INTO cas_number (cas_number_label, cas_number_cmr) VALUES (?1, ?2)",
            [cas, cmr],
        )?;
    }

    info!("- adding units");
    for unit in UNITS {
        let (unit_id, unit_label, unit_multiplier, unit_type, unit) = unit;
        tx.execute(
            "INSERT OR IGNORE INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![unit_id, unit_label, unit_multiplier, unit_type, unit],
        )?;
    }

    info!("- adding chimitheque admin");
    tx.execute(
        "INSERT OR IGNORE INTO person (person_id, person_email) VALUES (1, 'admin@chimitheque.fr')",
        (),
    )?;
    tx.execute(
        "INSERT OR IGNORE INTO permission (person, permission_name, permission_item, permission_entity) VALUES (1, 'all', 'all', NULL)",
        (),
    )?;

    info!("- adding GHS statements");
    update_ghs_statements(&tx)?;

    tx.commit()?;

    Ok(())
}

// https://pubchem.ncbi.nlm.nih.gov/ghs/
fn update_ghs_statements(
    db_transaction: &Transaction,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let file = include_str!("resources/ghscode_11.txt");
    for line in file.lines() {
        if let Some(captures) = HAZARD_STATEMENT_RE.captures(line) {
            let reference = captures.name("reference").unwrap().as_str();
            let label = captures.name("label").unwrap().as_str();

            debug!("reference: {reference}");
            debug!("label: {label}");

            db_transaction.execute(
                "INSERT INTO hazard_statement (hazard_statement_label, hazard_statement_reference)
            VALUES (?1, ?2)
            ON CONFLICT(hazard_statement_reference) DO UPDATE
            SET hazard_statement_reference = ?2;",
                (&label, &reference),
            )?;
        } else if let Some(captures) = PRECAUTIONARY_STATEMENT_RE.captures(line) {
            let reference = captures.name("reference").unwrap().as_str();
            let label = captures.name("label").unwrap().as_str();

            debug!("reference: {reference}");
            debug!("label: {label}");

            db_transaction.execute(
                "INSERT INTO precautionary_statement (precautionary_statement_label, precautionary_statement_reference)
            VALUES (?1, ?2)
            ON CONFLICT(precautionary_statement_reference) DO UPDATE
            SET precautionary_statement_reference = ?2;",
            (&label, &reference),
            )?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_test() {
        let _ = env_logger::builder().is_test(true).try_init();
        unsafe {
            std::env::set_var("SQLITE_EXTENSION_DIR", "src/extensions");
        };
    }

    #[test]
    fn connect_success() {
        init_test();
        assert!(connect("/tmp/storage.db").is_ok());
    }

    #[test]
    fn init_db_success() {
        init_test();
        let mut db_connection = connect_test();
        create_tables(&mut db_connection).unwrap();
        assert!(populate_db_with_base_data(&mut db_connection).is_ok());
    }

    #[test]
    fn populate_db_with_base_data_success() {
        init_test();
        let mut db_connection = connect_test();
        create_tables(&mut db_connection).unwrap();
        assert!(populate_db_with_base_data(&mut db_connection).is_ok());
    }

    #[test]
    fn update_ghs_statements_success() {
        init_test();
        let mut db_connection = connect_test();
        create_tables(&mut db_connection).unwrap();
        populate_db_with_base_data(&mut db_connection).unwrap();
        let tx = db_connection.transaction().unwrap();
        assert!(update_ghs_statements(&tx).is_ok());
    }

    #[test]
    fn product_type_check_constraint() {
        init_test();
        let mut db_connection = connect_test();
        create_tables(&mut db_connection).unwrap();
        db_connection
            .execute_batch(
                "INSERT INTO name (name_id, name_label) VALUES (1, 'ETHANOL');
                 INSERT INTO person (person_id, person_email) VALUES (1, 'admin@example.com');",
            )
            .unwrap();

        let insert = |product_id: i64, product_type: &str| {
            db_connection.execute(
                "INSERT INTO product (product_id, product_type, name, person) VALUES (?1, ?2, 1, 1)",
                params![product_id, product_type],
            )
        };
        for (product_id, product_type) in [(1, "chem"), (2, "bio"), (3, "cons")] {
            insert(product_id, product_type).unwrap();
        }
        let error = insert(4, "xyz").unwrap_err();
        assert!(
            error.to_string().contains("CHECK constraint failed"),
            "{error}"
        );

        let error = db_connection
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

    /// A new database with foreign keys on, one name, person, entity and product.
    fn constraints_db() -> Connection {
        init_test();
        let mut db_connection = connect_test();
        create_tables(&mut db_connection).unwrap();
        db_connection
            .execute_batch(
                "PRAGMA foreign_keys = ON;
                 INSERT INTO name (name_id, name_label) VALUES (1, 'ETHANOL');
                 INSERT INTO person (person_id, person_email) VALUES (1, 'admin@example.com');
                 INSERT INTO entity (entity_id, entity_name) VALUES (1, 'lab');
                 INSERT INTO product (product_id, product_type, name) VALUES (1, 'chem', 1);",
            )
            .unwrap();
        db_connection
    }

    fn assert_rejected(db_connection: &Connection, sql: &str, expected: &str) {
        let error = db_connection.execute(sql, []).unwrap_err();
        assert!(error.to_string().contains(expected), "{sql}: {error}");
    }

    #[test]
    fn create_tables_is_idempotent() {
        let mut db_connection = constraints_db();
        create_tables(&mut db_connection).unwrap();
        populate_db_with_base_data(&mut db_connection).unwrap();
        create_tables(&mut db_connection).unwrap();
        let products: i64 = db_connection
            .query_row("SELECT COUNT(*) FROM product", [], |row| row.get(0))
            .unwrap();
        assert_eq!(products, 1);
    }

    #[test]
    fn schema_rejects_invalid_values() {
        let db_connection = constraints_db();
        let check = "CHECK constraint failed";
        for sql in [
            // Empty labels.
            "INSERT INTO name (name_label) VALUES ('')",
            "INSERT INTO tag (tag_label) VALUES ('')",
            "INSERT INTO person (person_email) VALUES ('')",
            "INSERT INTO store_location (store_location_name, entity) VALUES ('', 1)",
            // Booleans.
            "UPDATE product SET product_restricted = 2 WHERE product_id = 1",
            "INSERT INTO store_location (store_location_name, entity, store_location_can_store) VALUES ('a', 1, -1)",
            // Allowed values.
            "INSERT INTO unit (unit_label, unit_type) VALUES ('x', 'volume')",
            "INSERT INTO permission (person, permission_name, permission_item) VALUES (1, 'x', 'all')",
            "INSERT INTO permission (person, permission_name, permission_item) VALUES (1, 'r', 'x')",
            // Ranges.
            "UPDATE product SET product_molecular_weight = 0 WHERE product_id = 1",
            "UPDATE product SET product_number_per_carton = 0 WHERE product_id = 1",
            "UPDATE product SET product_concentration = -1 WHERE product_id = 1",
            "INSERT INTO unit (unit_label, unit_type, unit_multiplier) VALUES ('x', 'quantity', 0)",
            // Own parent.
            "INSERT INTO store_location (store_location_id, store_location_name, entity, store_location) VALUES (5, 'a', 1, 5)",
        ] {
            assert_rejected(&db_connection, sql, check);
        }

        for sql in [
            "UPDATE product SET product_restricted = NULL WHERE product_id = 1",
            "INSERT INTO unit (unit_label) VALUES ('x')",
        ] {
            assert_rejected(&db_connection, sql, "NOT NULL constraint failed");
        }
    }

    #[test]
    fn schema_accepts_valid_and_missing_optional_values() {
        let db_connection = constraints_db();
        db_connection
            .execute_batch(
                "UPDATE product SET product_restricted = 1, product_molecular_weight = 46.07,
                    product_number_per_carton = NULL, product_concentration = 0 WHERE product_id = 1;
                 INSERT INTO store_location (store_location_id, store_location_name, entity, store_location_can_store)
                    VALUES (1, 'shelf', 1, 1);
                 INSERT INTO store_location (store_location_name, entity, store_location) VALUES ('box', 1, 1);
                 INSERT INTO permission (person, permission_name, permission_item, permission_entity)
                    VALUES (1, 'r', 'products', NULL), (1, 'all', 'all', 1);",
            )
            .unwrap();
    }

    #[test]
    fn bookmark_is_unique_per_person_and_product() {
        let db_connection = constraints_db();
        db_connection
            .execute("INSERT INTO bookmark (person, product) VALUES (1, 1)", [])
            .unwrap();
        assert_rejected(
            &db_connection,
            "INSERT INTO bookmark (person, product) VALUES (1, 1)",
            "UNIQUE constraint failed",
        );
    }

    #[test]
    fn deleting_an_entity_deletes_its_permissions() {
        let db_connection = constraints_db();
        db_connection
            .execute_batch(
                "INSERT INTO permission (person, permission_name, permission_item, permission_entity)
                    VALUES (1, 'all', 'all', 1), (1, 'r', 'products', NULL);
                 DELETE FROM entity WHERE entity_id = 1;",
            )
            .unwrap();
        let permissions: Vec<Option<i64>> = db_connection
            .prepare("SELECT permission_entity FROM permission")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(permissions, [None]);

        assert_rejected(
            &db_connection,
            "INSERT INTO permission (person, permission_name, permission_item, permission_entity) VALUES (1, 'r', 'products', 99)",
            "FOREIGN KEY constraint failed",
        );
    }

    #[test]
    fn references_are_unique_per_producer_and_supplier() {
        let db_connection = constraints_db();
        db_connection
            .execute_batch(
                "INSERT INTO producer (producer_id, producer_label) VALUES (1, 'P1'), (2, 'P2');
                 INSERT INTO supplier (supplier_id, supplier_label) VALUES (1, 'S1'), (2, 'S2');
                 INSERT INTO producer_ref (producer_ref_label, producer) VALUES ('REF', 1), ('REF', 2);
                 INSERT INTO supplier_ref (supplier_ref_label, supplier) VALUES ('REF', 1), ('REF', 2);",
            )
            .unwrap();
        for sql in [
            "INSERT INTO producer_ref (producer_ref_label, producer) VALUES ('REF', 1)",
            "INSERT INTO supplier_ref (supplier_ref_label, supplier) VALUES ('REF', 1)",
        ] {
            assert_rejected(&db_connection, sql, "UNIQUE constraint failed");
        }
    }

    #[test]
    fn permissions_are_unique() {
        let db_connection = constraints_db();
        db_connection
            .execute_batch(
                "INSERT INTO permission (person, permission_name, permission_item, permission_entity)
                    VALUES (1, 'r', 'products', NULL), (1, 'all', 'all', 1);",
            )
            .unwrap();

        // Duplicates are rejected, including with a NULL entity (all entities).
        for sql in [
            "INSERT INTO permission (person, permission_name, permission_item, permission_entity) VALUES (1, 'r', 'products', NULL)",
            "INSERT INTO permission (person, permission_name, permission_item, permission_entity) VALUES (1, 'all', 'all', 1)",
        ] {
            assert_rejected(&db_connection, sql, "UNIQUE constraint failed");
        }

        // REPLACE, used when saving a person's permissions, keeps a single row.
        db_connection
            .execute(
                "REPLACE INTO permission (person, permission_name, permission_item, permission_entity) VALUES (1, 'r', 'products', NULL)",
                [],
            )
            .unwrap();
        let count: i64 = db_connection
            .query_row("SELECT COUNT(*) FROM permission", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn base_data_permissions_are_not_duplicated_at_each_startup() {
        let mut db_connection = constraints_db();
        let count_permissions = |db_connection: &Connection| -> i64 {
            db_connection
                .query_row("SELECT COUNT(*) FROM permission", [], |row| row.get(0))
                .unwrap()
        };
        populate_db_with_base_data(&mut db_connection).unwrap();
        let first = count_permissions(&db_connection);
        create_tables(&mut db_connection).unwrap();
        populate_db_with_base_data(&mut db_connection).unwrap();
        assert_eq!(count_permissions(&db_connection), first);
    }

    #[test]
    fn storage_exit_date_is_not_before_entry_date() {
        let db_connection = constraints_db();
        db_connection
            .execute(
                "INSERT INTO store_location (store_location_id, store_location_name, entity) VALUES (1, 'shelf', 1)",
                [],
            )
            .unwrap();
        let insert = |entry: Option<i64>, exit: Option<i64>| {
            db_connection.execute(
                "INSERT INTO storage (storage_entry_date, storage_exit_date, product, store_location) VALUES (?1, ?2, 1, 1)",
                params![entry, exit],
            )
        };
        insert(Some(100), Some(200)).unwrap();
        insert(Some(100), Some(100)).unwrap();
        insert(None, Some(100)).unwrap();
        insert(Some(100), None).unwrap();
        let error = insert(Some(200), Some(100)).unwrap_err();
        assert!(
            error.to_string().contains("CHECK constraint failed"),
            "{error}"
        );
    }
}
