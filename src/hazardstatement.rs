use chimitheque_types::{
    hazardstatement::Hazardstatement as HazardstatementStruct, requestfilter::RequestFilter,
};
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
pub struct HazardstatementWrapper(pub HazardstatementStruct);

impl From<&Row<'_>> for HazardstatementWrapper {
    fn from(row: &Row) -> Self {
        Self({
            HazardstatementStruct {
                match_exact_search: false,
                hazardstatement_id: row.get_unwrap("hazardstatement_id"),
                hazardstatement_label: row.get_unwrap("hazardstatement_label"),
                hazardstatement_reference: row.get_unwrap("hazardstatement_reference"),
            }
        })
    }
}

pub fn parse(
    db_connection: &Connection,
    s: &str,
) -> Result<Option<HazardstatementStruct>, Box<dyn std::error::Error>> {
    debug!("s:{:?}", s);

    let (select_sql, select_values) = Query::select()
        .columns([
            Hazardstatement::HazardstatementId,
            Hazardstatement::HazardstatementLabel,
            Hazardstatement::HazardstatementReference,
        ])
        .from(Hazardstatement::Table)
        .cond_where(Expr::col(Hazardstatement::HazardstatementReference).eq(s))
        .build_rusqlite(SqliteQueryBuilder);

    debug!("select_sql: {}", select_sql.clone().as_str());
    debug!("select_values: {:?}", select_values);

    // Perform select query.
    let mut stmt = db_connection.prepare(&select_sql)?;
    let mayerr_query = stmt.query_row(&*select_values.as_params(), |row| {
        Ok(Some(HazardstatementStruct {
            match_exact_search: false,
            hazardstatement_id: row.get_unwrap(0),
            hazardstatement_label: row.get_unwrap(1),
            hazardstatement_reference: row.get_unwrap(2),
        }))
    });

    match mayerr_query {
        Ok(hazardstatement) => Ok(hazardstatement),
        Err(e) => match e {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            _ => Err(Box::new(e)),
        },
    }
}

pub fn get_hazardstatements(
    db_connection: &Connection,
    filter: RequestFilter,
) -> Result<(Vec<HazardstatementStruct>, usize), Box<dyn std::error::Error>> {
    debug!("filter:{:?}", filter);

    // Create common query statement.
    let mut expression = Query::select();
    expression.from(Hazardstatement::Table).conditions(
        filter.search.is_some(),
        |q| {
            q.and_where(
                Expr::col(Hazardstatement::HazardstatementReference)
                    .like(format!("%{}%", filter.search.clone().unwrap())),
            );
        },
        |_| {},
    );

    // Create count query.
    let (count_sql, count_values) = expression
        .clone()
        .expr(
            Expr::col((Hazardstatement::Table, Hazardstatement::HazardstatementId))
                .count_distinct(),
        )
        .build_rusqlite(SqliteQueryBuilder);

    debug!("count_sql: {}", count_sql.clone().as_str());
    debug!("count_values: {:?}", count_values);

    // Create select query.
    let (select_sql, select_values) = expression
        .columns([
            Hazardstatement::HazardstatementId,
            Hazardstatement::HazardstatementLabel,
            Hazardstatement::HazardstatementReference,
        ])
        .order_by(Hazardstatement::HazardstatementReference, Order::Asc)
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
        Ok(HazardstatementWrapper::from(row))
    })?;

    // Build result.
    let mut hazardstatements = Vec::new();
    for maybe_hazardstatement in rows {
        let mut hazardstatement = maybe_hazardstatement?;

        // Set match_exact_search for statement matching filter.search.
        if filter.search.is_some()
            && hazardstatement
                .0
                .hazardstatement_reference
                .eq(&filter.search.clone().unwrap())
        {
            hazardstatement.0.match_exact_search = true;

            // Inserting the statement at the beginning of the results.
            hazardstatements.insert(0, hazardstatement.0)
        } else {
            // Inserting the statement at the end of the results.
            hazardstatements.push(hazardstatement.0);
        }
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
    fn test_parse_hazardstatement() {
        init_logger();

        let mut db_connection = init_test_db();
        init_db(&mut db_connection).unwrap();

        info!("testing parse");
        assert!(parse(&db_connection, "EUH209A").is_ok_and(|u| u.is_some()));
        assert!(parse(&db_connection, "not exist").is_ok_and(|u| u.is_none()));
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
