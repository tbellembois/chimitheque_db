use chimitheque_types::requestfilter::RequestFilter;
use log::debug;
use rusqlite::{Connection, Row};
use sea_query::{Expr, Iden, Order, Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use serde::Serialize;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
enum Precautionarystatement {
    Table,
    PrecautionarystatementId,
    PrecautionarystatementLabel,
    PrecautionarystatementReference,
}

#[derive(Debug, Serialize, Default)]
pub struct PrecautionarystatementStruct {
    pub match_exact_search: bool,
    pub precautionarystatement_id: u64,
    pub precautionarystatement_label: String,
    pub precautionarystatement_reference: String,
}

impl From<&Row<'_>> for PrecautionarystatementStruct {
    fn from(row: &Row) -> Self {
        Self {
            match_exact_search: false,
            precautionarystatement_id: row.get_unwrap("precautionarystatement_id"),
            precautionarystatement_label: row.get_unwrap("precautionarystatement_label"),
            precautionarystatement_reference: row.get_unwrap("precautionarystatement_reference"),
        }
    }
}

pub fn get_precautionarystatements(
    db_connection: &Connection,
    filter: RequestFilter,
) -> Result<(Vec<PrecautionarystatementStruct>, usize), Box<dyn std::error::Error>> {
    debug!("filter:{:?}", filter);

    let (sql, values) = Query::select()
        .columns([
            Precautionarystatement::PrecautionarystatementId,
            Precautionarystatement::PrecautionarystatementLabel,
            Precautionarystatement::PrecautionarystatementReference,
        ])
        .from(Precautionarystatement::Table)
        .conditions(
            filter.search.is_some(),
            |q| {
                q.and_where(
                    Expr::col(Precautionarystatement::PrecautionarystatementReference)
                        .like(format!("%{}%", filter.search.clone().unwrap())),
                );
            },
            |_| {},
        )
        .order_by(
            Precautionarystatement::PrecautionarystatementReference,
            Order::Asc,
        )
        .build_rusqlite(SqliteQueryBuilder);

    let mut stmt = db_connection.prepare(sql.as_str())?;
    let rows = stmt.query_map(&*values.as_params(), |row| {
        Ok(PrecautionarystatementStruct::from(row))
    })?;

    // Result statemtents and count.
    let mut precautionarystatements = Vec::new();
    let mut count = 0;
    for maybe_precautionarystatement in rows {
        let mut precautionarystatement = maybe_precautionarystatement?;

        // Set match_exact_search for statement matching filter.search.
        if filter.search.is_some()
            && precautionarystatement
                .precautionarystatement_reference
                .eq(&filter.search.clone().unwrap())
        {
            precautionarystatement.match_exact_search = true;

            // Inserting the statement at the beginning of the results.
            precautionarystatements.insert(0, precautionarystatement)
        } else {
            // Inserting the statement at the end of the results.
            precautionarystatements.push(precautionarystatement);
        }

        count += 1;
    }

    debug!("precautionarystatements: {:#?}", precautionarystatements);

    Ok((precautionarystatements, count))
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::init::init_db;
    use chimitheque_types::requestfilter::RequestFilter;
    use log::info;
    use rusqlite::Connection;

    fn init_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    fn init_test_db() -> Connection {
        let mut db_connection = Connection::open_in_memory().unwrap();
        init_db(&mut db_connection).unwrap();

        // insert fake precautionarystatements.
        let _ = db_connection
            .execute(
                "INSERT INTO precautionarystatement (precautionarystatement_label, precautionarystatement_reference) VALUES (?1, ?2)",
                [String::from("precautionarystatement1"), String::from("precautionarystatement1-ref")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO precautionarystatement (precautionarystatement_label, precautionarystatement_reference) VALUES (?1, ?2)",
                [String::from("aa precautionarystatement1"), String::from("aa precautionarystatement1-ref")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO precautionarystatement (precautionarystatement_label, precautionarystatement_reference) VALUES (?1, ?2)",
                [String::from("precautionarystatement2"), String::from("precautionarystatement2-ref")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO precautionarystatement (precautionarystatement_label, precautionarystatement_reference) VALUES (?1, ?2)",
                [String::from("precautionarystatement3"), String::from("precautionarystatement3-ref")],
            )
            .unwrap();

        db_connection
    }

    #[test]
    fn test_get_precautionarystatements() {
        init_logger();

        let db_connection = init_test_db();

        info!("testing ok result");
        let filter = RequestFilter {
            ..Default::default()
        };
        assert!(get_precautionarystatements(&db_connection, filter,).is_ok());

        info!("testing filter search");
        let filter = RequestFilter {
            search: Some(String::from("precautionarystatement1-ref")),
            ..Default::default()
        };
        let (precautionarystatements, count) =
            get_precautionarystatements(&db_connection, filter).unwrap();

        // expected number of results.
        assert_eq!(count, 2);
        // expected exact match appears first.
        assert!(precautionarystatements[0]
            .precautionarystatement_reference
            .eq("precautionarystatement1-ref"));
        assert!(precautionarystatements[0]
            .precautionarystatement_label
            .eq("precautionarystatement1"));
    }
}
