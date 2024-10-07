use chimitheque_types::producer::Producer as ProducerStruct;
use chimitheque_types::{
    producerref::ProducerRef as ProducerRefStruct, requestfilter::RequestFilter,
};
use log::debug;
use rusqlite::{Connection, Row};
use sea_query::{Alias, Expr, Iden, Order, Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
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

        // insert fake producers.
        let _ = db_connection
            .execute(
                "INSERT INTO producer (producer_id, producer_label) VALUES (?1, ?2)",
                (300, String::from("FAKE_PRODUCER_1")),
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO producer (producer_id, producer_label) VALUES (?1, ?2)",
                (301, String::from("FAKE_PRODUCER_2")),
            )
            .unwrap();

        // insert fake producer_refs.
        let _ = db_connection
            .execute(
                "INSERT INTO producer_ref (producer_ref_label, producer) VALUES (?1, ?2)",
                (String::from("1_ref1"), 300),
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO producer_ref (producer_ref_label, producer) VALUES (?1, ?2)",
                (String::from("1_ref2"), 301),
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO producer_ref (producer_ref_label, producer) VALUES (?1, ?2)",
                (String::from("1234"), 300),
            )
            .unwrap();
        let _ = db_connection.execute(
            "INSERT INTO producer_ref (producer_ref_label, producer) VALUES (?1, ?2)",
            (String::from("12"), 300),
        );

        let _ = db_connection
            .execute(
                "INSERT INTO producer_ref (producer_ref_label, producer) VALUES (?1, ?2)",
                (String::from("2_ref1"), 301),
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO producer_ref (producer_ref_label, producer) VALUES (?1, ?2)",
                (String::from("2_ref2"), 301),
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO producer_ref (producer_ref_label, producer) VALUES (?1, ?2)",
                (String::from("1234"), 301),
            )
            .unwrap();
        let _ = db_connection.execute(
            "INSERT INTO producer_ref (producer_ref_label, producer) VALUES (?1, ?2)",
            (String::from("22"), 301),
        );

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
