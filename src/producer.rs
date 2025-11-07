use chimitheque_types::{producer::Producer as ProducerStruct, requestfilter::RequestFilter};
use log::debug;
use rusqlite::{Connection, Row};
use sea_query::{Expr, Iden, Order, Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use serde::Serialize;

#[allow(clippy::enum_variant_names)]
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

// pub fn create_update_producer(
//     db_connection: &mut Connection,
//     producer: ProducerStruct,
// ) -> Result<u64, Box<dyn std::error::Error>> {
//     debug!("create_update_producer: {:#?}", producer);

//     let db_transaction = db_connection.transaction()?;

//     let clean_producer_label = clean(&producer.producer_label, Transform::None);

//     // Update request: list of (columns, values) pairs to insert.
//     let columns_values = vec![(Producer::ProducerLabel, clean_producer_label.clone().into())];

//     // Create request: list of columns and values to insert.
//     let columns = vec![Producer::ProducerLabel];
//     let values = vec![SimpleExpr::Value(clean_producer_label.into())];

//     let sql_query: String;
//     let mut sql_values: RusqliteValues = RusqliteValues(vec![]);

//     if let Some(producer_id) = producer.producer_id {
//         // Update query.
//         (sql_query, sql_values) = Query::update()
//             .table(Producer::Table)
//             .values(columns_values)
//             .and_where(Expr::col(Producer::ProducerId).eq(producer_id))
//             .build_rusqlite(SqliteQueryBuilder);
//     } else {
//         // Insert query.
//         sql_query = Query::insert()
//             .into_table(Producer::Table)
//             .columns(columns)
//             .values(values)?
//             .to_string(SqliteQueryBuilder);
//     }

//     debug!("sql_query: {}", sql_query.clone().as_str());
//     debug!("sql_values: {:?}", sql_values);

//     _ = db_transaction.execute(&sql_query, &*sql_values.as_params())?;

//     let last_insert_update_id: u64;

//     if let Some(producer_id) = producer.producer_id {
//         last_insert_update_id = producer_id;
//     } else {
//         last_insert_update_id = db_transaction.last_insert_rowid().try_into()?;
//     }

//     debug!("last_insert_update_id: {}", last_insert_update_id);

//     db_transaction.commit()?;

//     Ok(last_insert_update_id)
// }

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
        assert_eq!(count, 7);
        // expected exact match appears first.
        assert!(producers[0].producer_label.eq("FAKE_PRODUCER"))
    }
}
