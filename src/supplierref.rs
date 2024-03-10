use chimitheque_types::requestfilter::RequestFilter;
use log::debug;
use rusqlite::{Connection, Row};
use sea_query::{Alias, Expr, Iden, Order, Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use serde::Serialize;

use crate::supplier::{Supplier, SupplierStruct};

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
enum Supplierref {
    Table,
    SupplierrefId,
    SupplierrefLabel,
    Supplier,
}

#[derive(Debug, Serialize)]
pub struct SupplierrefStruct {
    match_exact_search: bool,
    supplierref_id: u64,
    supplierref_label: String,

    supplier: SupplierStruct,
}

impl From<&Row<'_>> for SupplierrefStruct {
    fn from(row: &Row) -> Self {
        Self {
            match_exact_search: false,
            supplierref_id: row.get_unwrap("supplierref_id"),
            supplierref_label: row.get_unwrap("supplierref_label"),
            supplier: SupplierStruct {
                match_exact_search: false,
                supplier_id: row.get_unwrap("supplier.supplier_id"),
                supplier_label: row.get_unwrap("supplier.supplier_label"),
            },
        }
    }
}

pub fn get_supplierrefs(
    db_connection: &Connection,
    filter: RequestFilter,
) -> Result<(Vec<SupplierrefStruct>, usize), Box<dyn std::error::Error>> {
    debug!("filter:{:?}", filter);

    let (sql, values) = Query::select()
        .columns([Supplierref::SupplierrefId, Supplierref::SupplierrefLabel])
        .expr_as(
            Expr::col((Supplier::Table, Supplier::SupplierId)),
            Alias::new("supplier.supplier_id"),
        )
        .expr_as(
            Expr::col((Supplier::Table, Supplier::SupplierLabel)),
            Alias::new("supplier.supplier_label"),
        )
        .from(Supplierref::Table)
        .left_join(
            Supplier::Table,
            Expr::col((Supplierref::Table, Supplierref::Supplier))
                .equals((Supplier::Table, Supplier::SupplierId)),
        )
        .conditions(
            filter.search.is_some(),
            |q| {
                q.and_where(
                    Expr::col(Supplierref::SupplierrefLabel)
                        .like(format!("%{}%", filter.search.clone().unwrap())),
                );
            },
            |_| {},
        )
        .conditions(
            filter.supplier.is_some(),
            |q| {
                q.and_where(Expr::col(Supplierref::Supplier).eq(filter.supplier.unwrap()));
            },
            |_| {},
        )
        .order_by(Supplierref::SupplierrefLabel, Order::Asc)
        .build_rusqlite(SqliteQueryBuilder);

    let mut stmt = db_connection.prepare(sql.as_str())?;
    let rows = stmt.query_map(&*values.as_params(), |row| Ok(SupplierrefStruct::from(row)))?;

    // Result supliers and count.
    let mut supplierrefs = Vec::new();
    let mut count = 0;
    for maybe_supplierref in rows {
        let mut supplierref = maybe_supplierref?;

        // Set match_exact_search for supplierref matching filter.search.
        if filter.search.is_some()
            && supplierref
                .supplierref_label
                .eq(&filter.search.clone().unwrap())
        {
            supplierref.match_exact_search = true;

            // Inserting the supplier at the beginning of the results.
            supplierrefs.insert(0, supplierref)
        } else {
            // Inserting the supplier at the end of the results.
            supplierrefs.push(supplierref);
        }

        count += 1;
    }

    debug!("supplierrefs: {:#?}", supplierrefs);

    Ok((supplierrefs, count))
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
                "INSERT INTO supplier (supplier_id, supplier_label) VALUES (?1, ?2)",
                [String::from("300"), String::from("FAKE_SUPPLIER_1")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO supplier (supplier_id, supplier_label) VALUES (?1, ?2)",
                [String::from("301"), String::from("FAKE_SUPPLIER_2")],
            )
            .unwrap();

        // insert fake supplierrefs.
        let _ = db_connection
            .execute(
                "INSERT INTO supplierref (supplierref_label, supplier) VALUES (?1, ?2)",
                [String::from("1_ref1"), String::from("300")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO supplierref (supplierref_label, supplier) VALUES (?1, ?2)",
                [String::from("1_ref2"), String::from("300")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO supplierref (supplierref_label, supplier) VALUES (?1, ?2)",
                [String::from("1234"), String::from("300")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO supplierref (supplierref_label, supplier) VALUES (?1, ?2)",
                [String::from("12"), String::from("300")],
            )
            .unwrap();

        let _ = db_connection
            .execute(
                "INSERT INTO supplierref (supplierref_label, supplier) VALUES (?1, ?2)",
                [String::from("2_ref1"), String::from("301")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO supplierref (supplierref_label, supplier) VALUES (?1, ?2)",
                [String::from("2_ref2"), String::from("301")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO supplierref (supplierref_label, supplier) VALUES (?1, ?2)",
                [String::from("1234"), String::from("301")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO supplierref (supplierref_label, supplier) VALUES (?1, ?2)",
                [String::from("22"), String::from("301")],
            )
            .unwrap();

        db_connection
    }

    #[test]
    fn test_get_supplierrefs() {
        init_logger();

        let db_connection = init_test_db();

        info!("testing total result");
        let filter = RequestFilter {
            ..Default::default()
        };
        let (_, count) = get_supplierrefs(&db_connection, filter).unwrap();

        // expected number of results.
        assert_eq!(count, 8);

        info!("testing filter search");
        let filter = RequestFilter {
            search: Some(String::from("1_ref1")),
            ..Default::default()
        };
        let (supplierrefs, count) = get_supplierrefs(&db_connection, filter).unwrap();

        // expected number of results.
        assert_eq!(count, 1);
        // expected correct supplier.
        assert!(supplierrefs[0]
            .supplier
            .supplier_label
            .eq("FAKE_SUPPLIER_1"))
    }
}
