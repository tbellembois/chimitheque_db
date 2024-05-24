use chimitheque_types::{producer::Producer as ProducerStruct, requestfilter::RequestFilter};
use log::debug;
use rusqlite::{Connection, Row};
use sea_query::{Expr, Iden, Order, Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use serde::Serialize;

#[derive(Iden)]
pub enum Producer {
    Table,
    ProducerId,
    ProducerLabel,
}

#[derive(Debug, Serialize)]
pub struct ProducerWrapper(pub ProducerStruct);

impl From<&Row<'_>> for ProducerWrapper {
    fn from(row: &Row) -> Self {
        Self(ProducerStruct {
            producer_id: row.get_unwrap("producer_id"),
            producer_label: row.get_unwrap("producer_label"),
            match_exact_search: false,
        })
    }
}

pub fn get_producers(
    db_connection: &Connection,
    filter: RequestFilter,
) -> Result<(Vec<ProducerStruct>, usize), Box<dyn std::error::Error>> {
    debug!("filter:{:?}", filter);

    // Create common query statement.
    let mut expression = Query::select();
    expression.from(Producer::Table).conditions(
        filter.search.is_some(),
        |q| {
            q.and_where(
                Expr::col(Producer::ProducerLabel)
                    .like(format!("%{}%", filter.search.clone().unwrap())),
            );
        },
        |_| {},
    );

    // Create count query.
    let (count_sql, count_values) = expression
        .clone()
        .expr(Expr::col((Producer::Table, Producer::ProducerId)).count_distinct())
        .build_rusqlite(SqliteQueryBuilder);

    debug!("count_sql: {}", count_sql.clone().as_str());
    debug!("count_values: {:?}", count_values);

    // Create select query.
    let (select_sql, select_values) = expression
        .columns([Producer::ProducerId, Producer::ProducerLabel])
        .order_by(Producer::ProducerLabel, Order::Asc)
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
        Ok(ProducerWrapper::from(row).0)
    })?;

    // Build result.
    let mut producers = Vec::new();
    for maybe_producer in rows {
        let mut producer = maybe_producer?;

        // Set match_exact_search for producer matching filter.search.
        if filter.search.is_some() && producer.producer_label.eq(&filter.search.clone().unwrap()) {
            producer.match_exact_search = true;

            // Inserting the producer at the beginning of the results.
            producers.insert(0, producer)
        } else {
            // Inserting the producer at the end of the results.
            producers.push(producer);
        }
    }

    debug!("producers: {:#?}", producers);

    Ok((producers, count))
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
                "INSERT INTO producer (producer_label) VALUES (?1)",
                [String::from("FAKE_PRODUCER")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO producer (producer_label) VALUES (?1)",
                [String::from("FAKE_PRODUCER ONE")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO producer (producer_label) VALUES (?1)",
                [String::from("FAKE_PRODUCER TWO")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO producer (producer_label) VALUES (?1)",
                [String::from("FAKE_PRODUCER THREE")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO producer (producer_label) VALUES (?1)",
                [String::from("AAA FAKE_PRODUCER")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO producer (producer_label) VALUES (?1)",
                [String::from("YET ANOTHER PRODUCER")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO producer (producer_label) VALUES (?1)",
                [String::from("12345")],
            )
            .unwrap();

        db_connection
    }

    #[test]
    fn test_get_producers() {
        init_logger();

        let db_connection = init_test_db();

        info!("testing ok result");
        assert!(get_producers(
            &db_connection,
            RequestFilter {
                ..Default::default()
            },
        )
        .is_ok());

        info!("testing filter search");
        let filter = RequestFilter {
            search: Some(String::from("FAKE_PRODUCER")),
            ..Default::default()
        };
        let (producers, count) = get_producers(&db_connection, filter).unwrap();

        // expected number of results.
        assert_eq!(count, 5);
        // expected exact match appears first.
        assert!(producers[0].producer_label.eq("FAKE_PRODUCER"))
    }
}
