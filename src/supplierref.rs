use chimitheque_types::{
    requestfilter::RequestFilter, supplier::Supplier as SupplierStruct,
    supplierref::SupplierRef as SupplierRefStruct,
};
use chimitheque_utils::string::{Transform, clean};
use log::debug;
use rusqlite::{Connection, Row};
use sea_query::{
    Alias, Expr, ExprTrait, Iden, OnConflict, Order, Query, SimpleExpr, SqliteQueryBuilder,
};
use sea_query_rusqlite::RusqliteBinder;
use serde::Serialize;

use crate::supplier::Supplier;

/// Table and column names of the `supplier_ref` table, for `sea_query`.
#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum SupplierRef {
    /// The table.
    Table,
    /// The `supplier_ref_id` column.
    SupplierRefId,
    /// The `supplier_ref_label` column.
    SupplierRefLabel,
    /// The `supplier` column.
    Supplier,
}

/// A supplier ref read from a database row.
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

/// Returns the supplier references matching the filter, and their total count
/// without limit and offset.
pub fn get_supplier_refs(
    db_connection: &Connection,
    filter: &RequestFilter,
) -> Result<(Vec<SupplierRefStruct>, usize), Box<dyn std::error::Error + Send + Sync>> {
    debug!("filter:{filter:?}");

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

    debug!("count_sql: {}", count_sql.as_str());
    debug!("count_values: {count_values:?}");

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
        .order_by_expr(
            Expr::cust_with_expr("? COLLATE NOCASE", Expr::col(SupplierRef::SupplierRefLabel)),
            Order::Asc,
        )
        .conditions(
            filter.limit.is_some(),
            |q| {
                q.limit(filter.limit.unwrap() as u64);
            },
            |_| {},
        )
        .conditions(
            filter.offset.is_some(),
            |q| {
                q.offset(filter.offset.unwrap() as u64);
            },
            |_| {},
        )
        .build_rusqlite(SqliteQueryBuilder);

    debug!("select_sql: {}", select_sql.as_str());
    debug!("select_values: {select_values:?}");

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
    let mut supplier_refs = Vec::with_capacity(filter.limit.unwrap_or(100));
    for maybe_supplier_ref in rows {
        let mut supplier_ref = maybe_supplier_ref?;

        // Set match_exact_search for supplier_ref matching filter.search.
        if filter.search.is_some()
            && supplier_ref.supplier_ref_label == filter.search.clone().unwrap()
        {
            supplier_ref.match_exact_search = true;

            // Inserting the supplier at the beginning of the results.
            supplier_refs.insert(0, supplier_ref);
        } else {
            // Inserting the supplier at the end of the results.
            supplier_refs.push(supplier_ref);
        }
    }

    debug!("supplier_refs: {supplier_refs:#?}");

    Ok((supplier_refs, count))
}

/// Creates the supplier reference, or updates it if it has an id. Returns its
/// id.
pub fn create_update_supplier_ref(
    db_connection: &Connection,
    supplier_ref: &SupplierRefStruct,
) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    debug!("create_update_supplier_ref: {supplier_ref:#?}");

    let clean_supplier_ref_label = clean(&supplier_ref.supplier_ref_label, Transform::None);

    // Update request: list of (columns, values) pairs to insert.
    let columns_values = vec![
        (
            SupplierRef::SupplierRefLabel,
            clean_supplier_ref_label.clone().into(),
        ),
        (
            SupplierRef::Supplier,
            supplier_ref.supplier.supplier_id.into(),
        ),
    ];

    // Create request: list of columns and values to insert.
    let columns = vec![SupplierRef::SupplierRefLabel, SupplierRef::Supplier];
    let values = vec![
        SimpleExpr::Value(clean_supplier_ref_label.into()),
        SimpleExpr::Value(supplier_ref.supplier.supplier_id.into()),
    ];

    if let Some(supplier_ref_id) = supplier_ref.supplier_ref_id {
        // Update query.
        let (sql_query, sql_values) = Query::update()
            .table(SupplierRef::Table)
            .values(columns_values)
            .and_where(Expr::col(SupplierRef::SupplierRefId).eq(supplier_ref_id))
            .build_rusqlite(SqliteQueryBuilder);

        debug!("sql_query: {}", sql_query.as_str());
        debug!("sql_values: {sql_values:?}");

        _ = db_connection.execute(&sql_query, &*sql_values.as_params())?;

        Ok(supplier_ref_id)
    } else {
        // Get or create: a reference with the same label for the same supplier
        // (UNIQUE) is reused. The no-op update makes RETURNING give the existing
        // id, which DO NOTHING would not.
        let (sql_query, sql_values) = Query::insert()
            .into_table(SupplierRef::Table)
            .columns(columns)
            .values(values)?
            .on_conflict(
                OnConflict::columns([SupplierRef::Supplier, SupplierRef::SupplierRefLabel])
                    .update_column(SupplierRef::SupplierRefLabel)
                    .to_owned(),
            )
            .returning_col(SupplierRef::SupplierRefId)
            .build_rusqlite(SqliteQueryBuilder);

        debug!("sql_query: {}", sql_query.as_str());
        debug!("sql_values: {sql_values:?}");

        let supplier_ref_id: u64 =
            db_connection.query_row(&sql_query, &*sql_values.as_params(), |row| row.get(0))?;

        debug!("supplier_ref_id: {supplier_ref_id}");

        Ok(supplier_ref_id)
    }
}

#[cfg(test)]
#[path = "supplierref_tests.rs"]
mod supplierref_tests;
