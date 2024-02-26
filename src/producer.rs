use chimitheque_types::requestfilter::RequestFilter;
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
pub struct ProducerStruct {
    pub(crate) match_exact_search: bool,
    pub(crate) producer_id: u64,
    pub(crate) producer_label: String,
}

impl From<&Row<'_>> for ProducerStruct {
    fn from(row: &Row) -> Self {
        Self {
            producer_id: row.get_unwrap("producer_id"),
            producer_label: row.get_unwrap("producer_label"),
            match_exact_search: false,
        }
    }
}

pub fn get_producers(
    db_connection: &Connection,
    filter: RequestFilter,
) -> Result<(Vec<ProducerStruct>, usize), Box<dyn std::error::Error>> {
    debug!("filter:{:?}", filter);

    let (sql, values) = Query::select()
        .columns([Producer::ProducerId, Producer::ProducerLabel])
        .from(Producer::Table)
        .conditions(
            filter.search.is_some(),
            |q| {
                q.and_where(
                    Expr::col(Producer::ProducerLabel)
                        .like(format!("%{}%", filter.search.clone().unwrap())),
                );
            },
            |_| {},
        )
        .order_by(Producer::ProducerLabel, Order::Asc)
        .build_rusqlite(SqliteQueryBuilder);

    let mut stmt = db_connection.prepare(sql.as_str())?;
    let rows = stmt.query_map(&*values.as_params(), |row| Ok(ProducerStruct::from(row)))?;

    // Result supliers and count.
    let mut producers = Vec::new();
    let mut count = 0;
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

        count += 1;
    }

    debug!("producers: {:#?}", producers);

    Ok((producers, count))
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
