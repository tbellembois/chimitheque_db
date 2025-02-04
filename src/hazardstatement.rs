use chimitheque_types::{
    hazardstatement::HazardStatement as HazardStatementStruct, requestfilter::RequestFilter,
};
use log::debug;
use rusqlite::{Connection, Row};
use sea_query::{Expr, Iden, Order, Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use serde::Serialize;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum HazardStatement {
    Table,
    HazardStatementId,
    HazardStatementLabel,
    HazardStatementReference,
    HazardStatementCmr,
}

#[derive(Debug, Serialize, Default)]
pub struct HazardStatementWrapper(pub HazardStatementStruct);

impl From<&Row<'_>> for HazardStatementWrapper {
    fn from(row: &Row) -> Self {
        Self({
            HazardStatementStruct {
                match_exact_search: false,
                hazard_statement_id: row.get_unwrap("hazard_statement_id"),
                hazard_statement_label: row.get_unwrap("hazard_statement_label"),
                hazard_statement_reference: row.get_unwrap("hazard_statement_reference"),
                hazard_statement_cmr: row.get_unwrap("hazard_statement_cmr"),
            }
        })
    }
}

pub fn parse(
    db_connection: &Connection,
    s: &str,
) -> Result<Option<HazardStatementStruct>, Box<dyn std::error::Error>> {
    debug!("s:{:?}", s);

    let (select_sql, select_values) = Query::select()
        .columns([
            HazardStatement::HazardStatementId,
            HazardStatement::HazardStatementLabel,
            HazardStatement::HazardStatementReference,
            HazardStatement::HazardStatementCmr,
        ])
        .from(HazardStatement::Table)
        .cond_where(Expr::col(HazardStatement::HazardStatementReference).eq(s))
        .build_rusqlite(SqliteQueryBuilder);

    debug!("select_sql: {}", select_sql.clone().as_str());
    debug!("select_values: {:?}", select_values);

    // Perform select query.
    let mut stmt = db_connection.prepare(&select_sql)?;
    let mayerr_query = stmt.query_row(&*select_values.as_params(), |row| {
        Ok(Some(HazardStatementStruct {
            match_exact_search: false,
            hazard_statement_id: row.get_unwrap(0),
            hazard_statement_label: row.get_unwrap(1),
            hazard_statement_reference: row.get_unwrap(2),
            hazard_statement_cmr: row.get_unwrap(3),
        }))
    });

    match mayerr_query {
        Ok(hazard_statement) => Ok(hazard_statement),
        Err(e) => match e {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            _ => Err(Box::new(e)),
        },
    }
}

pub fn get_hazard_statements(
    db_connection: &Connection,
    filter: RequestFilter,
) -> Result<(Vec<HazardStatementStruct>, usize), Box<dyn std::error::Error>> {
    debug!("filter:{:?}", filter);

    // Create common query statement.
    let mut expression = Query::select();
    expression.from(HazardStatement::Table).conditions(
        filter.search.is_some(),
        |q| {
            q.and_where(
                Expr::col(HazardStatement::HazardStatementReference)
                    .like(format!("%{}%", filter.search.clone().unwrap())),
            );
        },
        |_| {},
    );

    // Create count query.
    let (count_sql, count_values) = expression
        .clone()
        .expr(
            Expr::col((HazardStatement::Table, HazardStatement::HazardStatementId))
                .count_distinct(),
        )
        .build_rusqlite(SqliteQueryBuilder);

    debug!("count_sql: {}", count_sql.clone().as_str());
    debug!("count_values: {:?}", count_values);

    // Create select query.
    let (select_sql, select_values) = expression
        .columns([
            HazardStatement::HazardStatementId,
            HazardStatement::HazardStatementLabel,
            HazardStatement::HazardStatementReference,
            HazardStatement::HazardStatementCmr,
        ])
        .order_by(HazardStatement::HazardStatementReference, Order::Asc)
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
        Ok(HazardStatementWrapper::from(row))
    })?;

    // Build result.
    let mut hazard_statements = Vec::new();
    for maybe_hazard_statement in rows {
        let mut hazard_statement = maybe_hazard_statement?;

        // Set match_exact_search for statement matching filter.search.
        if filter.search.is_some()
            && hazard_statement
                .0
                .hazard_statement_reference
                .eq(&filter.search.clone().unwrap())
        {
            hazard_statement.0.match_exact_search = true;

            // Inserting the statement at the beginning of the results.
            hazard_statements.insert(0, hazard_statement.0)
        } else {
            // Inserting the statement at the end of the results.
            hazard_statements.push(hazard_statement.0);
        }
    }

    debug!("hazard_statements: {:#?}", hazard_statements);

    Ok((hazard_statements, count))
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

        // insert fake hazard statements.
        let _ = db_connection
            .execute(
                "INSERT INTO hazard_statement (hazard_statement_label, hazard_statement_reference) VALUES (?1, ?2)",
                [String::from("hazard_statement1"), String::from("hazard_statement1-ref")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO hazard_statement (hazard_statement_label, hazard_statement_reference) VALUES (?1, ?2)",
                [String::from("aa hazard_statement1"), String::from("aa hazard_statement1-ref")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO hazard_statement (hazard_statement_label, hazard_statement_reference) VALUES (?1, ?2)",
                [String::from("hazard_statement2"), String::from("hazard_statement2-ref")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO hazard_statement (hazard_statement_label, hazard_statement_reference) VALUES (?1, ?2)",
                [String::from("hazard_statement3"), String::from("hazard_statement3-ref")],
            )
            .unwrap();

        db_connection
    }

    #[test]
    fn test_parse_hazard_statement() {
        init_logger();

        let db_connection = init_test_db();

        info!("testing parse");
        assert!(parse(&db_connection, "EUH209A").is_ok_and(|u| u.is_some()));
        assert!(parse(&db_connection, "not exist").is_ok_and(|u| u.is_none()));
    }

    #[test]
    fn test_get_hazard_statements() {
        init_logger();

        let db_connection = init_test_db();

        info!("testing ok result");
        let filter = RequestFilter {
            ..Default::default()
        };
        assert!(get_hazard_statements(&db_connection, filter,).is_ok());

        info!("testing filter search");
        let filter = RequestFilter {
            search: Some(String::from("hazard_statement1-ref")),
            ..Default::default()
        };
        let (hazard_statements, count) = get_hazard_statements(&db_connection, filter).unwrap();

        // expected number of results.
        assert_eq!(count, 2);
        // expected exact match appears first.
        assert!(hazard_statements[0]
            .hazard_statement_reference
            .eq("hazard_statement1-ref"));
        assert!(hazard_statements[0]
            .hazard_statement_label
            .eq("hazard_statement1"));
    }
}
