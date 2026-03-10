use chimitheque_types::{borrowing::Borrowing as BorrowingStruct, person::Person as PersonStruct};
use log::debug;
use rusqlite::{Connection, Row};
use sea_query::{Expr, Iden, Query, SqliteQueryBuilder};
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

pub fn toggle_storage_borrowing(
    db_connection: &mut Connection,
    person_id: u64,
    storage_id: u64,
    borrower_id: u64,
    borrowing_comment: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!("person_id: {person_id:?} borrower_id:{borrower_id:?} storage_id:{storage_id:?}");

    let db_transaction = db_connection.transaction()?;

    // Does a borrowing exists for this storage and borrower and person?
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

    debug!("exist_sql: {}", exist_sql.clone().as_str());
    debug!("exist_values: {exist_values:?}");

    // Perform exist query.
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
    if borrowing_exists {
        // Delete borrowing.
        let (delete_sql, delete_values) = Query::delete()
            .from_table(Borrowing::Table)
            .and_where(Expr::col((Borrowing::Table, Borrowing::Person)).eq(person_id))
            .and_where(Expr::col((Borrowing::Table, Borrowing::Storage)).eq(storage_id))
            .and_where(Expr::col((Borrowing::Table, Borrowing::Borrower)).eq(borrower_id))
            .build_rusqlite(SqliteQueryBuilder);

        debug!("delete_sql: {}", delete_sql.clone().as_str());
        debug!("delete_values: {delete_values:?}");

        // Perform delete query.
        let mut stmt = db_transaction.prepare(delete_sql.as_str())?;
        stmt.execute(&*delete_values.as_params())?;
    } else {
        // Insert borrowing.
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
    use crate::borrowing::{toggle_storage_borrowing, Borrowing};
    use rusqlite::Connection;

    // Helper function to verify that a borrowing exists in the database
    fn borrowing_exists(
        db_connection: &Connection,
        person_id: u64,
        storage_id: u64,
        borrower_id: u64,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        // Create query using sea_query
        let (count_sql, count_values) = Query::select()
            .expr(Expr::col((Borrowing::Table, Borrowing::BorrowingId)).count())
            .from(Borrowing::Table)
            .and_where(Expr::col((Borrowing::Table, Borrowing::Person)).eq(person_id))
            .and_where(Expr::col((Borrowing::Table, Borrowing::Storage)).eq(storage_id))
            .and_where(Expr::col((Borrowing::Table, Borrowing::Borrower)).eq(borrower_id))
            .build_rusqlite(SqliteQueryBuilder);

        // Perform count query
        let mut stmt = db_connection.prepare(count_sql.as_str())?;
        let count: i64 = stmt.query_row(&*count_values.as_params(), |row| row.get(0))?;

        Ok(count > 0)
    }

    #[test]
    fn test_toggle_storage_borrowing() {
        let mut db_connection = crate::test_utils::init_test();

        // Get an existing person, storage, and add another person for borrower
        let person_id = 1; // Existing person
        let storage_id = 1; // // Existing storage
        let borrower_id = 2; // Existing person

        // First toggle should create the borrowing
        toggle_storage_borrowing(
            &mut db_connection,
            person_id,
            storage_id,
            borrower_id,
            Some("Test borrowing comment".to_string()),
        )
        .unwrap();

        assert!(borrowing_exists(&db_connection, person_id, storage_id, borrower_id).unwrap());

        // Second toggle should remove the borrowing
        toggle_storage_borrowing(&mut db_connection, person_id, storage_id, borrower_id, None)
            .unwrap();

        assert!(!borrowing_exists(&db_connection, person_id, storage_id, borrower_id).unwrap());
    }

    #[test]
    fn test_toggle_storage_borrowing_non_existent_person() {
        let mut db_connection = crate::test_utils::init_test();

        let storage_id = 1; // Existing storage
        let non_existing_person_id = 999_999; // Non-existing person
        let borrower_id = 2; // Existing borrower

        // First toggle should fail because the person doesn't exist
        let result = toggle_storage_borrowing(
            &mut db_connection,
            non_existing_person_id,
            storage_id,
            borrower_id,
            Some("Test borrowing comment".to_string()),
        );

        assert!(result.is_err());
        assert!(!borrowing_exists(
            &db_connection,
            non_existing_person_id,
            storage_id,
            borrower_id
        )
        .unwrap());
    }

    #[test]
    fn test_toggle_storage_borrowing_concurent_storage() {
        let mut db_connection = crate::test_utils::init_test();

        let storage_id = 1; // Existing storage
        let person_id = 2; // Existing person
        let borrower_id_1 = 3; // Existing borrower
        let borrower_id_2 = 4; // Existing borrower

        // Create a first borrowing
        toggle_storage_borrowing(
            &mut db_connection,
            person_id,
            storage_id,
            borrower_id_1,
            Some("Test borrowing comment".to_string()),
        )
        .unwrap();

        // Then a second for the same storage that should fail
        let result = toggle_storage_borrowing(
            &mut db_connection,
            person_id,
            storage_id,
            borrower_id_2,
            Some("Test borrowing comment".to_string()),
        );

        assert!(result.is_err());
        assert!(!borrowing_exists(&db_connection, person_id, storage_id, borrower_id_2).unwrap());
    }

    #[test]
    fn test_toggle_storage_borrowing_non_existent_storage() {
        let mut db_connection = crate::test_utils::init_test();

        let person_id = 1; // Existing person
        let non_existing_storage_id = 999_999; // Non-existing storage
        let borrower_id = 2; // Existing borrower

        // First toggle should fail because the storage doesn't exist
        let result = toggle_storage_borrowing(
            &mut db_connection,
            person_id,
            non_existing_storage_id,
            borrower_id,
            Some("Test borrowing comment".to_string()),
        );

        assert!(result.is_err());
        assert!(!borrowing_exists(
            &db_connection,
            person_id,
            non_existing_storage_id,
            borrower_id
        )
        .unwrap());
    }

    #[test]
    fn test_toggle_storage_borrowing_non_existent_borrower() {
        let mut db_connection = crate::test_utils::init_test();

        let person_id = 1; // Existing person
        let storage_id = 1; // Existing storage
        let non_existing_borrower_id = 999_999; // Non-existing borrower

        // First toggle should fail because the borrower doesn't exist
        let result = toggle_storage_borrowing(
            &mut db_connection,
            person_id,
            storage_id,
            non_existing_borrower_id,
            Some("Test borrowing comment".to_string()),
        );

        assert!(result.is_err());
        assert!(!borrowing_exists(
            &db_connection,
            person_id,
            storage_id,
            non_existing_borrower_id
        )
        .unwrap());
    }

    #[test]
    fn test_toggle_storage_borrowing_with_comment() {
        let mut db_connection = crate::test_utils::init_test();

        let person_id = 1; // Existing person
        let storage_id = 1; // Existing storage
        let borrower_id = 2; // Existing borrower

        // Create borrowing with comment
        toggle_storage_borrowing(
            &mut db_connection,
            person_id,
            storage_id,
            borrower_id,
            Some("Important borrowing comment".to_string()),
        )
        .unwrap();

        // Verify the comment is stored
        let (query, params) = Query::select()
            .from(Borrowing::Table)
            .expr(Expr::col((Borrowing::Table, Borrowing::BorrowingComment)))
            .and_where(Expr::col((Borrowing::Table, Borrowing::Person)).eq(person_id))
            .and_where(Expr::col((Borrowing::Table, Borrowing::Storage)).eq(storage_id))
            .and_where(Expr::col((Borrowing::Table, Borrowing::Borrower)).eq(borrower_id))
            .build_rusqlite(SqliteQueryBuilder);

        let mut stmt = db_connection.prepare(query.as_str()).unwrap();
        let comment = stmt
            .query(&*params.as_params())
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .get::<_, String>(0)
            .unwrap();

        assert_eq!(comment, "Important borrowing comment".to_string());
    }
}
