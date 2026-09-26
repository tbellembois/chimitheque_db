use chimitheque_types::{borrowing::Borrowing as BorrowingStruct, person::Person as PersonStruct};
use log::debug;
use rusqlite::{Connection, Row};
use sea_query::{Expr, ExprTrait, Iden, Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use serde::Serialize;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Borrowing {
    Table,
    BorrowingId,
    BorrowingComment,
    Person,
    Storage,
    Borrower,
}

#[derive(Debug, Serialize, Default)]
pub struct BorrowingWrapper(pub BorrowingStruct);

impl From<&Row<'_>> for BorrowingWrapper {
    fn from(row: &Row) -> Self {
        Self({
            BorrowingStruct {
                borrowing_id: row.get_unwrap("borrowing_id"),
                borrowing_comment: row.get_unwrap("borrowing_comment"),
                person: row.get_unwrap("person"),
                storage: row.get_unwrap("storage"),
                borrower: PersonStruct {
                    person_id: row.get_unwrap("borrower_person_id"),
                    person_email: row.get_unwrap("borrower_person_email"),
                    ..Default::default()
                },
            }
        })
    }
}

// Toggles a storage borrowing for a given person in the database.
//
// This function implements a toggle operation that will:
// - Remove an existing borrowing if one exists for the specified person, storage, and borrower
// - Add a new borrowing if none exists for the specified person, storage, and borrower
//
// The operation is performed atomically using a database transaction to ensure
// data consistency. If any operation fails, the transaction will be rolled back.
//
// # Arguments
// * `db_connection` - Mutable reference to an active SQLite database connection
// * `person_id` - The ID of the person who owns the storage being borrowed
// * `storage_id` - The ID of the storage being borrowed
// * `borrower_id` - The ID of the person borrowing the storage
// * `borrowing_comment` - Optional comment about the borrowing
//
// # Returns
// * `Result<(), Box<dyn std::error::Error + Send + Sync>>` - Ok(()) on success,
//   or an error if any operation fails
pub fn toggle_storage_borrowing(
    db_connection: &mut Connection,
    person_id: u64,
    storage_id: u64,
    borrower_id: u64,
    borrowing_comment: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Log the parameters for debugging purposes
    debug!("person_id: {person_id:?} borrower_id:{borrower_id:?} storage_id:{storage_id:?}");

    // Begin a database transaction to ensure atomic operations
    let db_transaction = db_connection.transaction()?;

    // Check if a borrowing exists for this storage, borrower and person
    // This query checks if there is an existing borrowing record for the given combination
    let (exist_sql, exist_values) = Query::select()
        .expr(
            Expr::case(
                Expr::exists(
                    Query::select()
                        .expr(Expr::col((Borrowing::Table, Borrowing::BorrowingId)))
                        .from(Borrowing::Table)
                        .and_where(Expr::col((Borrowing::Table, Borrowing::Person)).eq(person_id))
                        .and_where(Expr::col((Borrowing::Table, Borrowing::Storage)).eq(storage_id))
                        .and_where(
                            Expr::col((Borrowing::Table, Borrowing::Borrower)).eq(borrower_id),
                        )
                        .take(),
                ),
                Expr::val(true),
            )
            .finally(Expr::val(false)),
        )
        .build_rusqlite(SqliteQueryBuilder);

    debug!("exist_sql: {}", exist_sql.as_str());
    debug!("exist_values: {exist_values:?}");

    // Perform exist query.
    // Execute the query to check if the borrowing exists
    let borrowing_exists: bool;
    {
        let mut stmt = db_transaction.prepare(exist_sql.as_str())?;
        let mut rows = stmt.query(&*exist_values.as_params())?;
        borrowing_exists = if let Some(row) = rows.next()? {
            row.get_unwrap(0)
        } else {
            false
        };
    }

    debug!("borrowing_exists: {borrowing_exists:?}");

    // Toggle borrowing.
    // If the borrowing exists, delete it; otherwise, insert a new one
    if borrowing_exists {
        // Delete borrowing.
        // Build and execute a DELETE query to remove the existing borrowing
        let (delete_sql, delete_values) = Query::delete()
            .from_table(Borrowing::Table)
            .and_where(Expr::col((Borrowing::Table, Borrowing::Person)).eq(person_id))
            .and_where(Expr::col((Borrowing::Table, Borrowing::Storage)).eq(storage_id))
            .and_where(Expr::col((Borrowing::Table, Borrowing::Borrower)).eq(borrower_id))
            .build_rusqlite(SqliteQueryBuilder);

        debug!("delete_sql: {}", delete_sql.as_str());
        debug!("delete_values: {delete_values:?}");

        // Perform delete query.
        // Execute the DELETE query to remove the borrowing
        let mut stmt = db_transaction.prepare(delete_sql.as_str())?;
        stmt.execute(&*delete_values.as_params())?;
    } else {
        // Insert borrowing.
        // Build and execute an INSERT query to add a new borrowing
        let (insert_sql, insert_values) = Query::insert()
            .into_table(Borrowing::Table)
            .columns([
                Borrowing::Person,
                Borrowing::Storage,
                Borrowing::Borrower,
                Borrowing::BorrowingComment,
            ])
            .values([
                person_id.into(),
                storage_id.into(),
                borrower_id.into(),
                borrowing_comment.into(),
            ])?
            .build_rusqlite(SqliteQueryBuilder);

        debug!("insert_sql: {}", insert_sql.as_str());
        debug!("insert_values: {insert_values:?}");

        // Perform insert query.
        // Execute the INSERT query to add the new borrowing
        let mut stmt = db_transaction.prepare(insert_sql.as_str())?;
        stmt.execute(&*insert_values.as_params())?;
    }

    // Commit the transaction to finalize the changes
    db_transaction.commit()?;

    Ok(())
}

#[cfg(test)]
#[path = "borrowing_tests.rs"]
mod borrowing_tests;
