use chimitheque_types::requestfilter::RequestFilter;
use log::debug;
use rusqlite::{Connection, Row};
use sea_query::{Expr, Iden, Order, Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use serde::Serialize;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
enum Hazardstatement {
    Table,
    HazardstatementId,
    HazardstatementLabel,
    HazardstatementReference,
}

#[derive(Debug, Serialize, Default)]
pub struct HazardstatementStruct {
    pub match_exact_search: bool,
    pub hazardstatement_id: u64,
    pub hazardstatement_label: String,
    pub hazardstatement_reference: String,
}

impl From<&Row<'_>> for HazardstatementStruct {
    fn from(row: &Row) -> Self {
        Self {
            match_exact_search: false,
            hazardstatement_id: row.get_unwrap("hazardstatement_id"),
            hazardstatement_label: row.get_unwrap("hazardstatement_label"),
            hazardstatement_reference: row.get_unwrap("hazardstatement_reference"),
        }
    }
}

pub fn get_hazardstatements(
    db_connection: &Connection,
    filter: RequestFilter,
) -> Result<(Vec<HazardstatementStruct>, usize), Box<dyn std::error::Error>> {
    debug!("filter:{:?}", filter);

    let (sql, values) = Query::select()
        .columns([
            Hazardstatement::HazardstatementId,
            Hazardstatement::HazardstatementLabel,
            Hazardstatement::HazardstatementReference,
        ])
        .from(Hazardstatement::Table)
        .conditions(
            filter.search.is_some(),
            |q| {
                q.and_where(
                    Expr::col(Hazardstatement::HazardstatementReference)
                        .like(format!("%{}%", filter.search.clone().unwrap())),
                );
            },
            |_| {},
        )
        .order_by(Hazardstatement::HazardstatementReference, Order::Asc)
        .build_rusqlite(SqliteQueryBuilder);

    let mut stmt = db_connection.prepare(sql.as_str())?;
    let rows = stmt.query_map(&*values.as_params(), |row| {
        Ok(HazardstatementStruct::from(row))
    })?;

    // Result statements and count.
    let mut hazardstatements = Vec::new();
    let mut count = 0;
    for maybe_hazardstatement in rows {
        let mut hazardstatement = maybe_hazardstatement?;

        // Set match_exact_search for statement matching filter.search.
        if filter.search.is_some()
            && hazardstatement
                .hazardstatement_reference
                .eq(&filter.search.clone().unwrap())
        {
            hazardstatement.match_exact_search = true;

            // Inserting the statement at the beginning of the results.
            hazardstatements.insert(0, hazardstatement)
        } else {
            // Inserting the statement at the end of the results.
            hazardstatements.push(hazardstatement);
        }

        count += 1;
    }

    debug!("hazardstatements: {:#?}", hazardstatements);

    Ok((hazardstatements, count))
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

        // insert fake hazardstatements.
        let _ = db_connection
            .execute(
                "INSERT INTO hazardstatement (hazardstatement_label, hazardstatement_reference) VALUES (?1, ?2)",
                [String::from("hazardstatement1"), String::from("hazardstatement1-ref")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO hazardstatement (hazardstatement_label, hazardstatement_reference) VALUES (?1, ?2)",
                [String::from("aa hazardstatement1"), String::from("aa hazardstatement1-ref")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO hazardstatement (hazardstatement_label, hazardstatement_reference) VALUES (?1, ?2)",
                [String::from("hazardstatement2"), String::from("hazardstatement2-ref")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO hazardstatement (hazardstatement_label, hazardstatement_reference) VALUES (?1, ?2)",
                [String::from("hazardstatement3"), String::from("hazardstatement3-ref")],
            )
            .unwrap();

        db_connection
    }

    #[test]
    fn test_get_hazardstatements() {
        init_logger();

        let db_connection = init_test_db();

        info!("testing ok result");
        let filter = RequestFilter {
            ..Default::default()
        };
        assert!(get_hazardstatements(&db_connection, filter,).is_ok());

        info!("testing filter search");
        let filter = RequestFilter {
            search: Some(String::from("hazardstatement1-ref")),
            ..Default::default()
        };
        let (hazardstatements, count) = get_hazardstatements(&db_connection, filter).unwrap();

        // expected number of results.
        assert_eq!(count, 2);
        // expected exact match appears first.
        assert!(hazardstatements[0]
            .hazardstatement_reference
            .eq("hazardstatement1-ref"));
        assert!(hazardstatements[0]
            .hazardstatement_label
            .eq("hazardstatement1"));
    }
}
