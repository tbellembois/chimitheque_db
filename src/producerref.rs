use chimitheque_types::producer::Producer as ProducerStruct;
use chimitheque_types::{
    producerref::ProducerRef as ProducerRefStruct, requestfilter::RequestFilter,
};
use chimitheque_utils::string::{clean, Transform};
use log::debug;
use rusqlite::{Connection, Row};
use sea_query::{Alias, Expr, Iden, Order, Query, SimpleExpr, SqliteQueryBuilder};
use sea_query_rusqlite::{RusqliteBinder, RusqliteValues};
use serde::Serialize;

use crate::producer::Producer;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum ProducerRef {
    Table,
    ProducerRefId,
    ProducerRefLabel,
    Producer,
}

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

pub fn get_producer_refs(
    db_connection: &Connection,
    filter: RequestFilter,
) -> Result<(Vec<ProducerRefStruct>, usize), Box<dyn std::error::Error>> {
    debug!("filter:{:?}", filter);

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

    debug!("count_sql: {}", count_sql.clone().as_str());
    debug!("count_values: {:?}", count_values);

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
        .order_by(ProducerRef::ProducerRefLabel, Order::Asc)
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
        Ok(ProducerRefWrapper::from(row).0)
    })?;

    // Build result.
    let mut producer_refs = Vec::new();
    for maybe_producer_ref in rows {
        let mut producer_ref = maybe_producer_ref?;

        // Set match_exact_search for producer_ref matching filter.search.
        if filter.search.is_some()
            && producer_ref
                .producer_ref_label
                .eq(&filter.search.clone().unwrap())
        {
            producer_ref.match_exact_search = true;

            // Inserting the producer at the beginning of the results.
            producer_refs.insert(0, producer_ref)
        } else {
            // Inserting the producer at the end of the results.
            producer_refs.push(producer_ref);
        }
    }

    debug!("producer_refs: {:#?}", producer_refs);

    Ok((producer_refs, count))
}

pub fn create_update_producer_ref(
    db_connection: &Connection,
    producer_ref: &ProducerRefStruct,
) -> Result<u64, Box<dyn std::error::Error>> {
    debug!("create_update_producer_ref: {:#?}", producer_ref);

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

    let sql_query: String;
    let mut sql_values: RusqliteValues = RusqliteValues(vec![]);

    if let Some(producer_ref_id) = producer_ref.producer_ref_id {
        // Update query.
        (sql_query, sql_values) = Query::update()
            .table(ProducerRef::Table)
            .values(columns_values)
            .and_where(Expr::col(ProducerRef::ProducerRefId).eq(producer_ref_id))
            .build_rusqlite(SqliteQueryBuilder);
    } else {
        // Insert query.
        sql_query = Query::insert()
            .into_table(ProducerRef::Table)
            .columns(columns)
            .values(values)?
            .to_string(SqliteQueryBuilder);
    }

    debug!("sql_query: {}", sql_query.clone().as_str());
    debug!("sql_values: {:?}", sql_values);

    _ = db_connection.execute(&sql_query, &*sql_values.as_params())?;

    let last_insert_update_id: u64;

    if let Some(producer_ref_id) = producer_ref.producer_ref_id {
        last_insert_update_id = producer_ref_id;
    } else {
        last_insert_update_id = db_connection.last_insert_rowid().try_into()?;
    }

    debug!("last_insert_update_id: {}", last_insert_update_id);

    Ok(last_insert_update_id)
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::init::{connect_test, init_db, insert_fake_values};
    use log::info;

    fn init_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    fn init_test_db() -> Connection {
        let mut db_connection = connect_test();
        init_db(&mut db_connection).unwrap();
        insert_fake_values(&mut db_connection).unwrap();
        db_connection
    }

    #[test]
    fn test_get_producer_refs() {
        init_logger();

        let db_connection = init_test_db();

        info!("testing total result");
        let filter = RequestFilter {
            ..Default::default()
        };
        let (_, count) = get_producer_refs(&db_connection, filter).unwrap();

        // expected number of results.
        assert_eq!(count, 8);

        info!("testing filter search");
        let filter = RequestFilter {
            search: Some(String::from("1_ref1")),
            ..Default::default()
        };
        let (producer_refs, count) = get_producer_refs(&db_connection, filter).unwrap();

        // expected number of results.
        assert_eq!(count, 1);
        // expected correct producer.
        assert!(producer_refs[0]
            .producer
            .producer_label
            .eq("FAKE_PRODUCER_1"))
    }
}
