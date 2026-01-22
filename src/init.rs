use log::{debug, info};
use regex::Regex;
use rusqlite::{Batch, Connection, Transaction};
use std::path::Path;
use std::{env, fs};

use crate::define::{
    CATEGORIES, CLASSES_OF_COMPOUNDS, CMR_CAS, PHYSICAL_STATES, PRODUCERS, SIGNAL_WORDS, SUPPLIERS,
    SYMBOLS, TAGS,
};

pub fn connect_test() -> Connection {
    let sql_extension_dir = env::var("SQLITE_EXTENSION_DIR")
        .expect("Missing SQLITE_EXTENSION_DIR environment variable.");
    let sql_extension_regex = Path::new(sql_extension_dir.as_str()).join("regexp.so");

    let db_connection = Connection::open_in_memory().unwrap();
    unsafe {
        db_connection
            .load_extension(sql_extension_regex, None)
            .expect("Unable to load regexp extension.")
    };

    db_connection
}

pub fn connect(db_path: &str) -> Result<Connection, rusqlite::Error> {
    let sql_extension_dir = env::var("SQLITE_EXTENSION_DIR")
        .expect("Missing SQLITE_EXTENSION_DIR environment variable.");
    let sql_extension_regex = Path::new(sql_extension_dir.as_str()).join("regexp.so");

    let db_connection = Connection::open(db_path)?;
    unsafe {
        db_connection
            .load_extension(sql_extension_regex, None)
            .expect("Unable to load regexp extension.")
    };

    Ok(db_connection)
}

pub fn insert_fake_values(db_connection: &mut Connection) -> Result<(), rusqlite::Error> {
    let sql = fs::read_to_string("/tmp/sample.sql").expect("Can not read sample.sql file.");

    info!("adding fake database values");

    let mut batch = Batch::new(db_connection, &sql);
    while let Some(mut stmt) = batch.next()? {
        stmt.execute([])?;
    }

    Ok(())
}

pub fn init_db(
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
        "INSERT OR IGNORE INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (14,'nM',1.0,'concentration',NULL)",
        (),
    )?;
    tx.execute(
        "INSERT OR IGNORE INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (16,'mM',1.0,'concentration',16)",
        (),
    )?;
    tx.execute(
        "INSERT OR IGNORE INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (15,'µM',1.0,'concentration',16)",
        (),
    )?;
    tx.execute(
        "INSERT OR IGNORE INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (20,'g/L',1.0,'concentration',NULL)",
        (),
    )?;
    tx.execute(
        "INSERT OR IGNORE INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (17,'ng/L',1.0,'concentration',20)",
        (),
    )?;
    tx.execute(
        "INSERT OR IGNORE INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (18,'µg/L',1.0,'concentration',20)",
        (),
    )?;
    tx.execute(
        "INSERT OR IGNORE INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (19,'mg/L',1.0,'concentration',20)",
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
        "INSERT OR IGNORE INTO permission (person, permission_name, permission_item, permission_entity) VALUES (1, 'all', 'all', -1)",
        (),
    )?;

    info!("- adding GHS statements");
    update_ghs_statements(&tx)?;

    tx.commit()?;

    Ok(())
}

// https://pubchem.ncbi.nlm.nih.gov/ghs/
pub fn update_ghs_statements(
    db_transaction: &Transaction,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let hazard_statement_re =
        Regex::new(r"(?P<reference>(EU){0,1}H[0-9]+)(\t)(?P<label>[^\t]+)(\t)")?;
    let precautionary_statement_re = Regex::new(r"(?P<reference>P[0-9+]+)(\t)(?P<label>[^\t]+)")?;

    let file = include_str!("resources/ghscode_11.txt");
    for line in file.lines() {
        // debug!("{:?}", line);

        if let Some(captures) = hazard_statement_re.captures(line) {
            let reference = captures.name("reference").unwrap().as_str();
            let label = captures.name("label").unwrap().as_str();

            debug!("reference: {reference}");
            debug!("label: {label}");

            db_transaction.execute(
            "INSERT OR REPLACE INTO hazard_statement (hazard_statement_label, hazard_statement_reference) VALUES (?1, ?2);",
            (&label, &reference),
            )?;
        } else if let Some(captures) = precautionary_statement_re.captures(line) {
            let reference = captures.name("reference").unwrap().as_str();
            let label = captures.name("label").unwrap().as_str();

            debug!("reference: {reference}");
            debug!("label: {label}");

            db_transaction.execute(
            "INSERT OR REPLACE INTO precautionary_statement (precautionary_statement_label, precautionary_statement_reference) VALUES (?1, ?2);",
            (&label, &reference),
            )?;
        };
    }

    Ok(())
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

        std::env::set_var(
            "SQLITE_EXTENSION_DIR",
            "/home/thbellem/workspace/workspace_rust/chimitheque_db/src/extensions",
        );

        assert!(connect("/tmp/storage.db").is_ok());
    }

    #[test]
    fn test_init_db() {
        init_logger();

        std::env::set_var(
            "SQLITE_EXTENSION_DIR",
            "/home/thbellem/workspace/workspace_rust/chimitheque_db/src/extensions",
        );

        // let mut db_connection = Connection::open_in_memory().unwrap();
        let mut db_connection = Connection::open("/tmp/storage.db").unwrap();
        let mayerr_initdb = init_db(&mut db_connection);

        info!("mayerr_initdb: {:?}", mayerr_initdb);

        assert!(mayerr_initdb.is_ok());
    }
}
