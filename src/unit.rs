use chimitheque_types::{
    error::ParseError, requestfilter::RequestFilter, unit::Unit as UnitStruct, unittype::UnitType,
};
use log::debug;
use rusqlite::{Connection, Row};
use sea_query::{Alias, Expr, Iden, JoinType, Order, Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use serde::Serialize;
use std::str::FromStr;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Unit {
    Table,
    UnitId,
    UnitLabel,
    UnitMultiplier,
    UnitType,
    Unit,
}

#[derive(Debug, Serialize)]
pub struct UnitWrapper(pub UnitStruct);

impl TryFrom<&Row<'_>> for UnitWrapper {
    type Error = ParseError;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        let unit_type_string: String = row.get_unwrap("unit_type");
        let unit_type = UnitType::from_str(&unit_type_string)?;

        // Test if there is a parent unit.
        let maybe_parent_unit: Option<u64> = row.get_unwrap("parent_unit_id");

        // We assume that the unit_type is the same for the parent.
        Ok(Self({
            UnitStruct {
                unit_id: row.get_unwrap("unit_id"),
                unit_label: row.get_unwrap("unit_label"),
                unit_multiplier: row.get_unwrap("unit_multiplier"),
                unit_type: unit_type.clone(),
                unit: maybe_parent_unit.map(|_| {
                    Box::new(UnitStruct {
                        unit_id: row.get_unwrap("parent_unit_id"),
                        unit_label: row.get_unwrap("parent_unit_label"),
                        unit_multiplier: row.get_unwrap("parent_unit_multiplier"),
                        unit_type,
                        unit: None,
                    })
                }),
            }
        }))
    }
}

pub fn parse(
    db_connection: &Connection,
    s: &str,
) -> Result<Option<UnitStruct>, Box<dyn std::error::Error>> {
    debug!("s:{:?}", s);

    let (select_sql, select_values) = Query::select()
        .expr(Expr::col((Unit::Table, Unit::UnitId)))
        .expr(Expr::col((Unit::Table, Unit::UnitLabel)))
        .expr(Expr::col((Unit::Table, Unit::UnitMultiplier)))
        .expr(Expr::col((Unit::Table, Unit::UnitType)))
        .expr_as(
            Expr::col((Alias::new("parent"), Alias::new("unit_id"))),
            Alias::new("parent_unit_id"),
        )
        .expr_as(
            Expr::col((Alias::new("parent"), Alias::new("unit_label"))),
            Alias::new("parent_unit_label"),
        )
        .expr_as(
            Expr::col((Alias::new("parent"), Alias::new("unit_multiplier"))),
            Alias::new("parent_unit_multiplier"),
        )
        .from(Unit::Table)
        .join_as(
            JoinType::LeftJoin,
            Unit::Table,
            Alias::new("parent"),
            Expr::col((Unit::Table, Unit::Unit))
                .equals((Alias::new("parent"), Alias::new("unit_id"))),
        )
        .cond_where(Expr::col((Unit::Table, Unit::UnitLabel)).eq(s))
        .build_rusqlite(SqliteQueryBuilder);

    debug!("select_sql: {}", select_sql.clone().as_str());
    debug!("select_values: {:?}", select_values);

    // Perform select query.
    let mut stmt = db_connection.prepare(&select_sql)?;
    let mayerr_query = stmt.query_row(&*select_values.as_params(), |row| {
        Ok({
            match UnitWrapper::try_from(row) {
                Ok(unit) => unit.0,
                Err(e) => return Err(rusqlite::Error::ToSqlConversionFailure(Box::new(e))),
            }
        })
    });

    match mayerr_query {
        Ok(unit) => Ok(Some(unit)),
        Err(e) => match e {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            _ => Err(Box::new(e)),
        },
    }
}

pub fn get_units(
    db_connection: &Connection,
    filter: RequestFilter,
) -> Result<(Vec<UnitStruct>, usize), Box<dyn std::error::Error>> {
    debug!("filter:{:?}", filter);

    // Create common query statement.
    let mut expression = Query::select();
    expression
        .from(Unit::Table)
        .join_as(
            JoinType::LeftJoin,
            Unit::Table,
            Alias::new("parent"),
            Expr::col((Unit::Table, Unit::Unit))
                .equals((Alias::new("parent"), Alias::new("unit_id"))),
        )
        .conditions(
            filter.search.is_some(),
            |q| {
                q.and_where(
                    Expr::col((Unit::Table, Unit::UnitLabel))
                        .like(format!("%{}%", filter.search.clone().unwrap())),
                );
            },
            |_| {},
        )
        .conditions(
            filter.unit_type.is_some(),
            |q| {
                q.and_where(Expr::col((Unit::Table, Unit::UnitType)).eq(filter.unit_type.unwrap()));
            },
            |_| {},
        );

    // Create count query.
    let (count_sql, count_values) = expression
        .clone()
        .expr(Expr::col((Unit::Table, Unit::UnitId)).count_distinct())
        .build_rusqlite(SqliteQueryBuilder);

    debug!("count_sql: {}", count_sql.clone().as_str());
    debug!("count_values: {:?}", count_values);

    // Create select query.
    let (select_sql, select_values) = expression
        .expr(Expr::col((Unit::Table, Unit::UnitId)))
        .expr(Expr::col((Unit::Table, Unit::UnitLabel)))
        .expr(Expr::col((Unit::Table, Unit::UnitMultiplier)))
        .expr(Expr::col((Unit::Table, Unit::UnitType)))
        .expr_as(
            Expr::col((Alias::new("parent"), Alias::new("unit_id"))),
            Alias::new("parent_unit_id"),
        )
        .expr_as(
            Expr::col((Alias::new("parent"), Alias::new("unit_label"))),
            Alias::new("parent_unit_label"),
        )
        .expr_as(
            Expr::col((Alias::new("parent"), Alias::new("unit_multiplier"))),
            Alias::new("parent_unit_multiplier"),
        )
        .order_by_columns([
            ((Unit::Table, Unit::UnitType), Order::Asc),
            ((Unit::Table, Unit::UnitLabel), Order::Asc),
        ])
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
    let mut units = Vec::new();
    let mut rows = stmt.query(&*select_values.as_params())?;
    while let Some(row) = rows.next()? {
        let unit = UnitWrapper::try_from(row)?;
        units.push(unit.0);
    }

    debug!("units: {:#?}", units);

    Ok((units, count))
}

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
    fn test_parse_unit() {
        init_logger();

        let db_connection = init_test_db();

        info!("testing parse");
        assert!(parse(&db_connection, "mL").is_ok_and(|u| u.is_some()));
        assert!(parse(&db_connection, "not exist").is_ok_and(|u| u.is_none()));
    }

    #[test]
    fn test_get_units() {
        init_logger();

        let db_connection = init_test_db();

        info!("testing total result");
        let filter = RequestFilter {
            ..Default::default()
        };
        let (units, count) = get_units(&db_connection, filter).unwrap();
        assert_eq!(count, 23);
        assert_eq!(units.len(), 23);

        info!("testing filter");
        let filter = RequestFilter {
            unit_type: Some(UnitType::Quantity.to_string()),
            ..Default::default()
        };
        let (units, count) = get_units(&db_connection, filter).unwrap();
        assert_eq!(count, 10);
        assert_eq!(units.len(), 10);
    }
}
