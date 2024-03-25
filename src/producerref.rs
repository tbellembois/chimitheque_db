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

    let (sql, values) = Query::select()
        .columns([Producerref::ProducerrefId, Producerref::ProducerrefLabel])
        .expr_as(
            Expr::col((Producer::Table, Producer::ProducerId)),
            Alias::new("producer.producer_id"),
        )
        .expr_as(
            Expr::col((Producer::Table, Producer::ProducerLabel)),
            Alias::new("producer.producer_label"),
        )
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
        )
        .order_by(Producerref::ProducerrefLabel, Order::Asc)
        .build_rusqlite(SqliteQueryBuilder);

    let mut stmt = db_connection.prepare(sql.as_str())?;
    let rows = stmt.query_map(&*values.as_params(), |row| Ok(ProducerrefStruct::from(row)))?;

    // Result supliers and count.
    let mut producerrefs = Vec::new();
    let mut count = 0;
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

        count += 1;
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
