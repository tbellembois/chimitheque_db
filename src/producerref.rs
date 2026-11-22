use chimitheque_types::producer::Producer as ProducerStruct;
use chimitheque_types::{
    producerref::ProducerRef as ProducerRefStruct, requestfilter::RequestFilter,
};
use chimitheque_utils::string::{Transform, clean};
use log::debug;
use rusqlite::{Connection, Row};
use sea_query::{
    Alias, Expr, ExprTrait, Iden, OnConflict, Order, Query, SimpleExpr, SqliteQueryBuilder,
};
use sea_query_rusqlite::RusqliteBinder;
use serde::Serialize;

use crate::producer::Producer;

/// Table and column names of the `producer_ref` table, for `sea_query`.
#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum ProducerRef {
    /// The table.
    Table,
    /// The `producer_ref_id` column.
    ProducerRefId,
    /// The `producer_ref_label` column.
    ProducerRefLabel,
    /// The `producer` column.
    Producer,
}

/// A producer ref read from a database row.
#[derive(Debug, Serialize)]
pub struct ProducerRefWrapper(ProducerRefStruct);

impl From<&Row<'_>> for ProducerRefWrapper {
    fn from(row: &Row) -> Self {
        Self({
            ProducerRefStruct {
                match_exact_search: false,
                producer_ref_id: row.get_unwrap("producer_ref_id"),
                producer_ref_label: row.get_unwrap("producer_ref_label"),
                producer: ProducerStruct {
                    match_exact_search: false,
                    producer_id: row.get_unwrap("producer.producer_id"),
                    producer_label: row.get_unwrap("producer.producer_label"),
                },
            }
        })
    }
}

/// Returns the producer references matching the filter, and their total count
/// without limit and offset.
pub fn get_producer_refs(
    db_connection: &Connection,
    filter: &RequestFilter,
) -> Result<(Vec<ProducerRefStruct>, usize), Box<dyn std::error::Error + Send + Sync>> {
    debug!("filter:{filter:?}");

    // Create common query statement.
    let mut expression = Query::select();
    expression
        .from(ProducerRef::Table)
        .left_join(
            Producer::Table,
            Expr::col((ProducerRef::Table, ProducerRef::Producer))
                .equals((Producer::Table, Producer::ProducerId)),
        )
        .conditions(
            filter.search.is_some(),
            |q| {
                q.and_where(
                    Expr::col(ProducerRef::ProducerRefLabel)
                        .like(format!("%{}%", filter.search.clone().unwrap())),
                );
            },
            |_| {},
        )
        .conditions(
            filter.producer.is_some(),
            |q| {
                q.and_where(Expr::col(ProducerRef::Producer).eq(filter.producer.unwrap()));
            },
            |_| {},
        );

    // Create count query.
    let (count_sql, count_values) = expression
        .clone()
        .expr(Expr::col((ProducerRef::Table, ProducerRef::ProducerRefId)).count_distinct())
        .build_rusqlite(SqliteQueryBuilder);

    debug!("count_sql: {}", count_sql.as_str());
    debug!("count_values: {count_values:?}");

    // Create select query.
    let (select_sql, select_values) = expression
        .columns([ProducerRef::ProducerRefId, ProducerRef::ProducerRefLabel])
        .expr_as(
            Expr::col((Producer::Table, Producer::ProducerId)),
            Alias::new("producer.producer_id"),
        )
        .expr_as(
            Expr::col((Producer::Table, Producer::ProducerLabel)),
            Alias::new("producer.producer_label"),
        )
        .order_by_expr(
            Expr::cust_with_expr("? COLLATE NOCASE", Expr::col(ProducerRef::ProducerRefLabel)),
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
        Ok(ProducerRefWrapper::from(row).0)
    })?;

    // Build result.
    let mut producer_refs = Vec::with_capacity(filter.limit.unwrap_or(100));
    for maybe_producer_ref in rows {
        let mut producer_ref = maybe_producer_ref?;

        // Set match_exact_search for producer_ref matching filter.search.
        if filter.search.is_some()
            && producer_ref.producer_ref_label == (filter.search.clone().unwrap())
        {
            producer_ref.match_exact_search = true;

            // Inserting the producer at the beginning of the results.
            producer_refs.insert(0, producer_ref);
        } else {
            // Inserting the producer at the end of the results.
            producer_refs.push(producer_ref);
        }
    }

    debug!("producer_refs: {producer_refs:#?}");

    Ok((producer_refs, count))
}

/// Creates the producer reference, or updates it if it has an id. Returns its
/// id.
pub fn create_update_producer_ref(
    db_connection: &Connection,
    producer_ref: &ProducerRefStruct,
) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    debug!("create_update_producer_ref: {producer_ref:#?}");

    let clean_producer_ref_label = clean(&producer_ref.producer_ref_label, Transform::None);

    // Update request: list of (columns, values) pairs to insert.
    let columns_values = vec![
        (
            ProducerRef::ProducerRefLabel,
            clean_producer_ref_label.clone().into(),
        ),
        (
            ProducerRef::Producer,
            producer_ref.producer.producer_id.into(),
        ),
    ];

    // Create request: list of columns and values to insert.
    let columns = vec![ProducerRef::ProducerRefLabel, ProducerRef::Producer];
    let values = vec![
        SimpleExpr::Value(clean_producer_ref_label.into()),
        SimpleExpr::Value(producer_ref.producer.producer_id.into()),
    ];

    if let Some(producer_ref_id) = producer_ref.producer_ref_id {
        // Update query.
        let (sql_query, sql_values) = Query::update()
            .table(ProducerRef::Table)
            .values(columns_values)
            .and_where(Expr::col(ProducerRef::ProducerRefId).eq(producer_ref_id))
            .build_rusqlite(SqliteQueryBuilder);

        debug!("sql_query: {}", sql_query.as_str());
        debug!("sql_values: {sql_values:?}");

        _ = db_connection.execute(&sql_query, &*sql_values.as_params())?;

        Ok(producer_ref_id)
    } else {
        // Get or create: a reference with the same label for the same producer
        // (UNIQUE) is reused. The no-op update makes RETURNING give the existing
        // id, which DO NOTHING would not.
        let (sql_query, sql_values) = Query::insert()
            .into_table(ProducerRef::Table)
            .columns(columns)
            .values(values)?
            .on_conflict(
                OnConflict::columns([ProducerRef::Producer, ProducerRef::ProducerRefLabel])
                    .update_column(ProducerRef::ProducerRefLabel)
                    .to_owned(),
            )
            .returning_col(ProducerRef::ProducerRefId)
            .build_rusqlite(SqliteQueryBuilder);

        debug!("sql_query: {}", sql_query.as_str());
        debug!("sql_values: {sql_values:?}");

        let producer_ref_id: u64 =
            db_connection.query_row(&sql_query, &*sql_values.as_params(), |row| row.get(0))?;

        debug!("producer_ref_id: {producer_ref_id}");

        Ok(producer_ref_id)
    }
}

#[cfg(test)]
#[path = "producerref_tests.rs"]
mod producerref_tests;
