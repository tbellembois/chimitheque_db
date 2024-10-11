use std::{fs, io::Write};

use log::info;
use rusqlite::{Batch, Connection};

use crate::define::{
    CATEGORIES, CMR_CAS, HAZARD_STATEMENTS, PRECAUTIONARY_STATEMENTS, PRODUCERS, SIGNAL_WORDS,
    SUPPLIERS, SYMBOLS, TAGS,
};

pub fn connect(db_path: &str) -> Result<Connection, rusqlite::Error> {
    // let regexp_extension = include_bytes!("extensions/regexp.so");

    // let mut regexp_extension_file = NamedTempFile::new().expect("Unable to create temp file.");
    // regexp_extension_file
    //     .write_all(regexp_extension)
    //     .expect("Unable to write temp file.");
    // fs::write("/home/thbellem/ext.so", regexp_extension)
    // .expect("Unable to write regexp extension file.");

    println!("{:?}", std::env::current_exe());

    let db_connection = Connection::open(db_path)?;
    unsafe {
        db_connection
            .load_extension(
                "/home/thbellem/workspace/workspace_rust/chimitheque_db/src/extensions/regexp.so",
                None,
            )
            .expect("Unable to load regexp extension.")
    };

    Ok(db_connection)
}

pub fn insert_fake_values(db_connection: &mut Connection) -> Result<(), rusqlite::Error> {
    let sql = fs::read_to_string("./src/resources/storage_fake.sql")
        .expect("Can not read storage_fake.sql file.");

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
    let sql = r#"
        DROP TABLE IF EXISTS bookmark;
        CREATE TABLE bookmark (
            bookmark_id	INTEGER,
            person	INTEGER NOT NULL,
            product	INTEGER NOT NULL,
            FOREIGN KEY(person) REFERENCES person(person_id),
            FOREIGN KEY(product) REFERENCES product(product_id),
            PRIMARY KEY(bookmark_id)
        ) STRICT;

        DROP TABLE IF EXISTS borrowing;
        CREATE TABLE borrowing (
            borrowing_id	INTEGER,
            borrowing_comment	TEXT,
            person	INTEGER NOT NULL,
            borrower	INTEGER NOT NULL,
            storage	INTEGER NOT NULL UNIQUE,
            FOREIGN KEY(person) REFERENCES person(person_id),
            FOREIGN KEY(storage) REFERENCES storage(storage_id),
            FOREIGN KEY(borrower) REFERENCES person(person_id),
            PRIMARY KEY(borrowing_id)
        ) STRICT;

        DROP TABLE IF EXISTS casnumber;
        CREATE TABLE casnumber (
            casnumber_id	INTEGER,
            casnumber_label	TEXT NOT NULL UNIQUE,
            casnumber_cmr	TEXT,
            PRIMARY KEY(casnumber_id)
        ) STRICT;

        DROP TABLE IF EXISTS category;
        CREATE TABLE category (
            category_id	INTEGER,
            category_label	TEXT NOT NULL UNIQUE,
            PRIMARY KEY(category_id)
        ) STRICT;

        DROP TABLE IF EXISTS cenumber;
        CREATE TABLE cenumber (
            cenumber_id	INTEGER,
            cenumber_label	TEXT NOT NULL UNIQUE,
            PRIMARY KEY(cenumber_id)
        ) STRICT;

        DROP TABLE IF EXISTS classofcompound;
        CREATE TABLE classofcompound (
            classofcompound_id	INTEGER,
            classofcompound_label	TEXT NOT NULL UNIQUE,
            PRIMARY KEY(classofcompound_id)
        ) STRICT;

        DROP TABLE IF EXISTS empiricalformula;
        CREATE TABLE empiricalformula (
            empiricalformula_id	INTEGER,
            empiricalformula_label	TEXT NOT NULL UNIQUE,
            PRIMARY KEY(empiricalformula_id)
        ) STRICT;

        DROP TABLE IF EXISTS entity;
        CREATE TABLE entity (
            entity_id	INTEGER,
            entity_name	TEXT NOT NULL UNIQUE,
            entity_description	TEXT,
            PRIMARY KEY(entity_id)
        ) STRICT;

        DROP TABLE IF EXISTS entitypeople;
        CREATE TABLE entitypeople (
            entitypeople_entity_id	INTEGER NOT NULL,
            entitypeople_person_id	INTEGER NOT NULL,
            PRIMARY KEY(entitypeople_entity_id,entitypeople_person_id),
            FOREIGN KEY(entitypeople_person_id) REFERENCES person(person_id),
            FOREIGN KEY(entitypeople_entity_id) REFERENCES entity(entity_id)
        ) STRICT;

        DROP TABLE IF EXISTS hazardstatement;
        CREATE TABLE hazardstatement (
            hazardstatement_id	INTEGER,
            hazardstatement_label	TEXT NOT NULL,
            hazardstatement_reference	TEXT NOT NULL UNIQUE,
            hazardstatement_cmr	TEXT,
            PRIMARY KEY(hazardstatement_id)
        ) STRICT;

        DROP TABLE IF EXISTS linearformula;
        CREATE TABLE linearformula (
            linearformula_id	INTEGER,
            linearformula_label	TEXT NOT NULL UNIQUE,
            PRIMARY KEY(linearformula_id)
        ) STRICT;

        DROP TABLE IF EXISTS name;
        CREATE TABLE name (
            name_id	INTEGER,
            name_label	TEXT NOT NULL UNIQUE,
            PRIMARY KEY(name_id)
        ) STRICT;

        DROP TABLE IF EXISTS permission;
        CREATE TABLE permission (
            permission_id	INTEGER,
            person	INTEGER NOT NULL,
            permission_perm_name	TEXT NOT NULL,
            permission_item_name	TEXT NOT NULL,
            permission_entity_id	INTEGER,
            FOREIGN KEY(person) REFERENCES person(person_id),
            PRIMARY KEY(permission_id)
        ) STRICT;

        DROP TABLE IF EXISTS person;
        CREATE TABLE person (
            person_id	INTEGER,
            person_email	TEXT NOT NULL UNIQUE,
            PRIMARY KEY(person_id)
        ) STRICT;

        DROP TABLE IF EXISTS personentities;
        CREATE TABLE personentities (
            personentities_person_id	INTEGER NOT NULL,
            personentities_entity_id	INTEGER NOT NULL,
            PRIMARY KEY(personentities_person_id,personentities_entity_id),
            FOREIGN KEY(personentities_person_id) REFERENCES person(person_id),
            FOREIGN KEY(personentities_entity_id) REFERENCES entity(entity_id)
        ) STRICT;

        DROP TABLE IF EXISTS physicalstate;
        CREATE TABLE physicalstate (
            physicalstate_id	INTEGER,
            physicalstate_label	TEXT NOT NULL UNIQUE,
            PRIMARY KEY(physicalstate_id)
        ) STRICT;

        DROP TABLE IF EXISTS precautionarystatement;
        CREATE TABLE precautionarystatement (
            precautionary_statement_id	INTEGER,
            precautionarystatement_label	TEXT NOT NULL,
            precautionarystatement_reference	TEXT NOT NULL UNIQUE,
            PRIMARY KEY(precautionary_statement_id)
        ) STRICT;

        DROP TABLE IF EXISTS producer;
        CREATE TABLE producer (
            producer_id	INTEGER,
            producer_label	TEXT NOT NULL UNIQUE,
            PRIMARY KEY(producer_id)
        ) STRICT;

        DROP TABLE IF EXISTS producerref;
        CREATE TABLE producerref (
            producerref_id	INTEGER,
            producerref_label	TEXT NOT NULL,
            producer	INTEGER,
            FOREIGN KEY(producer) REFERENCES producer(producer_id),
            PRIMARY KEY(producerref_id)
        ) STRICT;

        DROP TABLE IF EXISTS product;
        CREATE TABLE product (
            product_id	INTEGER,
            product_inchi TEXT,
            product_inchikey TEXT,
            product_canonical_smiles TEXT,
            product_specificity	TEXT,
            product_msds	TEXT,
            product_restricted	INTEGER DEFAULT 0,
            product_radioactive	INTEGER DEFAULT 0,
            product_threed_formula	TEXT,
            product_twod_formula	TEXT,
            product_disposal_comment	TEXT,
            product_remark	TEXT,
            product_qrcode	TEXT,
            product_sheet	TEXT,
            product_concentration	REAL,
            product_temperature	REAL,
            product_molecular_weight REAL,
            cas_number	INTEGER,
            ce_number	INTEGER,
            person	INTEGER NOT NULL,
            empirical_formula	INTEGER,
            linear_formula	INTEGER,
            physical_state	INTEGER,
            signal_word	INTEGER,
            name	INTEGER NOT NULL,
            producer_ref	INTEGER,
            unit_temperature	INTEGER,
            unit_molecular_weight INTEGER,
            category	INTEGER,
            product_number_per_carton	INTEGER,
            product_number_per_bag	INTEGER,
            FOREIGN KEY(person) REFERENCES person(person_id),
            FOREIGN KEY(empirical_formula) REFERENCES empiricalformula(empiricalformula_id),
            FOREIGN KEY(linear_formula) REFERENCES linearformula(linearformula_id),
            FOREIGN KEY(cas_number) REFERENCES casnumber(casnumber_id),
            FOREIGN KEY(ce_number) REFERENCES cenumber(cenumber_id),
            FOREIGN KEY(producer_ref) REFERENCES producerref(producerref_id),
            FOREIGN KEY(category) REFERENCES category(category_id),
            PRIMARY KEY(product_id),
            FOREIGN KEY(unit_temperature) REFERENCES unit(unit_id),
            FOREIGN KEY(unit_molecular_weight) REFERENCES unit(unit_id),
            FOREIGN KEY(physical_state) REFERENCES physicalstate(physicalstate_id),
            FOREIGN KEY(signal_word) REFERENCES signalword(signalword_id),
            FOREIGN KEY(name) REFERENCES name(name_id)
        ) STRICT;

        DROP TABLE IF EXISTS productclassofcompound;
        CREATE TABLE productclassofcompound (
            productclassofcompound_product_id	INTEGER NOT NULL,
            productclassofcompound_classofcompound_id	INTEGER NOT NULL,
            PRIMARY KEY(productclassofcompound_product_id,productclassofcompound_classofcompound_id),
            FOREIGN KEY(productclassofcompound_product_id) REFERENCES product(product_id),
            FOREIGN KEY(productclassofcompound_classofcompound_id) REFERENCES classofcompound(classofcompound_id)
        ) STRICT;

        DROP TABLE IF EXISTS producthazardstatements;
        CREATE TABLE producthazardstatements (
            producthazardstatements_product_id	INTEGER NOT NULL,
            producthazardstatements_hazardstatement_id	INTEGER NOT NULL,
            PRIMARY KEY(producthazardstatements_product_id,producthazardstatements_hazardstatement_id),
            FOREIGN KEY(producthazardstatements_product_id) REFERENCES product(product_id),
            FOREIGN KEY(producthazardstatements_hazardstatement_id) REFERENCES hazardstatement(hazardstatement_id)
        ) STRICT;

        DROP TABLE IF EXISTS productprecautionarystatements;
        CREATE TABLE productprecautionarystatements (
            productprecautionarystatements_product_id	INTEGER NOT NULL,
            productprecautionarystatements_precautionary_statement_id	INTEGER NOT NULL,
            PRIMARY KEY(productprecautionarystatements_product_id,productprecautionarystatements_precautionary_statement_id),
            FOREIGN KEY(productprecautionarystatements_product_id) REFERENCES product(product_id),
            FOREIGN KEY(productprecautionarystatements_precautionary_statement_id) REFERENCES precautionarystatement(precautionary_statement_id)
        ) STRICT;

        DROP TABLE IF EXISTS productsupplierrefs;
        CREATE TABLE productsupplierrefs (
            productsupplierrefs_product_id	INTEGER NOT NULL,
            productsupplierrefs_supplierref_id	INTEGER NOT NULL,
            PRIMARY KEY(productsupplierrefs_product_id,productsupplierrefs_supplierref_id),
            FOREIGN KEY(productsupplierrefs_product_id) REFERENCES product(product_id),
            FOREIGN KEY(productsupplierrefs_supplierref_id) REFERENCES supplierref(supplierref_id)
        ) STRICT;

        DROP TABLE IF EXISTS productsymbols;
        CREATE TABLE productsymbols (
            productsymbols_product_id	INTEGER NOT NULL,
            productsymbols_symbol_id	INTEGER NOT NULL,
            PRIMARY KEY(productsymbols_product_id,productsymbols_symbol_id),
            FOREIGN KEY(productsymbols_product_id) REFERENCES product(product_id),
            FOREIGN KEY(productsymbols_symbol_id) REFERENCES symbol(symbol_id)
        ) STRICT;

        DROP TABLE IF EXISTS productsynonyms;
        CREATE TABLE productsynonyms (
            productsynonyms_product_id	INTEGER NOT NULL,
            productsynonyms_name_id	INTEGER NOT NULL,
            PRIMARY KEY(productsynonyms_product_id,productsynonyms_name_id),
            FOREIGN KEY(productsynonyms_product_id) REFERENCES product(product_id),
            FOREIGN KEY(productsynonyms_name_id) REFERENCES name(name_id)
        ) STRICT;

        DROP TABLE IF EXISTS producttags;
        CREATE TABLE producttags (
            producttags_product_id	INTEGER NOT NULL,
            producttags_tag_id	INTEGER NOT NULL,
            PRIMARY KEY(producttags_product_id,producttags_tag_id),
            FOREIGN KEY(producttags_product_id) REFERENCES product(product_id),
            FOREIGN KEY(producttags_tag_id) REFERENCES tag(tag_id)
        ) STRICT;

        DROP TABLE IF EXISTS signalword;
        CREATE TABLE signalword (
            signalword_id	INTEGER,
            signalword_label	TEXT NOT NULL UNIQUE,
            PRIMARY KEY(signalword_id)
        ) STRICT;

        DROP TABLE IF EXISTS storage;
        CREATE TABLE storage (
            storage_id	INTEGER,
            storage_creationdate	TEXT NOT NULL,
            storage_modificationdate	TEXT NOT NULL,
            storage_entrydate	TEXT,
            storage_exitdate	TEXT,
            storage_openingdate	TEXT,
            storage_expirationdate	TEXT,
            storage_quantity	REAL,
            storage_barecode	TEXT,
            storage_comment	TEXT,
            storage_reference	TEXT,
            storage_batchnumber	TEXT,
            storage_todestroy	INTEGER DEFAULT 0,
            storage_archive	INTEGER DEFAULT 0,
            storage_qrcode	BLOB,
            storage_concentration	REAL,
            storage_number_of_unit	INTEGER,
            storage_number_of_bag	INTEGER,
            storage_number_of_carton	INTEGER,
            person	INTEGER NOT NULL,
            product	INTEGER NOT NULL,
            storelocation	INTEGER NOT NULL,
            unit_concentration	REAL,
            unit_quantity	REAL,
            supplier	INTEGER,
            storage	INTEGER,
            FOREIGN KEY(unit_concentration) REFERENCES unit(unit_id),
            FOREIGN KEY(storage) REFERENCES storage(storage_id),
            FOREIGN KEY(unit_quantity) REFERENCES unit(unit_id),
            FOREIGN KEY(supplier) REFERENCES supplier(supplier_id),
            FOREIGN KEY(person) REFERENCES person(person_id),
            FOREIGN KEY(product) REFERENCES product(product_id),
            FOREIGN KEY(storelocation) REFERENCES storelocation(storelocation_id),
            PRIMARY KEY(storage_id)
        ) STRICT;

        DROP TABLE IF EXISTS storelocation;
        CREATE TABLE storelocation (
            storelocation_id	INTEGER,
            storelocation_name	TEXT NOT NULL,
            storelocation_color	TEXT,
            storelocation_canstore	INTEGER DEFAULT 0,
            storelocation_fullpath	TEXT,
            entity	INTEGER NOT NULL,
            storelocation	INTEGER,
            FOREIGN KEY(storelocation) REFERENCES storelocation(storelocation_id),
            FOREIGN KEY(entity) REFERENCES entity(entity_id),
            PRIMARY KEY(storelocation_id)
        ) STRICT;

        DROP TABLE IF EXISTS supplier;
        CREATE TABLE supplier (
            supplier_id	INTEGER,
            supplier_label	TEXT NOT NULL UNIQUE,
            PRIMARY KEY(supplier_id)
        ) STRICT;

        DROP TABLE IF EXISTS supplierref;
        CREATE TABLE supplierref (
            supplierref_id	INTEGER,
            supplierref_label	TEXT NOT NULL,
            supplier	INTEGER,
            FOREIGN KEY(supplier) REFERENCES supplier(supplier_id),
            PRIMARY KEY(supplierref_id)
        ) STRICT;

        DROP TABLE IF EXISTS symbol;
        CREATE TABLE symbol (
            symbol_id	INTEGER,
            symbol_label	TEXT NOT NULL UNIQUE,
            PRIMARY KEY(symbol_id)
        ) STRICT;

        DROP TABLE IF EXISTS tag;
        CREATE TABLE tag (
            tag_id	INTEGER,
            tag_label	TEXT NOT NULL UNIQUE,
            PRIMARY KEY(tag_id)
        ) STRICT;

        DROP TABLE IF EXISTS unit;
        CREATE TABLE unit (
            unit_id	INTEGER,
            unit_label	TEXT NOT NULL UNIQUE,
            unit_multiplier	REAL NOT NULL DEFAULT 1,
            unit_type	TEXT,
            unit	INTEGER,
            FOREIGN KEY(unit) REFERENCES unit(unit_id),
            PRIMARY KEY(unit_id)
        ) STRICT;

        DROP TABLE IF EXISTS welcomeannounce;
        CREATE TABLE welcomeannounce (
            welcomeannounce_id	INTEGER,
            welcomeannounce_text	TEXT,
            PRIMARY KEY(welcomeannounce_id)
        ) STRICT;

        DROP INDEX IF EXISTS idx_casnumber;
        CREATE UNIQUE INDEX idx_casnumber ON casnumber (
            casnumber_label
        );
        DROP INDEX IF EXISTS idx_category_label;
        CREATE UNIQUE INDEX idx_category_label ON category (
            category_label
        );
        DROP INDEX IF EXISTS idx_cenumber;
        CREATE UNIQUE INDEX idx_cenumber ON cenumber (
            cenumber_label
        );
        DROP INDEX IF EXISTS idx_classofcompound;
        CREATE UNIQUE INDEX idx_classofcompound ON classofcompound (
            classofcompound_label
        );
        DROP INDEX IF EXISTS idx_empiricalformula;
        CREATE UNIQUE INDEX idx_empiricalformula ON empiricalformula (
            empiricalformula_label
        );
        DROP INDEX IF EXISTS idx_entity;
        CREATE UNIQUE INDEX idx_entity ON entity (
            entity_name
        );
        DROP INDEX IF EXISTS idx_entitypeople;
        CREATE UNIQUE INDEX idx_entitypeople ON entitypeople (
            entitypeople_entity_id,
            entitypeople_person_id
        );
        DROP INDEX IF EXISTS idx_hazardstatement;
        CREATE UNIQUE INDEX idx_hazardstatement ON hazardstatement (
            hazardstatement_reference
        );
        DROP INDEX IF EXISTS idx_linearformula;
        CREATE UNIQUE INDEX idx_linearformula ON linearformula (
            linearformula_label
        );
        DROP INDEX IF EXISTS idx_name;
        CREATE UNIQUE INDEX idx_name ON name (
            name_label
        );
        DROP INDEX IF EXISTS idx_permission;
        CREATE UNIQUE INDEX idx_permission ON permission (
            person,
            permission_item_name,
            permission_perm_name,
            permission_entity_id
        );
        DROP INDEX IF EXISTS idx_permission_entity_id;
        CREATE INDEX idx_permission_entity_id ON permission (
            permission_entity_id	ASC
        );
        DROP INDEX IF EXISTS idx_permission_item_name;
        CREATE INDEX idx_permission_item_name ON permission (
            permission_item_name	ASC
        );
        DROP INDEX IF EXISTS idx_permission_perm_name;
        CREATE INDEX idx_permission_perm_name ON permission (
            permission_perm_name	ASC
        );
        DROP INDEX IF EXISTS idx_permission_person;
        CREATE INDEX idx_permission_person ON permission (
            person	ASC
        );
        DROP INDEX IF EXISTS idx_person;
        CREATE UNIQUE INDEX idx_person ON person (
            person_email
        );
        DROP INDEX IF EXISTS idx_personentities;
        CREATE UNIQUE INDEX idx_personentities ON personentities (
            personentities_person_id,
            personentities_entity_id
        );
        DROP INDEX IF EXISTS idx_precautionarystatement;
        CREATE UNIQUE INDEX idx_precautionarystatement ON precautionarystatement (
            precautionarystatement_reference
        );
        DROP INDEX IF EXISTS idx_producer_label;
        CREATE UNIQUE INDEX idx_producer_label ON producer (
            producer_label
        );
        DROP INDEX IF EXISTS idx_producerref_label;

        DROP INDEX IF EXISTS idx_product_casnumber;
        CREATE UNIQUE INDEX idx_product_casnumber ON product (
            product_id,
            cas_number
        );
        DROP INDEX IF EXISTS idx_product_cenumber;
        CREATE UNIQUE INDEX idx_product_cenumber ON product (
            product_id,
            ce_number
        );
        DROP INDEX IF EXISTS idx_product_empiricalformula;
        CREATE UNIQUE INDEX idx_product_empiricalformula ON product (
            product_id,
            empirical_formula
        );
        DROP INDEX IF EXISTS idx_product_name;
        CREATE UNIQUE INDEX idx_product_name ON product (
            product_id,
            name
        );
        DROP INDEX IF EXISTS idx_productclassofcompound;
        CREATE UNIQUE INDEX idx_productclassofcompound ON productclassofcompound (
            productclassofcompound_product_id,
            productclassofcompound_classofcompound_id
        );
        DROP INDEX IF EXISTS idx_producthazardstatements;
        CREATE UNIQUE INDEX idx_producthazardstatements ON producthazardstatements (
            producthazardstatements_product_id,
            producthazardstatements_hazardstatement_id
        );
        DROP INDEX IF EXISTS idx_productprecautionarystatements;
        CREATE UNIQUE INDEX idx_productprecautionarystatements ON productprecautionarystatements (
            productprecautionarystatements_product_id,
            productprecautionarystatements_precautionary_statement_id
        );
        DROP INDEX IF EXISTS idx_productsupplierrefs;
        CREATE UNIQUE INDEX idx_productsupplierrefs ON productsupplierrefs (
            productsupplierrefs_product_id,
            productsupplierrefs_supplierref_id
        );
        DROP INDEX IF EXISTS idx_productsymbols;
        CREATE UNIQUE INDEX idx_productsymbols ON productsymbols (
            productsymbols_product_id,
            productsymbols_symbol_id
        );
        DROP INDEX IF EXISTS idx_productsynonyms;
        CREATE UNIQUE INDEX idx_productsynonyms ON productsynonyms (
            productsynonyms_product_id,
            productsynonyms_name_id
        );
        DROP INDEX IF EXISTS idx_producttags;
        CREATE UNIQUE INDEX idx_producttags ON producttags (
            producttags_product_id,
            producttags_tag_id
        );
        DROP INDEX IF EXISTS idx_storage_product;
        CREATE UNIQUE INDEX idx_storage_product ON storage (
            storage_id,
            product
        );
        DROP INDEX IF EXISTS idx_storage_storelocation;
        CREATE UNIQUE INDEX idx_storage_storelocation ON storage (
            storage_id,
            storelocation
        );
        DROP INDEX IF EXISTS idx_storage_storelocation_product;
        CREATE UNIQUE INDEX idx_storage_storelocation_product ON storage (
            storage_id,
            storelocation,
            product
        );
        DROP INDEX IF EXISTS idx_supplierref_label;

        DROP INDEX IF EXISTS idx_tag_label;
        CREATE UNIQUE INDEX idx_tag_label ON tag (
            tag_label
        );
        "#;

    info!("creating database structure");

    let mut batch = Batch::new(db_connection, sql);
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
            "INSERT INTO signalword (signalword_label) VALUES (?1)",
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
            "INSERT INTO precautionarystatement (precautionarystatement_label, precautionarystatement_reference) VALUES (?1, ?2)",
            [label, reference],
        )?;
    }

    info!("- adding hazard statements");
    for hazard_statement in HAZARD_STATEMENTS {
        let (label, reference) = hazard_statement;
        tx.execute(
            "INSERT INTO hazardstatement (hazardstatement_label, hazardstatement_reference) VALUES (?1, ?2)",
            [label, reference],
        )?;
    }

    info!("- adding CMR CAS numbers");
    for cmr_cas in CMR_CAS {
        let (cas, cmr) = cmr_cas;
        tx.execute(
            "INSERT INTO casnumber (casnumber_label, casnumber_cmr) VALUES (?1, ?2)",
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

        assert!(connect("/tmp/storage.db").is_ok());
    }

    #[test]
    fn test_init_db() {
        init_logger();

        let mut db_connection = Connection::open_in_memory().unwrap();
        let mayerr_initdb = init_db(&mut db_connection);

        info!("mayerr_initdb: {:?}", mayerr_initdb);

        assert!(mayerr_initdb.is_ok());
    }
}
