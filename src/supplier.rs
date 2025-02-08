use chimitheque_types::{requestfilter::RequestFilter, supplier::Supplier as SupplierStruct};
use chimitheque_utils::string::{clean, Transform};
use log::debug;
use rusqlite::{Connection, Row};
use sea_query::{Expr, Iden, Order, Query, SimpleExpr, SqliteQueryBuilder};
use sea_query_rusqlite::{RusqliteBinder, RusqliteValues};
use serde::Serialize;

#[derive(Iden)]
pub enum Supplier {
    Table,
    SupplierId,
    SupplierLabel,
}

#[derive(Debug, Serialize)]
pub struct SupplierWrapper(pub SupplierStruct);

impl From<&Row<'_>> for SupplierWrapper {
    fn from(row: &Row) -> Self {
        Self({
            SupplierStruct {
                supplier_id: row.get_unwrap("supplier_id"),
                supplier_label: row.get_unwrap("supplier_label"),
                match_exact_search: false,
            }
        })
    }
}

pub fn get_suppliers(
    db_connection: &Connection,
    filter: RequestFilter,
) -> Result<(Vec<SupplierStruct>, usize), Box<dyn std::error::Error>> {
    debug!("filter:{:?}", filter);

    // Create common query statement.
    let mut expression = Query::select();
    expression.from(Supplier::Table).conditions(
        filter.search.is_some(),
        |q| {
            q.and_where(
                Expr::col(Supplier::SupplierLabel)
                    .like(format!("%{}%", filter.search.clone().unwrap())),
            );
        },
        |_| {},
    );

    // Create count query.
    let (count_sql, count_values) = expression
        .clone()
        .expr(Expr::col((Supplier::Table, Supplier::SupplierId)).count_distinct())
        .build_rusqlite(SqliteQueryBuilder);

    debug!("count_sql: {}", count_sql.clone().as_str());
    debug!("count_values: {:?}", count_values);

    // Create select query.
    let (select_sql, select_values) = expression
        .columns([Supplier::SupplierId, Supplier::SupplierLabel])
        .order_by(Supplier::SupplierLabel, Order::Asc)
        .conditions(
            filter.limit.is_some(),
            |q| {
                q.limit(filter.limit.unwrap());
            },
            |_| {},
        )
        .conditions(
            filter.offset.is_some(),
            |q| {
                q.offset(filter.offset.unwrap());
            },
            |_| {},
        )
        .build_rusqlite(SqliteQueryBuilder);

    debug!("select_sql: {}", select_sql.clone().as_str());
    debug!("select_values: {:?}", select_values);

    // Perform count query.
    let mut stmt = db_connection.prepare(count_sql.as_str())?;
    let mut rows = stmt.query(&*count_values.as_params())?;
    let count: usize = if let Some(row) = rows.next()? {
        row.get_unwrap(0)
    } else {
        0
    };

    // Perform select query.
    let mut stmt = db_connection.prepare(select_sql.as_str())?;
    let rows = stmt.query_map(&*select_values.as_params(), |row| {
        Ok(SupplierWrapper::from(row).0)
    })?;

    // Build result.
    let mut suppliers = Vec::new();
    for maybe_supplier in rows {
        let mut supplier = maybe_supplier?;

        // Set match_exact_search for supplier matching filter.search.
        if filter.search.is_some() && supplier.supplier_label.eq(&filter.search.clone().unwrap()) {
            supplier.match_exact_search = true;

            // Inserting the supplier at the beginning of the results.
            suppliers.insert(0, supplier)
        } else {
            // Inserting the supplier at the end of the results.
            suppliers.push(supplier);
        }
    }

    debug!("suppliers: {:#?}", suppliers);

    Ok((suppliers, count))
}

pub fn create_update_supplier(
    db_connection: &Connection,
    supplier: SupplierStruct,
) -> Result<u64, Box<dyn std::error::Error>> {
    debug!("create_update_supplier: {:#?}", supplier);

    let clean_supplier_label = clean(&supplier.supplier_label, Transform::None);

    // Update request: list of (columns, values) pairs to insert.
    let columns_values = vec![(Supplier::SupplierLabel, clean_supplier_label.clone().into())];

    // Create request: list of columns and values to insert.
    let columns = vec![Supplier::SupplierLabel];
    let values = vec![SimpleExpr::Value(clean_supplier_label.into())];

    let sql_query: String;
    let mut sql_values: RusqliteValues = RusqliteValues(vec![]);

    if let Some(supplier_id) = supplier.supplier_id {
        // Update query.
        (sql_query, sql_values) = Query::update()
            .table(Supplier::Table)
            .values(columns_values)
            .and_where(Expr::col(Supplier::SupplierId).eq(supplier_id))
            .build_rusqlite(SqliteQueryBuilder);
    } else {
        // Insert query.
        sql_query = Query::insert()
            .into_table(Supplier::Table)
            .columns(columns)
            .values(values)?
            .to_string(SqliteQueryBuilder);
    }

    debug!("sql_query: {}", sql_query.clone().as_str());
    debug!("sql_values: {:?}", sql_values);

    _ = db_connection.execute(&sql_query, &*sql_values.as_params())?;

    let last_insert_update_id: u64;

    if let Some(supplier_id) = supplier.supplier_id {
        last_insert_update_id = supplier_id;
    } else {
        last_insert_update_id = db_connection.last_insert_rowid().try_into()?;
    }

    debug!("last_insert_update_id: {}", last_insert_update_id);

    Ok(last_insert_update_id)
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::init::init_db;
    use log::info;

    fn init_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    fn init_test_db() -> Connection {
        let mut db_connection = Connection::open_in_memory().unwrap();
        init_db(&mut db_connection).unwrap();

        // insert fake suppliers.
        let _ = db_connection
            .execute(
                "INSERT INTO supplier (supplier_label) VALUES (?1)",
                [String::from("FAKE_SUPPLIER")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO supplier (supplier_label) VALUES (?1)",
                [String::from("FAKE_SUPPLIER ONE")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO supplier (supplier_label) VALUES (?1)",
                [String::from("FAKE_SUPPLIER TWO")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO supplier (supplier_label) VALUES (?1)",
                [String::from("FAKE_SUPPLIER THREE")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO supplier (supplier_label) VALUES (?1)",
                [String::from("AAA FAKE_SUPPLIER")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO supplier (supplier_label) VALUES (?1)",
                [String::from("YET ANOTHER SUPPLIER")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO supplier (supplier_label) VALUES (?1)",
                [String::from("12345")],
            )
            .unwrap();

        db_connection
    }

    #[test]
    fn test_get_suppliers() {
        init_logger();

        let db_connection = init_test_db();

        info!("testing ok result");
        assert!(get_suppliers(
            &db_connection,
            RequestFilter {
                ..Default::default()
            },
        )
        .is_ok());

        info!("testing filter search");
        let filter = RequestFilter {
            search: Some(String::from("FAKE_SUPPLIER")),
            ..Default::default()
        };
        let (suppliers, count) = get_suppliers(&db_connection, filter).unwrap();

        // expected number of results.
        assert_eq!(count, 5);
        // expected exact match appears first.
        assert!(suppliers[0].supplier_label.eq("FAKE_SUPPLIER"))
    }
}
