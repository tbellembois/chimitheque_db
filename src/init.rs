use log::info;
use rusqlite::{Batch, Connection, OpenFlags};

use crate::define::{
    CATEGORIES, CMR_CAS, HAZARD_STATEMENTS, PRECAUTIONARY_STATEMENTS, PRODUCERS, SIGNAL_WORDS,
    SUPPLIERS, SYMBOLS, TAGS,
};

pub fn connect(db_path: &str) -> Result<Connection, rusqlite::Error> {
    Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
}

pub fn init_db(db_connection: &mut Connection) -> Result<(), rusqlite::Error> {
    let sql =
        "
        DROP TABLE IF EXISTS bookmark;
        CREATE TABLE bookmark (
            bookmark_id	integer,
            person	integer NOT NULL,
            product	integer NOT NULL,
            FOREIGN KEY(person) REFERENCES person(person_id),
            FOREIGN KEY(product) REFERENCES product(product_id),
            PRIMARY KEY(bookmark_id)
        );
        DROP TABLE IF EXISTS borrowing;
        CREATE TABLE borrowing (
            borrowing_id	integer,
            borrowing_comment	string,
            person	integer NOT NULL,
            borrower	integer NOT NULL,
            storage	integer NOT NULL UNIQUE,
            FOREIGN KEY(person) REFERENCES person(person_id),
            FOREIGN KEY(storage) REFERENCES storage(storage_id),
            FOREIGN KEY(borrower) REFERENCES person(person_id),
            PRIMARY KEY(borrowing_id)
        );
        DROP TABLE IF EXISTS captcha;
        CREATE TABLE captcha (
            captcha_id	integer,
            captcha_token	string NOT NULL,
            captcha_text	string NOT NULL,
            PRIMARY KEY(captcha_id)
        );
        DROP TABLE IF EXISTS casnumber;
        CREATE TABLE casnumber (
            casnumber_id	integer,
            casnumber_label	string NOT NULL UNIQUE,
            casnumber_cmr	string,
            PRIMARY KEY(casnumber_id)
        );
        DROP TABLE IF EXISTS category;
        CREATE TABLE category (
            category_id	integer,
            category_label	string NOT NULL,
            PRIMARY KEY(category_id)
        );
        DROP TABLE IF EXISTS cenumber;
        CREATE TABLE cenumber (
            cenumber_id	integer,
            cenumber_label	string NOT NULL UNIQUE,
            PRIMARY KEY(cenumber_id)
        );
        DROP TABLE IF EXISTS classofcompound;
        CREATE TABLE classofcompound (
            classofcompound_id	integer,
            classofcompound_label	string NOT NULL UNIQUE,
            PRIMARY KEY(classofcompound_id)
        );
        DROP TABLE IF EXISTS empiricalformula;
        CREATE TABLE empiricalformula (
            empiricalformula_id	integer,
            empiricalformula_label	string NOT NULL UNIQUE,
            PRIMARY KEY(empiricalformula_id)
        );
        DROP TABLE IF EXISTS entity;
        CREATE TABLE entity (
            entity_id	integer,
            entity_name	string NOT NULL UNIQUE,
            entity_description	string,
            PRIMARY KEY(entity_id)
        );
        DROP TABLE IF EXISTS entityldapgroups;
        CREATE TABLE entityldapgroups (
            entityldapgroups_entity_id	integer NOT NULL,
            entityldapgroups_ldapgroup	string NOT NULL,
            PRIMARY KEY(entityldapgroups_entity_id,entityldapgroups_ldapgroup),
            FOREIGN KEY(entityldapgroups_entity_id) REFERENCES entity(entity_id)
        );
        DROP TABLE IF EXISTS entitypeople;
        CREATE TABLE entitypeople (
            entitypeople_entity_id	integer NOT NULL,
            entitypeople_person_id	integer NOT NULL,
            PRIMARY KEY(entitypeople_entity_id,entitypeople_person_id),
            FOREIGN KEY(entitypeople_person_id) REFERENCES person(person_id),
            FOREIGN KEY(entitypeople_entity_id) REFERENCES entity(entity_id)
        );
        DROP TABLE IF EXISTS hazardstatement;
        CREATE TABLE hazardstatement (
            hazardstatement_id	integer,
            hazardstatement_label	string NOT NULL,
            hazardstatement_reference	string NOT NULL,
            hazardstatement_cmr	string,
            PRIMARY KEY(hazardstatement_id)
        );
        DROP TABLE IF EXISTS linearformula;
        CREATE TABLE linearformula (
            linearformula_id	integer,
            linearformula_label	string NOT NULL UNIQUE,
            PRIMARY KEY(linearformula_id)
        );
        DROP TABLE IF EXISTS name;
        CREATE TABLE name (
            name_id	integer,
            name_label	string NOT NULL UNIQUE,
            PRIMARY KEY(name_id)
        );
        DROP TABLE IF EXISTS permission;
        CREATE TABLE permission (
            permission_id	integer,
            person	integer NOT NULL,
            permission_perm_name	string NOT NULL,
            permission_item_name	string NOT NULL,
            permission_entity_id	integer,
            FOREIGN KEY(person) REFERENCES person(person_id),
            PRIMARY KEY(permission_id)
        );
        DROP TABLE IF EXISTS person;
        CREATE TABLE person (
            person_id	integer,
            person_email	string NOT NULL,
            person_password	string NOT NULL,
            person_aeskey	string NOT NULL,
            PRIMARY KEY(person_id)
        );
        DROP TABLE IF EXISTS personentities;
        CREATE TABLE personentities (
            personentities_person_id	integer NOT NULL,
            personentities_entity_id	integer NOT NULL,
            PRIMARY KEY(personentities_person_id,personentities_entity_id),
            FOREIGN KEY(personentities_person_id) REFERENCES person(person_id),
            FOREIGN KEY(personentities_entity_id) REFERENCES entity(entity_id)
        );
        DROP TABLE IF EXISTS physicalstate;
        CREATE TABLE physicalstate (
            physicalstate_id	integer,
            physicalstate_label	string NOT NULL UNIQUE,
            PRIMARY KEY(physicalstate_id)
        );
        DROP TABLE IF EXISTS precautionarystatement;
        CREATE TABLE precautionarystatement (
            precautionarystatement_id	integer,
            precautionarystatement_label	string NOT NULL,
            precautionarystatement_reference	string NOT NULL,
            PRIMARY KEY(precautionarystatement_id)
        );
        DROP TABLE IF EXISTS producer;
        CREATE TABLE producer (
            producer_id	integer,
            producer_label	string NOT NULL,
            PRIMARY KEY(producer_id)
        );
        DROP TABLE IF EXISTS producerref;
        CREATE TABLE producerref (
            producerref_id	integer,
            producerref_label	string NOT NULL,
            producer	integer,
            FOREIGN KEY(producer) REFERENCES producer(producer_id),
            PRIMARY KEY(producerref_id)
        );
        DROP TABLE IF EXISTS product;
        CREATE TABLE product (
            product_id	integer,
            product_specificity	string,
            product_msds	string,
            product_restricted	boolean DEFAULT 0,
            product_radioactive	boolean DEFAULT 0,
            product_threedformula	string,
            product_twodformula	string,
            product_molformula	blob,
            product_disposalcomment	string,
            product_remark	string,
            product_qrcode	string,
            product_sheet	string,
            product_concentration	integer,
            product_temperature	integer,
            casnumber	integer,
            cenumber	integer,
            person	integer NOT NULL,
            empiricalformula	integer,
            linearformula	integer,
            physicalstate	integer,
            signalword	integer,
            name	integer NOT NULL,
            producerref	integer,
            unit_temperature	integer,
            category	integer,
            product_number_per_carton	INTEGER,
            product_number_per_bag	INTEGER,
            FOREIGN KEY(person) REFERENCES person(person_id),
            FOREIGN KEY(empiricalformula) REFERENCES empiricalformula(empiricalformula_id),
            FOREIGN KEY(linearformula) REFERENCES linearformula(linearformula_id),
            FOREIGN KEY(casnumber) REFERENCES casnumber(casnumber_id),
            FOREIGN KEY(cenumber) REFERENCES cenumber(cenumber_id),
            FOREIGN KEY(producerref) REFERENCES producerref(producerref_id),
            FOREIGN KEY(category) REFERENCES category(category_id),
            PRIMARY KEY(product_id),
            FOREIGN KEY(unit_temperature) REFERENCES unit(unit_id),
            FOREIGN KEY(physicalstate) REFERENCES physicalstate(physicalstate_id),
            FOREIGN KEY(signalword) REFERENCES signalword(signalword_id),
            FOREIGN KEY(name) REFERENCES name(name_id)
        );
        DROP TABLE IF EXISTS productclassofcompound;
        CREATE TABLE productclassofcompound (
            productclassofcompound_product_id	integer NOT NULL,
            productclassofcompound_classofcompound_id	integer NOT NULL,
            PRIMARY KEY(productclassofcompound_product_id,productclassofcompound_classofcompound_id),
            FOREIGN KEY(productclassofcompound_product_id) REFERENCES product(product_id),
            FOREIGN KEY(productclassofcompound_classofcompound_id) REFERENCES classofcompound(classofcompound_id)
        );
        DROP TABLE IF EXISTS producthazardstatements;
        CREATE TABLE producthazardstatements (
            producthazardstatements_product_id	integer NOT NULL,
            producthazardstatements_hazardstatement_id	integer NOT NULL,
            PRIMARY KEY(producthazardstatements_product_id,producthazardstatements_hazardstatement_id),
            FOREIGN KEY(producthazardstatements_product_id) REFERENCES product(product_id),
            FOREIGN KEY(producthazardstatements_hazardstatement_id) REFERENCES hazardstatement(hazardstatement_id)
        );
        DROP TABLE IF EXISTS productprecautionarystatements;
        CREATE TABLE productprecautionarystatements (
            productprecautionarystatements_product_id	integer NOT NULL,
            productprecautionarystatements_precautionarystatement_id	integer NOT NULL,
            PRIMARY KEY(productprecautionarystatements_product_id,productprecautionarystatements_precautionarystatement_id),
            FOREIGN KEY(productprecautionarystatements_product_id) REFERENCES product(product_id),
            FOREIGN KEY(productprecautionarystatements_precautionarystatement_id) REFERENCES precautionarystatement(precautionarystatement_id)
        );
        DROP TABLE IF EXISTS productsupplierrefs;
        CREATE TABLE productsupplierrefs (
            productsupplierrefs_product_id	integer NOT NULL,
            productsupplierrefs_supplierref_id	integer NOT NULL,
            PRIMARY KEY(productsupplierrefs_product_id,productsupplierrefs_supplierref_id),
            FOREIGN KEY(productsupplierrefs_product_id) REFERENCES product(product_id),
            FOREIGN KEY(productsupplierrefs_supplierref_id) REFERENCES supplierref(supplierref_id)
        );
        DROP TABLE IF EXISTS productsymbols;
        CREATE TABLE productsymbols (
            productsymbols_product_id	integer NOT NULL,
            productsymbols_symbol_id	integer NOT NULL,
            PRIMARY KEY(productsymbols_product_id,productsymbols_symbol_id),
            FOREIGN KEY(productsymbols_product_id) REFERENCES product(product_id),
            FOREIGN KEY(productsymbols_symbol_id) REFERENCES symbol(symbol_id)
        );
        DROP TABLE IF EXISTS productsynonyms;
        CREATE TABLE productsynonyms (
            productsynonyms_product_id	integer NOT NULL,
            productsynonyms_name_id	integer NOT NULL,
            PRIMARY KEY(productsynonyms_product_id,productsynonyms_name_id),
            FOREIGN KEY(productsynonyms_product_id) REFERENCES product(product_id),
            FOREIGN KEY(productsynonyms_name_id) REFERENCES name(name_id)
        );
        DROP TABLE IF EXISTS producttags;
        CREATE TABLE producttags (
            producttags_product_id	integer NOT NULL,
            producttags_tag_id	integer NOT NULL,
            PRIMARY KEY(producttags_product_id,producttags_tag_id),
            FOREIGN KEY(producttags_product_id) REFERENCES product(product_id),
            FOREIGN KEY(producttags_tag_id) REFERENCES tag(tag_id)
        );
        DROP TABLE IF EXISTS signalword;
        CREATE TABLE signalword (
            signalword_id	integer,
            signalword_label	string NOT NULL UNIQUE,
            PRIMARY KEY(signalword_id)
        );
        DROP TABLE IF EXISTS storage;
        CREATE TABLE storage (
            storage_id	integer,
            storage_creationdate	datetime NOT NULL,
            storage_modificationdate	datetime NOT NULL,
            storage_entrydate	datetime,
            storage_exitdate	datetime,
            storage_openingdate	datetime,
            storage_expirationdate	datetime,
            storage_quantity	float,
            storage_barecode	text,
            storage_comment	text,
            storage_reference	text,
            storage_batchnumber	text,
            storage_todestroy	boolean DEFAULT 0,
            storage_archive	boolean DEFAULT 0,
            storage_qrcode	blob,
            storage_concentration	integer,
            storage_number_of_unit	integer,
            storage_number_of_bag	integer,
            storage_number_of_carton	integer,
            person	integer NOT NULL,
            product	integer NOT NULL,
            storelocation	integer NOT NULL,
            unit_concentration	integer,
            unit_quantity	integer,
            supplier	integer,
            storage	integer,
            FOREIGN KEY(unit_concentration) REFERENCES unit(unit_id),
            FOREIGN KEY(storage) REFERENCES storage(storage_id),
            FOREIGN KEY(unit_quantity) REFERENCES unit(unit_id),
            FOREIGN KEY(supplier) REFERENCES supplier(supplier_id),
            FOREIGN KEY(person) REFERENCES person(person_id),
            FOREIGN KEY(product) REFERENCES product(product_id),
            FOREIGN KEY(storelocation) REFERENCES storelocation(storelocation_id),
            PRIMARY KEY(storage_id)
        );
        DROP TABLE IF EXISTS storelocation;
        CREATE TABLE storelocation (
            storelocation_id	integer,
            storelocation_name	string NOT NULL,
            storelocation_color	string,
            storelocation_canstore	boolean DEFAULT 0,
            storelocation_fullpath	string,
            entity	integer NOT NULL,
            storelocation	integer,
            FOREIGN KEY(storelocation) REFERENCES storelocation(storelocation_id),
            FOREIGN KEY(entity) REFERENCES entity(entity_id),
            PRIMARY KEY(storelocation_id)
        );
        DROP TABLE IF EXISTS supplier;
        CREATE TABLE supplier (
            supplier_id	integer,
            supplier_label	string NOT NULL,
            PRIMARY KEY(supplier_id)
        );
        DROP TABLE IF EXISTS supplierref;
        CREATE TABLE supplierref (
            supplierref_id	integer,
            supplierref_label	string NOT NULL,
            supplier	integer,
            FOREIGN KEY(supplier) REFERENCES supplier(supplier_id),
            PRIMARY KEY(supplierref_id)
        );
        DROP TABLE IF EXISTS symbol;
        CREATE TABLE symbol (
            symbol_id	integer,
            symbol_label	string NOT NULL,
            symbol_image	string,
            PRIMARY KEY(symbol_id)
        );
        DROP TABLE IF EXISTS tag;
        CREATE TABLE tag (
            tag_id	integer,
            tag_label	string NOT NULL,
            PRIMARY KEY(tag_id)
        );
        DROP TABLE IF EXISTS unit;
        CREATE TABLE unit (
            unit_id	integer,
            unit_label	string NOT NULL UNIQUE,
            unit_multiplier	integer NOT NULL DEFAULT 1,
            unit_type	string,
            unit	integer,
            FOREIGN KEY(unit) REFERENCES unit(unit_id),
            PRIMARY KEY(unit_id)
        );
        DROP TABLE IF EXISTS welcomeannounce;
        CREATE TABLE welcomeannounce (
            welcomeannounce_id	integer,
            welcomeannounce_text	string,
            PRIMARY KEY(welcomeannounce_id)
        );
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
        DROP INDEX IF EXISTS idx_entityldapgroups;
        CREATE UNIQUE INDEX idx_entityldapgroups ON entityldapgroups (
            entityldapgroups_entity_id,
            entityldapgroups_ldapgroup
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
        CREATE UNIQUE INDEX idx_producerref_label ON producerref (
            producerref_label
        );
        DROP INDEX IF EXISTS idx_product_casnumber;
        CREATE UNIQUE INDEX idx_product_casnumber ON product (
            product_id,
            casnumber
        );
        DROP INDEX IF EXISTS idx_product_cenumber;
        CREATE UNIQUE INDEX idx_product_cenumber ON product (
            product_id,
            cenumber
        );
        DROP INDEX IF EXISTS idx_product_empiricalformula;
        CREATE UNIQUE INDEX idx_product_empiricalformula ON product (
            product_id,
            empiricalformula
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
            productprecautionarystatements_precautionarystatement_id
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
        CREATE UNIQUE INDEX idx_supplierref_label ON supplierref (
            supplierref_label
        );
        DROP INDEX IF EXISTS idx_tag_label;
        CREATE UNIQUE INDEX idx_tag_label ON tag (
            tag_label
        );
        ";

    info!("- creating database structure");

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

        let db_path = "/home/thbellem/S3Drive/chimitheque-db/storage.db";
        info!("connecting to {}", db_path);

        assert!(connect(db_path).is_ok());
    }
}
