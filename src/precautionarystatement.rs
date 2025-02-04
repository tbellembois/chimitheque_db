use chimitheque_types::{
    precautionarystatement::PrecautionaryStatement as PrecautionaryStatementStruct,
    requestfilter::RequestFilter,
};
use log::debug;
use rusqlite::{Connection, Row};
use sea_query::{Expr, Iden, Order, Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use serde::Serialize;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum PrecautionaryStatement {
    Table,
    PrecautionaryStatementId,
    PrecautionaryStatementLabel,
    PrecautionaryStatementReference,
}

#[derive(Debug, Serialize, Default)]
pub struct PrecautionaryStatementWrapper(pub PrecautionaryStatementStruct);

impl From<&Row<'_>> for PrecautionaryStatementWrapper {
    fn from(row: &Row) -> Self {
        Self({
            PrecautionaryStatementStruct {
                match_exact_search: false,
                precautionary_statement_id: row.get_unwrap("precautionary_statement_id"),
                precautionary_statement_label: row.get_unwrap("precautionary_statement_label"),
                precautionary_statement_reference: row
                    .get_unwrap("precautionary_statement_reference"),
            }
        })
    }
}

pub fn parse(
    db_connection: &Connection,
    s: &str,
) -> Result<Option<PrecautionaryStatementStruct>, Box<dyn std::error::Error>> {
    debug!("s:{:?}", s);

    let (select_sql, select_values) = Query::select()
        .columns([
            PrecautionaryStatement::PrecautionaryStatementId,
            PrecautionaryStatement::PrecautionaryStatementLabel,
            PrecautionaryStatement::PrecautionaryStatementReference,
        ])
        .from(PrecautionaryStatement::Table)
        .cond_where(Expr::col(PrecautionaryStatement::PrecautionaryStatementReference).eq(s))
        .build_rusqlite(SqliteQueryBuilder);

    debug!("select_sql: {}", select_sql.clone().as_str());
    debug!("select_values: {:?}", select_values);

    // Perform select query.
    let mut stmt = db_connection.prepare(&select_sql)?;
    let mayerr_query = stmt.query_row(&*select_values.as_params(), |row| {
        Ok(Some(PrecautionaryStatementStruct {
            match_exact_search: false,
            precautionary_statement_id: row.get_unwrap(0),
            precautionary_statement_label: row.get_unwrap(1),
            precautionary_statement_reference: row.get_unwrap(2),
        }))
    });

    match mayerr_query {
        Ok(precautionary_statement) => Ok(precautionary_statement),
        Err(e) => match e {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            _ => Err(Box::new(e)),
        },
    }
}

pub fn get_precautionary_statements(
    db_connection: &Connection,
    filter: RequestFilter,
) -> Result<(Vec<PrecautionaryStatementWrapper>, usize), Box<dyn std::error::Error>> {
    debug!("filter:{:?}", filter);

    // Create common query statement.
    let mut expression = Query::select();
    expression.from(PrecautionaryStatement::Table).conditions(
        filter.search.is_some(),
        |q| {
            q.and_where(
                Expr::col(PrecautionaryStatement::PrecautionaryStatementReference)
                    .like(format!("%{}%", filter.search.clone().unwrap())),
            );
        },
        |_| {},
    );

    // Create count query.
    let (count_sql, count_values) = expression
        .clone()
        .expr(
            Expr::col((
                PrecautionaryStatement::Table,
                PrecautionaryStatement::PrecautionaryStatementId,
            ))
            .count_distinct(),
        )
        .build_rusqlite(SqliteQueryBuilder);

    debug!("count_sql: {}", count_sql.clone().as_str());
    debug!("count_values: {:?}", count_values);

    // Create select query.
    let (select_sql, select_values) = expression
        .columns([
            PrecautionaryStatement::PrecautionaryStatementId,
            PrecautionaryStatement::PrecautionaryStatementLabel,
            PrecautionaryStatement::PrecautionaryStatementReference,
        ])
        .order_by(
            PrecautionaryStatement::PrecautionaryStatementReference,
            Order::Asc,
        )
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
        Ok(PrecautionaryStatementWrapper::from(row))
    })?;

    // Build result.
    let mut precautionary_statements = Vec::new();
    for maybe_precautionary_statement in rows {
        let mut precautionary_statement = maybe_precautionary_statement?;

        // Set match_exact_search for statement matching filter.search.
        if filter.search.is_some()
            && precautionary_statement
                .0
                .precautionary_statement_reference
                .eq(&filter.search.clone().unwrap())
        {
            precautionary_statement.0.match_exact_search = true;

            // Inserting the statement at the beginning of the results.
            precautionary_statements.insert(0, precautionary_statement)
        } else {
            // Inserting the statement at the end of the results.
            precautionary_statements.push(precautionary_statement);
        }
    }

    debug!("precautionary_statements: {:#?}", precautionary_statements);

    Ok((precautionary_statements, count))
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

        // insert fake precautionary_statements.
        let _ = db_connection
            .execute(
                "INSERT INTO precautionary_statement (precautionary_statement_label, precautionary_statement_reference) VALUES (?1, ?2)",
                [String::from("precautionary_statement1"), String::from("precautionary_statement1-ref")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO precautionary_statement (precautionary_statement_label, precautionary_statement_reference) VALUES (?1, ?2)",
                [String::from("aa precautionary_statement1"), String::from("aa precautionary_statement1-ref")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO precautionary_statement (precautionary_statement_label, precautionary_statement_reference) VALUES (?1, ?2)",
                [String::from("precautionary_statement2"), String::from("precautionary_statement2-ref")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO precautionary_statement (precautionary_statement_label, precautionary_statement_reference) VALUES (?1, ?2)",
                [String::from("precautionary_statement3"), String::from("precautionary_statement3-ref")],
            )
            .unwrap();

        db_connection
    }

    #[test]
    fn test_parse_precautionary_statement() {
        init_logger();

        let db_connection = init_test_db();

        info!("testing parse");
        assert!(parse(&db_connection, "P322").is_ok_and(|u| u.is_some()));
        assert!(parse(&db_connection, "not exist").is_ok_and(|u| u.is_none()));
    }

    #[test]
    fn test_get_precautionary_statements() {
        init_logger();

        let db_connection = init_test_db();

        info!("testing ok result");
        let filter = RequestFilter {
            ..Default::default()
        };
        assert!(get_precautionary_statements(&db_connection, filter,).is_ok());

        info!("testing filter search");
        let filter = RequestFilter {
            search: Some(String::from("precautionary_statement1-ref")),
            ..Default::default()
        };
        let (precautionary_statements, count) =
            get_precautionary_statements(&db_connection, filter).unwrap();

        // expected number of results.
        assert_eq!(count, 2);
        // expected exact match appears first.
        assert!(precautionary_statements[0]
            .0
            .precautionary_statement_reference
            .eq("precautionary_statement1-ref"));
        assert!(precautionary_statements[0]
            .0
            .precautionary_statement_label
            .eq("precautionary_statement1"));
    }
}
