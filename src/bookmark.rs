use chimitheque_types::bookmark::Bookmark as BookmarkStruct;
use log::debug;
use rusqlite::Connection;
use rusqlite::Row;
use sea_query::{Expr, ExprTrait, Iden, Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use serde::Serialize;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Bookmark {
    Table,
    BookmarkId,
    Person,
    Product,
}

#[derive(Debug, Serialize, Default)]
pub struct BookmarkWrapper(pub BookmarkStruct);

impl From<&Row<'_>> for BookmarkWrapper {
    fn from(row: &Row) -> Self {
        Self({
            BookmarkStruct {
                bookmark_id: row.get_unwrap("bookmark_id"),
                person: row.get_unwrap("person"),
                product: row.get_unwrap("product"),
            }
        })
    }
}

/// Toggles a product bookmark for a given person in the database.
///
/// This function implements a toggle operation that will:
/// - Remove an existing bookmark if one exists for the specified person and product
/// - Add a new bookmark if none exists for the specified person and product
///
/// The operation is performed atomically using a database transaction to ensure
/// data consistency. If any operation fails, the transaction will be rolled back.
///
/// # Arguments
/// * `db_connection` - Mutable reference to an active SQLite database connection
/// * `person_id` - The ID of the person who owns the bookmark
/// * `product_id` - The ID of the product being bookmarked
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error + Send + Sync>>` - Ok(()) on success,
///   or an error if any operation fails
///
pub fn toggle_product_bookmark(
    db_connection: &mut Connection,
    person_id: u64,
    product_id: u64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Log the person_id and product_id for debugging purposes
    debug!("person_id: {person_id:?} product_id:{product_id:?}");

    // Begin a database transaction to ensure atomic operations
    let db_transaction = db_connection.transaction()?;

    // Does a bookmark exists for this product and person?
    // This query checks if there is an existing bookmark record for the given person and product
    let (exist_sql, exist_values) = Query::select()
        .expr(
            Expr::case(
                Expr::exists(
                    Query::select()
                        .expr(Expr::col((Bookmark::Table, Bookmark::BookmarkId)))
                        .from(Bookmark::Table)
                        .and_where(Expr::col((Bookmark::Table, Bookmark::Person)).eq(person_id))
                        .and_where(Expr::col((Bookmark::Table, Bookmark::Product)).eq(product_id))
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
    // Execute the query to check if the bookmark exists
    let bookmark_exists: bool;
    {
        let mut stmt = db_transaction.prepare(exist_sql.as_str())?;
        let mut rows = stmt.query(&*exist_values.as_params())?;
        bookmark_exists = if let Some(row) = rows.next()? {
            row.get_unwrap(0)
        } else {
            false
        };
    }

    debug!("bookmark_exists: {bookmark_exists:?}");

    // Toggle bookmark.
    // If the bookmark exists, delete it; otherwise, insert a new bookmark
    if bookmark_exists {
        // Delete bookmark.
        // Build and execute a DELETE query to remove the existing bookmark
        let (delete_sql, delete_values) = Query::delete()
            .from_table(Bookmark::Table)
            .and_where(Expr::col((Bookmark::Table, Bookmark::Person)).eq(person_id))
            .and_where(Expr::col((Bookmark::Table, Bookmark::Product)).eq(product_id))
            .build_rusqlite(SqliteQueryBuilder);

        debug!("delete_sql: {}", delete_sql.as_str());
        debug!("delete_values: {delete_values:?}");

        // Perform delete query.
        // Execute the DELETE query to remove the bookmark
        let mut stmt = db_transaction.prepare(delete_sql.as_str())?;
        stmt.execute(&*delete_values.as_params())?;
    } else {
        // Insert bookmark.
        // Build and execute an INSERT query to add a new bookmark
        let (insert_sql, insert_values) = Query::insert()
            .into_table(Bookmark::Table)
            .columns([Bookmark::Person, Bookmark::Product])
            .values([person_id.into(), product_id.into()])?
            .build_rusqlite(SqliteQueryBuilder);

        debug!("insert_sql: {}", insert_sql.as_str());
        debug!("insert_values: {insert_values:?}");

        // Perform insert query.
        // Execute the INSERT query to add the new bookmark
        let mut stmt = db_transaction.prepare(insert_sql.as_str())?;
        stmt.execute(&*insert_values.as_params())?;
    }

    // Commit the transaction to finalize the changes
    db_transaction.commit()?;

    Ok(())
}
