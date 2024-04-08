use chimitheque_types::requestfilter::RequestFilter;
use log::debug;
use rusqlite::{Connection, Row};
use sea_query::{Alias, Expr, Iden, Order, Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use serde::Serialize;

use crate::producer::{Producer, ProducerStruct};

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
enum Producerref {
    Table,
    ProducerrefId,
    ProducerrefLabel,
    Producer,
}

#[derive(Debug, Serialize)]
pub struct ProducerrefStruct {
    match_exact_search: bool,
    producerref_id: u64,
    producerref_label: String,

    producer: ProducerStruct,
}

impl From<&Row<'_>> for ProducerrefStruct {
    fn from(row: &Row) -> Self {
        Self {
            match_exact_search: false,
            producerref_id: row.get_unwrap("producerref_id"),
            producerref_label: row.get_unwrap("producerref_label"),
            producer: ProducerStruct {
                match_exact_search: false,
                producer_id: row.get_unwrap("producer.producer_id"),
                producer_label: row.get_unwrap("producer.producer_label"),
            },
        }
    }
}

pub fn get_producerrefs(
    db_connection: &Connection,
    filter: RequestFilter,
) -> Result<(Vec<ProducerrefStruct>, usize), Box<dyn std::error::Error>> {
    debug!("filter:{:?}", filter);

    // Create common query statement.
    let mut expression = Query::select();
    expression
        .from(Producerref::Table)
        .left_join(
            Producer::Table,
            Expr::col((Producerref::Table, Producerref::Producer))
                .equals((Producer::Table, Producer::ProducerId)),
        )
        .conditions(
            filter.search.is_some(),
            |q| {
                q.and_where(
                    Expr::col(Producerref::ProducerrefLabel)
                        .like(format!("%{}%", filter.search.clone().unwrap())),
                );
            },
            |_| {},
        )
        .conditions(
            filter.producer.is_some(),
            |q| {
                q.and_where(Expr::col(Producerref::Producer).eq(filter.producer.unwrap()));
            },
            |_| {},
        );

    // Create count query.
    let (count_sql, count_values) = expression
        .clone()
        .expr(Expr::col((Producerref::Table, Producerref::ProducerrefId)).count_distinct())
        .build_rusqlite(SqliteQueryBuilder);

    debug!("count_sql: {}", count_sql.clone().as_str());
    debug!("count_values: {:?}", count_values);

    // Create select query.
    let (select_sql, select_values) = expression
        .columns([Producerref::ProducerrefId, Producerref::ProducerrefLabel])
        .expr_as(
            Expr::col((Producer::Table, Producer::ProducerId)),
            Alias::new("producer.producer_id"),
        )
        .expr_as(
            Expr::col((Producer::Table, Producer::ProducerLabel)),
            Alias::new("producer.producer_label"),
        )
        .order_by(Producerref::ProducerrefLabel, Order::Asc)
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
        Ok(ProducerrefStruct::from(row))
    })?;

    // Build result.
    let mut producerrefs = Vec::new();
    for maybe_producerref in rows {
        let mut producerref = maybe_producerref?;

        // Set match_exact_search for producerref matching filter.search.
        if filter.search.is_some()
            && producerref
                .producerref_label
                .eq(&filter.search.clone().unwrap())
        {
            producerref.match_exact_search = true;

            // Inserting the producer at the beginning of the results.
            producerrefs.insert(0, producerref)
        } else {
            // Inserting the producer at the end of the results.
            producerrefs.push(producerref);
        }
    }

    debug!("producerrefs: {:#?}", producerrefs);

    Ok((producerrefs, count))
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

        // insert fake producerrefs.
        let _ = db_connection
            .execute(
                "INSERT INTO producerref (producerref_label, producer) VALUES (?1, ?2)",
                (String::from("1_ref1"), 300),
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO producerref (producerref_label, producer) VALUES (?1, ?2)",
                (String::from("1_ref2"), 301),
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO producerref (producerref_label, producer) VALUES (?1, ?2)",
                (String::from("1234"), 300),
            )
            .unwrap();
        let _ = db_connection.execute(
            "INSERT INTO producerref (producerref_label, producer) VALUES (?1, ?2)",
            (String::from("12"), 300),
        );

        let _ = db_connection
            .execute(
                "INSERT INTO producerref (producerref_label, producer) VALUES (?1, ?2)",
                (String::from("2_ref1"), 301),
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO producerref (producerref_label, producer) VALUES (?1, ?2)",
                (String::from("2_ref2"), 301),
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO producerref (producerref_label, producer) VALUES (?1, ?2)",
                (String::from("1234"), 301),
            )
            .unwrap();
        let _ = db_connection.execute(
            "INSERT INTO producerref (producerref_label, producer) VALUES (?1, ?2)",
            (String::from("22"), 301),
        );

        db_connection
    }

    #[test]
    fn test_get_producerrefs() {
        init_logger();

        let db_connection = init_test_db();

        info!("testing total result");
        let filter = RequestFilter {
            ..Default::default()
        };
        let (_, count) = get_producerrefs(&db_connection, filter).unwrap();

        // expected number of results.
        assert_eq!(count, 8);

        info!("testing filter search");
        let filter = RequestFilter {
            search: Some(String::from("1_ref1")),
            ..Default::default()
        };
        let (producerrefs, count) = get_producerrefs(&db_connection, filter).unwrap();

        // expected number of results.
        assert_eq!(count, 1);
        // expected correct producer.
        assert!(producerrefs[0]
            .producer
            .producer_label
            .eq("FAKE_PRODUCER_1"))
    }
}
