use log::info;
use rusqlite::{Batch, Connection};
use std::path::Path;
use std::{env, fs};

use crate::define::{
    CATEGORIES, CMR_CAS, HAZARD_STATEMENTS, PRECAUTIONARY_STATEMENTS, PRODUCERS, SIGNAL_WORDS,
    SUPPLIERS, SYMBOLS, TAGS,
};

// Temporary function to connect and initialize the database.
// Called from Go code.
// Remove me after Rust code is ready.
pub fn connect_and_init_db(db_path: &str) -> Result<(), rusqlite::Error> {
    let mut db_connection = connect(db_path)?;
    init_db(&mut db_connection)?;
    Ok(())
}

pub fn connect(db_path: &str) -> Result<Connection, rusqlite::Error> {
    let sql_extension_dir = match env::var("SQLITE_EXTENSION_DIR") {
        Ok(val) => val,
        Err(_) => panic!("Missing SQLITE_EXTENSION_DIR environment variable."),
    };

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

pub fn init_db(db_connection: &mut Connection) -> Result<(), rusqlite::Error> {
    // https://sqlite.org/stricttables.html
    // INTEGER
    // REAL
    // TEXT
    // BLOB
    // ANY
    let sql = fs::read_to_string("/tmp/shema.sql").expect("Can not read shema.sql file.");

    info!("creating database structure");

    let mut batch = Batch::new(db_connection, &sql);
    while let Some(mut stmt) = batch.next()? {
        stmt.execute([])?;
    }

    let tx = db_connection.transaction()?;

    info!("- adding tags");
    for tag in TAGS {
        tx.execute("INSERT INTO tag (tag_label) VALUES (?1)", [tag])?;
    }

    info!("- adding categories");
    for category in CATEGORIES {
        tx.execute(
            "INSERT INTO category (category_label) VALUES (?1)",
            [category],
        )?;
    }

    info!("- adding suppliers");
    for supplier in SUPPLIERS {
        tx.execute(
            "INSERT INTO supplier (supplier_label) VALUES (?1)",
            [supplier],
        )?;
    }

    info!("- adding producers");
    for producer in PRODUCERS {
        tx.execute(
            "INSERT INTO producer (producer_label) VALUES (?1)",
            [producer],
        )?;
    }

    info!("- adding signal words");
    for signal_word in SIGNAL_WORDS {
        tx.execute(
            "INSERT INTO signal_word (signal_word_label) VALUES (?1)",
            [signal_word],
        )?;
    }

    info!("- adding symbols");
    for symbol in SYMBOLS {
        tx.execute("INSERT INTO symbol (symbol_label) VALUES (?1)", [symbol])?;
    }

    info!("- adding precautionary statements");
    for precautionary_statement in PRECAUTIONARY_STATEMENTS {
        let (label, reference) = precautionary_statement;
        tx.execute(
            "INSERT INTO precautionary_statement (precautionary_statement_label, precautionary_statement_reference) VALUES (?1, ?2)",
            [label, reference],
        )?;
    }

    info!("- adding hazard statements");
    for hazard_statement in HAZARD_STATEMENTS {
        let (label, reference) = hazard_statement;
        tx.execute(
            "INSERT INTO hazard_statement (hazard_statement_label, hazard_statement_reference) VALUES (?1, ?2)",
            [label, reference],
        )?;
    }

    info!("- adding CMR CAS numbers");
    for cmr_cas in CMR_CAS {
        let (cas, cmr) = cmr_cas;
        tx.execute(
            "INSERT INTO cas_number (cas_number_label, cas_number_cmr) VALUES (?1, ?2)",
            [cas, cmr],
        )?;
    }

    info!("- adding units");
    tx.execute("INSERT INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (1,'L',1.0,'quantity',NULL)", ())?;
    tx.execute("INSERT INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (2,'mL',0.001,'quantity',1)", ())?;
    tx.execute("INSERT INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (3,'µL',1.0e-06,'quantity',1)", ())?;
    tx.execute("INSERT INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (5,'g',1.0,'quantity',NULL)", ())?;
    tx.execute("INSERT INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (4,'kg',1000.0,'quantity',5)", ())?;
    tx.execute("INSERT INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (6,'mg',0.001,'quantity',5)", ())?;
    tx.execute("INSERT INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (7,'µg',1.0e-06,'quantity',5)", ())?;
    tx.execute("INSERT INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (8,'m',1.0,'quantity',NULL)", ())?;
    tx.execute("INSERT INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (9,'dm',10.0,'quantity',8)", ())?;
    tx.execute("INSERT INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (10,'cm',100.0,'quantity',8)", ())?;
    tx.execute(
        "INSERT INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (11,'°K',1.0,'temperature',NULL)",
        (),
    )?;
    tx.execute("INSERT INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (12,'°F',1.0,'temperature',11)", ())?;
    tx.execute("INSERT INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (13,'°C',1.0,'temperature',11)", ())?;
    tx.execute(
        "INSERT INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (14,'nM',1.0,'concentration',NULL)",
        (),
    )?;
    tx.execute(
        "INSERT INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (16,'mM',1.0,'concentration',16)",
        (),
    )?;
    tx.execute(
        "INSERT INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (15,'µM',1.0,'concentration',16)",
        (),
    )?;
    tx.execute(
        "INSERT INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (20,'g/L',1.0,'concentration',NULL)",
        (),
    )?;
    tx.execute(
        "INSERT INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (17,'ng/L',1.0,'concentration',20)",
        (),
    )?;
    tx.execute(
        "INSERT INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (18,'µg/L',1.0,'concentration',20)",
        (),
    )?;
    tx.execute(
        "INSERT INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (19,'mg/L',1.0,'concentration',20)",
        (),
    )?;
    tx.execute(
        "INSERT INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (21,'%',1.0,'concentration',NULL)",
        (),
    )?;
    tx.execute(
        "INSERT INTO unit (unit_id, unit_label, unit_multiplier, unit_type, unit)  VALUES (22,'X',1.0,'concentration',NULL)",
        (),
    )?;

    info!("- adding chimitheque admin");
    tx.execute(
        "INSERT INTO person (person_id, person_email) VALUES (1, 'admin@chimitheque.fr')",
        (),
    )?;
    tx.execute(
        "INSERT INTO permission (person, permission_name, permission_item, permission_entity) VALUES (1, 'all', 'all', -1)",
        (),
    )?;

    tx.execute(
        "INSERT INTO unit (unit_label, unit_multiplier, unit_type) VALUES ('g/mol', 1, 'molecularweight');",
        (),
    )?;

    tx.commit()
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
