use chimitheque_types::{
    requestfilter::RequestFilter, supplier::Supplier as SupplierStruct,
    supplierref::Supplierref as SupplierrefStruct,
};
use log::debug;
use rusqlite::{Connection, Row};
use sea_query::{Alias, Expr, Iden, Order, Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use serde::Serialize;

use crate::supplier::Supplier;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
enum Supplierref {
    Table,
    SupplierrefId,
    SupplierrefLabel,
    Supplier,
}

#[derive(Debug, Serialize)]
pub struct SupplierrefWrapper(pub SupplierrefStruct);

impl From<&Row<'_>> for SupplierrefWrapper {
    fn from(row: &Row) -> Self {
        Self({
            SupplierrefStruct {
                match_exact_search: false,
                supplierref_id: row.get_unwrap("supplierref_id"),
                supplierref_label: row.get_unwrap("supplierref_label"),
                supplier: SupplierStruct {
                    match_exact_search: false,
                    supplier_id: row.get_unwrap("supplier.supplier_id"),
                    supplier_label: row.get_unwrap("supplier.supplier_label"),
                },
            }
        })
    }
}

pub fn get_supplierrefs(
    db_connection: &Connection,
    filter: RequestFilter,
) -> Result<(Vec<SupplierrefStruct>, usize), Box<dyn std::error::Error>> {
    debug!("filter:{:?}", filter);

    // Create common query statement.
    let mut expression = Query::select();
    expression
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
        );

    // Create count query.
    let (count_sql, count_values) = expression
        .clone()
        .expr(Expr::col((Supplierref::Table, Supplierref::SupplierrefId)).count_distinct())
        .build_rusqlite(SqliteQueryBuilder);

    debug!("count_sql: {}", count_sql.clone().as_str());
    debug!("count_values: {:?}", count_values);

    // Create select query.

    let (select_sql, select_values) = expression
        .columns([Supplierref::SupplierrefId, Supplierref::SupplierrefLabel])
        .expr_as(
            Expr::col((Supplier::Table, Supplier::SupplierId)),
            Alias::new("supplier.supplier_id"),
        )
        .expr_as(
            Expr::col((Supplier::Table, Supplier::SupplierLabel)),
            Alias::new("supplier.supplier_label"),
        )
        .order_by(Supplierref::SupplierrefLabel, Order::Asc)
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
        Ok(SupplierrefWrapper::from(row).0)
    })?;

    // Build result.
    let mut supplierrefs = Vec::new();
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
    }

    debug!("supplierrefs: {:#?}", supplierrefs);

    Ok((supplierrefs, count))
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
                "INSERT INTO supplier (supplier_id, supplier_label) VALUES (?1, ?2)",
                (300, String::from("FAKE_SUPPLIER_1")),
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO supplier (supplier_id, supplier_label) VALUES (?1, ?2)",
                (301, String::from("FAKE_SUPPLIER_2")),
            )
            .unwrap();

        // insert fake supplierrefs.
        let _ = db_connection
            .execute(
                "INSERT INTO supplierref (supplierref_label, supplier) VALUES (?1, ?2)",
                (String::from("1_ref1"), 300),
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO supplierref (supplierref_label, supplier) VALUES (?1, ?2)",
                (String::from("1_ref2"), 300),
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO supplierref (supplierref_label, supplier) VALUES (?1, ?2)",
                (String::from("1234"), 300),
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO supplierref (supplierref_label, supplier) VALUES (?1, ?2)",
                (String::from("12"), 300),
            )
            .unwrap();

        let _ = db_connection
            .execute(
                "INSERT INTO supplierref (supplierref_label, supplier) VALUES (?1, ?2)",
                (String::from("2_ref1"), 301),
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO supplierref (supplierref_label, supplier) VALUES (?1, ?2)",
                (String::from("2_ref2"), 301),
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO supplierref (supplierref_label, supplier) VALUES (?1, ?2)",
                (String::from("1234"), 301),
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO supplierref (supplierref_label, supplier) VALUES (?1, ?2)",
                (String::from("22"), 301),
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
