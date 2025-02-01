use chimitheque_types::{
    requestfilter::RequestFilter, supplier::Supplier as SupplierStruct,
    supplierref::SupplierRef as SupplierRefStruct,
};
use log::debug;
use rusqlite::{Connection, Row};
use sea_query::{Alias, Expr, Iden, Order, Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use serde::Serialize;

use crate::supplier::Supplier;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum SupplierRef {
    Table,
    SupplierRefId,
    SupplierRefLabel,
    Supplier,
}

#[derive(Debug, Serialize)]
pub struct SupplierRefWrapper(pub SupplierRefStruct);

impl From<&Row<'_>> for SupplierRefWrapper {
    fn from(row: &Row) -> Self {
        Self({
            SupplierRefStruct {
                match_exact_search: false,
                supplier_ref_id: row.get_unwrap("supplier_ref_id"),
                supplier_ref_label: row.get_unwrap("supplier_ref_label"),
                supplier: SupplierStruct {
                    match_exact_search: false,
                    supplier_id: row.get_unwrap("supplier.supplier_id"),
                    supplier_label: row.get_unwrap("supplier.supplier_label"),
                },
            }
        })
    }
}

pub fn get_supplier_refs(
    db_connection: &Connection,
    filter: RequestFilter,
) -> Result<(Vec<SupplierRefStruct>, usize), Box<dyn std::error::Error>> {
    debug!("filter:{:?}", filter);

    // Create common query statement.
    let mut expression = Query::select();
    expression
        .from(SupplierRef::Table)
        .left_join(
            Supplier::Table,
            Expr::col((SupplierRef::Table, SupplierRef::Supplier))
                .equals((Supplier::Table, Supplier::SupplierId)),
        )
        .conditions(
            filter.search.is_some(),
            |q| {
                q.and_where(
                    Expr::col(SupplierRef::SupplierRefLabel)
                        .like(format!("%{}%", filter.search.clone().unwrap())),
                );
            },
            |_| {},
        )
        .conditions(
            filter.supplier.is_some(),
            |q| {
                q.and_where(Expr::col(SupplierRef::Supplier).eq(filter.supplier.unwrap()));
            },
            |_| {},
        );

    // Create count query.
    let (count_sql, count_values) = expression
        .clone()
        .expr(Expr::col((SupplierRef::Table, SupplierRef::SupplierRefId)).count_distinct())
        .build_rusqlite(SqliteQueryBuilder);

    debug!("count_sql: {}", count_sql.clone().as_str());
    debug!("count_values: {:?}", count_values);

    // Create select query.

    let (select_sql, select_values) = expression
        .columns([SupplierRef::SupplierRefId, SupplierRef::SupplierRefLabel])
        .expr_as(
            Expr::col((Supplier::Table, Supplier::SupplierId)),
            Alias::new("supplier.supplier_id"),
        )
        .expr_as(
            Expr::col((Supplier::Table, Supplier::SupplierLabel)),
            Alias::new("supplier.supplier_label"),
        )
        .order_by(SupplierRef::SupplierRefLabel, Order::Asc)
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
        Ok(SupplierRefWrapper::from(row).0)
    })?;

    // Build result.
    let mut supplier_refs = Vec::new();
    for maybe_supplier_ref in rows {
        let mut supplier_ref = maybe_supplier_ref?;

        // Set match_exact_search for supplier_ref matching filter.search.
        if filter.search.is_some()
            && supplier_ref
                .supplier_ref_label
                .eq(&filter.search.clone().unwrap())
        {
            supplier_ref.match_exact_search = true;

            // Inserting the supplier at the beginning of the results.
            supplier_refs.insert(0, supplier_ref)
        } else {
            // Inserting the supplier at the end of the results.
            supplier_refs.push(supplier_ref);
        }
    }

    debug!("supplier_refs: {:#?}", supplier_refs);

    Ok((supplier_refs, count))
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

        // insert fake supplier refs.
        let _ = db_connection
            .execute(
                "INSERT INTO supplier_ref (supplier_ref_label, supplier) VALUES (?1, ?2)",
                (String::from("1_ref1"), 300),
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO supplier_ref (supplier_ref_label, supplier) VALUES (?1, ?2)",
                (String::from("1_ref2"), 300),
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO supplier_ref (supplier_ref_label, supplier) VALUES (?1, ?2)",
                (String::from("1234"), 300),
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO supplier_ref (supplier_ref_label, supplier) VALUES (?1, ?2)",
                (String::from("12"), 300),
            )
            .unwrap();

        let _ = db_connection
            .execute(
                "INSERT INTO supplier_ref (supplier_ref_label, supplier) VALUES (?1, ?2)",
                (String::from("2_ref1"), 301),
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO supplier_ref (supplier_ref_label, supplier) VALUES (?1, ?2)",
                (String::from("2_ref2"), 301),
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO supplier_ref (supplier_ref_label, supplier) VALUES (?1, ?2)",
                (String::from("1234"), 301),
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO supplier_ref (supplier_ref_label, supplier) VALUES (?1, ?2)",
                (String::from("22"), 301),
            )
            .unwrap();

        db_connection
    }

    #[test]
    fn test_get_supplier_refs() {
        init_logger();

        let db_connection = init_test_db();

        info!("testing total result");
        let filter = RequestFilter {
            ..Default::default()
        };
        let (_, count) = get_supplier_refs(&db_connection, filter).unwrap();

        // expected number of results.
        assert_eq!(count, 8);

        info!("testing filter search");
        let filter = RequestFilter {
            search: Some(String::from("1_ref1")),
            ..Default::default()
        };
        let (supplier_refs, count) = get_supplier_refs(&db_connection, filter).unwrap();

        // expected number of results.
        assert_eq!(count, 1);
        // expected correct supplier.
        assert!(supplier_refs[0]
            .supplier
            .supplier_label
            .eq("FAKE_SUPPLIER_1"))
    }
}
