use chimitheque_types::requestfilter::RequestFilter;
use log::debug;
use rusqlite::{Connection, Row};
use sea_query::{Expr, Iden, Order, Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use serde::Serialize;

#[derive(Iden)]
pub enum Supplier {
    Table,
    SupplierId,
    SupplierLabel,
}

#[derive(Debug, Serialize)]
pub struct SupplierStruct {
    pub match_exact_search: bool,
    pub supplier_id: u64,
    pub supplier_label: String,
}

impl From<&Row<'_>> for SupplierStruct {
    fn from(row: &Row) -> Self {
        Self {
            supplier_id: row.get_unwrap("supplier_id"),
            supplier_label: row.get_unwrap("supplier_label"),
            match_exact_search: false,
        }
    }
}

pub fn get_suppliers(
    db_connection: &Connection,
    filter: RequestFilter,
) -> Result<(Vec<SupplierStruct>, usize), Box<dyn std::error::Error>> {
    debug!("filter:{:?}", filter);

    let (sql, values) = Query::select()
        .columns([Supplier::SupplierId, Supplier::SupplierLabel])
        .from(Supplier::Table)
        .conditions(
            filter.search.is_some(),
            |q| {
                q.and_where(
                    Expr::col(Supplier::SupplierLabel)
                        .like(format!("%{}%", filter.search.clone().unwrap())),
                );
            },
            |_| {},
        )
        .order_by(Supplier::SupplierLabel, Order::Asc)
        .build_rusqlite(SqliteQueryBuilder);

    let mut stmt = db_connection.prepare(sql.as_str())?;
    let rows = stmt.query_map(&*values.as_params(), |row| Ok(SupplierStruct::from(row)))?;

    // Result supliers and count.
    let mut suppliers = Vec::new();
    let mut count = 0;
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

        count += 1;
    }

    debug!("suppliers: {:#?}", suppliers);

    Ok((suppliers, count))
}

#[cfg(test)]
mod tests {

    use log::info;

    use crate::init::init_db;

    use super::*;

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
