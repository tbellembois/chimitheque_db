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
    debug!(
        "person_id: {:?} borrower_id:{:?} storage_id:{:?}",
        person_id, borrower_id, storage_id
    );

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
    debug!("exist_values: {:?}", exist_values);

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

    debug!("borrowing_exists: {:?}", borrowing_exists);

    // Toggle borrowing.
    match borrowing_exists {
        true => {
            // Delete borrowing.
            let (delete_sql, delete_values) = Query::delete()
                .from_table(Borrowing::Table)
                .and_where(Expr::col((Borrowing::Table, Borrowing::Person)).eq(person_id))
                .and_where(Expr::col((Borrowing::Table, Borrowing::Storage)).eq(storage_id))
                .and_where(Expr::col((Borrowing::Table, Borrowing::Borrower)).eq(borrower_id))
                .build_rusqlite(SqliteQueryBuilder);

            debug!("delete_sql: {}", delete_sql.clone().as_str());
            debug!("delete_values: {:?}", delete_values);

            // Perform delete query.
            let mut stmt = db_transaction.prepare(delete_sql.as_str())?;
            stmt.execute(&*delete_values.as_params())?;
        }
        false => {
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
            debug!("insert_values: {:?}", insert_values);

            // Perform insert query.
            let mut stmt = db_transaction.prepare(insert_sql.as_str())?;
            stmt.execute(&*insert_values.as_params())?;
        }
    }

    db_transaction.commit()?;

    Ok(())
}
