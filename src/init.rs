use chimitheque_types::{
    casnumber::CasNumber, cenumber::CeNumber, empiricalformula::EmpiricalFormula,
    linearformula::LinearFormula, name::Name, requestfilter::RequestFilter,
};
use log::{debug, error, info};
use rusqlite::{Batch, Connection, OpenFlags, Transaction, fallible_iterator::FallibleIterator};
use std::env;
use std::path::Path;

use crate::{
    define::{
        CATEGORIES, CLASSES_OF_COMPOUNDS, CMR_CAS, HAZARD_STATEMENT_RE, PHYSICAL_STATES,
        PRECAUTIONARY_STATEMENT_RE, PRODUCERS, SIGNAL_WORDS, SUPPLIERS, SYMBOLS, TAGS,
    },
    searchable::{create_update, get_many},
};

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
                        return Err(err);
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
                        return Err(err);
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
                        return Err(err);
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
                        return Err(err);
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
                        return Err(err);
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
    tx.execute("INSERT OR IGNORE INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (1,'L',1.0,'quantity',NULL)", ())?;
    tx.execute("INSERT OR IGNORE INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (2,'mL',0.001,'quantity',1)", ())?;
    tx.execute("INSERT OR IGNORE INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (3,'µL',1.0e-06,'quantity',1)", ())?;
    tx.execute("INSERT OR IGNORE INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (5,'g',1.0,'quantity',NULL)", ())?;
    tx.execute("INSERT OR IGNORE INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (4,'kg',1000.0,'quantity',5)", ())?;
    tx.execute("INSERT OR IGNORE INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (6,'mg',0.001,'quantity',5)", ())?;
    tx.execute("INSERT OR IGNORE INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (7,'µg',1.0e-06,'quantity',5)", ())?;
    tx.execute("INSERT OR IGNORE INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (8,'m',1.0,'quantity',NULL)", ())?;
    tx.execute("INSERT OR IGNORE INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (9,'dm',10.0,'quantity',8)", ())?;
    tx.execute("INSERT OR IGNORE INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (10,'cm',100.0,'quantity',8)", ())?;
    tx.execute(
        "INSERT OR IGNORE INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (11,'°K',1.0,'temperature',NULL)",
        (),
    )?;
    tx.execute("INSERT OR IGNORE INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (12,'°F',1.0,'temperature',11)", ())?;
    tx.execute("INSERT OR IGNORE INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (13,'°C',1.0,'temperature',11)", ())?;
    tx.execute(
        "INSERT OR IGNORE INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (16,'mM',1.0,'concentration',NULL)",
        (),
    )?;
    tx.execute(
        "INSERT OR IGNORE INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (14,'nM',1.0e-06,'concentration',16)",
        (),
    )?;
    tx.execute(
        "INSERT OR IGNORE INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (15,'µM',1.0e-03,'concentration',16)",
        (),
    )?;
    tx.execute(
        "INSERT OR IGNORE INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (20,'g/L',1.0,'concentration',NULL)",
        (),
    )?;
    tx.execute(
        "INSERT OR IGNORE INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (17,'ng/L',1.0e-09,'concentration',20)",
        (),
    )?;
    tx.execute(
        "INSERT OR IGNORE INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (18,'µg/L',1.0e-06,'concentration',20)",
        (),
    )?;
    tx.execute(
        "INSERT OR IGNORE INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (19,'mg/L',1.0e-03,'concentration',20)",
        (),
    )?;
    tx.execute(
        "INSERT OR IGNORE INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (21,'%',1.0,'concentration',NULL)",
        (),
    )?;
    tx.execute(
        "INSERT OR IGNORE INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (22,'X',1.0,'concentration',NULL)",
        (),
    )?;
    tx.execute(
        "INSERT OR IGNORE INTO unit (unit_label, unit_multiplier, unit_type) VALUES ('g/mol', 1, 'molecular_weight');",
        (),
    )?;

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
    fn update_ghs_statements_success() {
        init_test();
        let mut db_connection = connect_test();
        create_tables(&mut db_connection).unwrap();
        populate_db_with_base_data(&mut db_connection).unwrap();
        let tx = db_connection.transaction().unwrap();
        assert!(update_ghs_statements(&tx).is_ok());
    }
}
