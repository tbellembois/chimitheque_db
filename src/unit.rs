use chimitheque_types::requestfilter::RequestFilter;
use log::debug;
use rusqlite::{Connection, Row};
use sea_query::{Alias, Expr, Iden, JoinType, Order, Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use serde::Serialize;
use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

#[derive(Debug, Serialize, Clone)]
enum UnitType {
    Quantity,
    Concentration,
    Temperature,
    MolecularWeight,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseUnitTypeError;

impl Display for ParseUnitTypeError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "wrong unit type")
    }
}

impl std::error::Error for ParseUnitTypeError {}

impl FromStr for UnitType {
    type Err = ParseUnitTypeError;

    fn from_str(input: &str) -> Result<UnitType, Self::Err> {
        match input {
            "quantity" => Ok(UnitType::Quantity),
            "concentration" => Ok(UnitType::Concentration),
            "temperature" => Ok(UnitType::Temperature),
            "molecularweight" => Ok(UnitType::MolecularWeight),
            _ => Err(ParseUnitTypeError),
        }
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
enum Unit {
    Table,
    UnitId,
    UnitLabel,
    UnitMultiplier,
    UnitType,
    Unit,
}

#[derive(Debug, Serialize)]
pub struct UnitStruct {
    unit_id: u64,
    unit_label: String,
    unit_multiplier: f64,
    unit_type: UnitType,

    unit: Option<Box<UnitStruct>>,
}

impl TryFrom<&Row<'_>> for UnitStruct {
    type Error = ParseUnitTypeError;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        let unit_type_string: String = row.get_unwrap("unit_type");
        let unit_type = UnitType::from_str(&unit_type_string)?;

        // Test if there is a parent unit.
        let maybe_parent_unit: Option<u64> = row.get_unwrap("parent_unit_id");

        // We assume that the unit_type is the same for the parent.
        Ok(Self {
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
        })
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
        let unit = UnitStruct::try_from(row)?;
        units.push(unit);
    }

    debug!("units: {:#?}", units);

    Ok((units, count))
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

        db_connection
    }

    #[test]
    fn test_get_units() {
        init_logger();

        let mut db_connection = init_test_db();
        init_db(&mut db_connection).unwrap();

        info!("testing total result");
        let filter = RequestFilter {
            ..Default::default()
        };
        let (units, count) = get_units(&db_connection, filter).unwrap();
        assert_eq!(count, 22);
        assert_eq!(units.len(), 22);
    }
}
