//! Schema migrations for existing databases.
//!
//! `create_tables` only creates missing tables (`CREATE TABLE IF NOT EXISTS`), so a
//! change to an existing table needs a migration. `migrate` is called at the end of
//! `create_tables` and applies each migration whose version is above the database's
//! `PRAGMA user_version`, then records the new version.

use log::info;
use rusqlite::Connection;
use std::fmt::{Display, Formatter};

/// Schema version after all migrations below have been applied.
pub const SCHEMA_VERSION: i64 = 11;

#[derive(Debug, PartialEq, Eq)]
pub enum MigrationError {
    /// Some products have a `product_type` other than `chem`, `bio` or `cons`.
    InvalidProductTypes(u64),
    /// Rebuilding a table left foreign key violations.
    ForeignKeyViolations(u64),
}

impl Display for MigrationError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            MigrationError::InvalidProductTypes(count) => write!(
                f,
                "{count} product(s) have a product_type other than chem, bio or cons; fix them before upgrading"
            ),
            MigrationError::ForeignKeyViolations(count) => {
                write!(f, "migration would leave {count} foreign key violation(s)")
            }
        }
    }
}

impl std::error::Error for MigrationError {}

// Version 11: add CHECK ("product_type" IN ('chem', 'bio', 'cons')) to the product table.
// SQLite can not add a CHECK to an existing table, so the table is rebuilt following
// https://www.sqlite.org/lang_altertable.html#otheralter
// This is a snapshot of the product table and its indexes in shema.sql at version 11.
const V11_CREATE_PRODUCT_NEW: &str = include_str!("resources/migration_v11_create_product.sql");
const V11_REPLACE_PRODUCT: &str = include_str!("resources/migration_v11_replace_product.sql");

/// Applies pending migrations to bring the database to `SCHEMA_VERSION`.
pub fn migrate(
    db_connection: &mut Connection,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let version: i64 = db_connection.pragma_query_value(None, "user_version", |row| row.get(0))?;

    if version < 11 {
        info!("migrating database from version {version} to 11");
        migrate_to_v11(db_connection)?;
    }

    Ok(())
}

fn migrate_to_v11(
    db_connection: &mut Connection,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Refuse to migrate rather than lose or reject rows silently.
    let invalid_count: u64 = db_connection.query_row(
        "SELECT COUNT(*) FROM product WHERE product_type NOT IN ('chem', 'bio', 'cons')",
        [],
        |row| row.get(0),
    )?;
    if invalid_count > 0 {
        return Err(Box::new(MigrationError::InvalidProductTypes(invalid_count)));
    }

    // Foreign keys must be off while the referenced table is replaced. The pragma
    // has no effect inside a transaction, so it is changed around it.
    let foreign_keys: bool =
        db_connection.pragma_query_value(None, "foreign_keys", |row| row.get(0))?;
    db_connection.pragma_update(None, "foreign_keys", false)?;

    let result = rebuild_product_table(db_connection);

    // Restore the previous setting whatever the outcome.
    db_connection.pragma_update(None, "foreign_keys", foreign_keys)?;

    result
}

fn rebuild_product_table(
    db_connection: &mut Connection,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Rolled back when dropped without commit, leaving the database unchanged on error.
    let transaction = db_connection.transaction()?;

    transaction.execute_batch(V11_CREATE_PRODUCT_NEW)?;

    // Copy only the columns present in both tables, so an older product table
    // missing a column still migrates.
    let columns = {
        let mut statement = transaction.prepare(
            "SELECT old.name FROM pragma_table_info('product') AS old
             JOIN pragma_table_info('product_new') AS new ON new.name = old.name
             ORDER BY old.cid",
        )?;
        let names = statement
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        names
            .iter()
            .map(|name| format!("\"{name}\""))
            .collect::<Vec<_>>()
            .join(", ")
    };
    transaction.execute(
        &format!("INSERT INTO product_new ({columns}) SELECT {columns} FROM product"),
        [],
    )?;

    transaction.execute_batch(V11_REPLACE_PRODUCT)?;

    let violations: u64 =
        transaction.query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
            row.get(0)
        })?;
    if violations > 0 {
        return Err(Box::new(MigrationError::ForeignKeyViolations(violations)));
    }

    transaction.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    transaction.commit()?;

    Ok(())
}

#[cfg(test)]
#[path = "migration_tests.rs"]
mod migration_tests;
