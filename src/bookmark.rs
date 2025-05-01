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
    db_connection: &Connection,
    person_id: u64,
    product_id: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("person_id: {:?} product_id:{:?}", person_id, product_id);

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
    debug!("exist_values: {:?}", exist_values);

    // Perform exist query.
    let mut stmt = db_connection.prepare(exist_sql.as_str())?;
    let mut rows = stmt.query(&*exist_values.as_params())?;
    let bookmark_exists: bool = if let Some(row) = rows.next()? {
        row.get_unwrap(0)
    } else {
        false
    };

    debug!("bookmark_exists: {:?}", bookmark_exists);

    // Toggle bookmark.
    match bookmark_exists {
        true => {
            // Delete bookmark.
            let (delete_sql, delete_values) = Query::delete()
                .from_table(Bookmark::Table)
                .and_where(Expr::col((Bookmark::Table, Bookmark::Person)).eq(person_id))
                .and_where(Expr::col((Bookmark::Table, Bookmark::Product)).eq(product_id))
                .build_rusqlite(SqliteQueryBuilder);

            debug!("delete_sql: {}", delete_sql.clone().as_str());
            debug!("delete_values: {:?}", delete_values);

            // Perform delete query.
            let mut stmt = db_connection.prepare(delete_sql.as_str())?;
            stmt.execute(&*delete_values.as_params())?;
        }
        false => {
            // Insert bookmark.
            let (insert_sql, insert_values) = Query::insert()
                .into_table(Bookmark::Table)
                .columns([Bookmark::Person, Bookmark::Product])
                .values([person_id.into(), product_id.into()])?
                .build_rusqlite(SqliteQueryBuilder);

            debug!("insert_sql: {}", insert_sql.clone().as_str());
            debug!("insert_values: {:?}", insert_values);

            // Perform insert query.
            let mut stmt = db_connection.prepare(insert_sql.as_str())?;
            stmt.execute(&*insert_values.as_params())?;
        }
    }

    Ok(())
}
