use chimitheque_types::bookmark::Bookmark as BookmarkStruct;
use log::debug;
use rusqlite::Connection;
use rusqlite::Row;
use sea_query::Expr;
use sea_query::Iden;
use sea_query::Query;
use sea_query::SqliteQueryBuilder;
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

pub fn toggle_product_bookmark(
    db_connection: &mut Connection,
    person_id: u64,
    product_id: u64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!("person_id: {person_id:?} product_id:{product_id:?}");

    let db_transaction = db_connection.transaction()?;

    // Does a bookmark exists for this product and person?
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

    debug!("exist_sql: {}", exist_sql.clone().as_str());
    debug!("exist_values: {exist_values:?}");

    // Perform exist query.
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
    if bookmark_exists {
        // Delete bookmark.
        let (delete_sql, delete_values) = Query::delete()
            .from_table(Bookmark::Table)
            .and_where(Expr::col((Bookmark::Table, Bookmark::Person)).eq(person_id))
            .and_where(Expr::col((Bookmark::Table, Bookmark::Product)).eq(product_id))
            .build_rusqlite(SqliteQueryBuilder);

        debug!("delete_sql: {}", delete_sql.clone().as_str());
        debug!("delete_values: {delete_values:?}");

        // Perform delete query.
        let mut stmt = db_transaction.prepare(delete_sql.as_str())?;
        stmt.execute(&*delete_values.as_params())?;
    } else {
        // Insert bookmark.
        let (insert_sql, insert_values) = Query::insert()
            .into_table(Bookmark::Table)
            .columns([Bookmark::Person, Bookmark::Product])
            .values([person_id.into(), product_id.into()])?
            .build_rusqlite(SqliteQueryBuilder);

        debug!("insert_sql: {}", insert_sql.clone().as_str());
        debug!("insert_values: {insert_values:?}");

        // Perform insert query.
        let mut stmt = db_transaction.prepare(insert_sql.as_str())?;
        stmt.execute(&*insert_values.as_params())?;
    }

    db_transaction.commit()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    // Helper function to verify that a bookmark exists in the database
    fn bookmark_exists(
        db_connection: &Connection,
        person_id: u64,
        product_id: u64,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        // Create query using sea_query
        let (count_sql, count_values) = Query::select()
            .expr(Expr::col((Bookmark::Table, Bookmark::BookmarkId)).count())
            .from(Bookmark::Table)
            .and_where(Expr::col((Bookmark::Table, Bookmark::Person)).eq(person_id))
            .and_where(Expr::col((Bookmark::Table, Bookmark::Product)).eq(product_id))
            .build_rusqlite(SqliteQueryBuilder);

        debug!("Exist check query: {}", count_sql.clone().as_str());

        // Perform count query
        let mut stmt = db_connection.prepare(count_sql.as_str())?;
        let count: i64 = stmt.query_row(&*count_values.as_params(), |row| row.get(0))?;

        Ok(count > 0)
    }

    #[test]
    fn test_toggle_product_bookmark() {
        let mut db_connection = crate::test_utils::init_test();

        // Test the base case where a bookmark doesn't exist yet
        let person_id = 1;
        let product_id = 1;

        // First toggle should create the bookmark
        toggle_product_bookmark(&mut db_connection, person_id, product_id).unwrap();
        assert!(bookmark_exists(&db_connection, person_id, product_id).unwrap());

        // Second toggle should remove the bookmark
        toggle_product_bookmark(&mut db_connection, person_id, product_id).unwrap();
        assert!(!bookmark_exists(&db_connection, person_id, product_id).unwrap());
    }

    #[test]
    fn test_toggle_product_bookmark_invalid_input() {
        let mut db_connection = crate::test_utils::init_test();

        // First test case: Existing person, non-existing product
        let existing_person_id = 1; // We know this person exists in our test database
        let non_existing_product_id = 999_999; // Doesn't exist

        // Attempt to bookmark a non-existing product
        let result = toggle_product_bookmark(
            &mut db_connection,
            existing_person_id,
            non_existing_product_id,
        );
        assert!(result.is_err());

        // Verify no bookmark was actually created
        assert!(
            !bookmark_exists(&db_connection, existing_person_id, non_existing_product_id).unwrap()
        );

        // Second test case: Non-existing person, existing product
        let non_existing_person_id = 999_999; // Doesn't exist
        let existing_product_id = 1; // We know this product exists

        // Attempt to bookmark an existing product for a non-existing person
        let result = toggle_product_bookmark(
            &mut db_connection,
            non_existing_person_id,
            existing_product_id,
        );
        assert!(result.is_err());

        // Verify no bookmark was actually created
        assert!(
            !bookmark_exists(&db_connection, non_existing_person_id, existing_product_id).unwrap()
        );

        // Finally, verify the existing product is still properly bookmarkable
        toggle_product_bookmark(&mut db_connection, 1, existing_product_id).unwrap();
        assert!(bookmark_exists(&db_connection, 1, existing_product_id).unwrap());
    }

    #[test]
    fn test_toggle_product_bookmark_multiple_products() {
        let mut db_connection = crate::test_utils::init_test();

        let person_id = 4;

        // Toggle multiple products
        let products = [5, 6, 7];

        for &product_id in &products {
            toggle_product_bookmark(&mut db_connection, person_id, product_id).unwrap();
            assert!(bookmark_exists(&db_connection, person_id, product_id).unwrap());
        }

        // Now un-toggle them
        for &product_id in &products {
            toggle_product_bookmark(&mut db_connection, person_id, product_id).unwrap();
            assert!(!bookmark_exists(&db_connection, person_id, product_id).unwrap());
        }
    }

    #[test]
    fn test_toggle_product_bookmark_duplicate_calls() {
        let mut db_connection = crate::test_utils::init_test();

        let person_id = 7;
        let product_id = 8;

        // Toggle multiple times to ensure no errors
        for _ in 0..5 {
            toggle_product_bookmark(&mut db_connection, person_id, product_id).unwrap();
        }

        // Check the state - should be true because of odd toggles
        let exists = bookmark_exists(&db_connection, person_id, product_id).unwrap();
        assert!(exists);

        // Toggle multiple times to ensure no errors
        for _ in 0..5 {
            toggle_product_bookmark(&mut db_connection, person_id, product_id).unwrap();
        }

        // Check the state - should be false because of odd toggles
        let exists = bookmark_exists(&db_connection, person_id, product_id).unwrap();
        assert!(!exists);
    }

    #[test]
    fn test_toggle_product_bookmark_concurrent_users() {
        let mut db_connection = crate::test_utils::init_test();

        // Two different users bookmarking the same product
        let product_id = 9;

        // User 1 bookmarks
        toggle_product_bookmark(&mut db_connection, 1, product_id).unwrap();
        assert!(bookmark_exists(&db_connection, 1, product_id).unwrap());

        // User 2 bookmarks
        toggle_product_bookmark(&mut db_connection, 2, product_id).unwrap();
        assert!(bookmark_exists(&db_connection, 2, product_id).unwrap());

        // User 1 un-bookmarks
        toggle_product_bookmark(&mut db_connection, 1, product_id).unwrap();
        assert!(!bookmark_exists(&db_connection, 1, product_id).unwrap());

        // User 2's bookmark is still there
        assert!(bookmark_exists(&db_connection, 2, product_id).unwrap());
    }
}
