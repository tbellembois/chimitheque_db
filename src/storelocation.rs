use crate::entity::{Entity, EntityStruct};
use chimitheque_types::requestfilter::RequestFilter;
use log::debug;
use rusqlite::{Connection, Row};
use sea_query::{Alias, Expr, Iden, JoinType, Order, Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use serde::Serialize;
use std::{error::Error, str::FromStr};

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
enum Storelocation {
    Table,
    StorelocationId,
    StorelocationName,
    StorelocationCanstore,
    StorelocationColor,
    StorelocationFullpath,
    Entity,
    Storelocation,
}

impl FromStr for Storelocation {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Storelocation, Self::Err> {
        match s {
            "entity.entity_name" => Ok(Storelocation::Entity),
            "storelocation" => Ok(Storelocation::Storelocation),
            _ => Ok(Storelocation::StorelocationFullpath),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct StorelocationStruct {
    storelocation_id: u64,
    storelocation_name: String,
    storelocation_canstore: bool,
    storelocation_color: Option<String>,
    storelocation_fullpath: Option<String>,

    entity: Option<EntityStruct>,
    storelocation: Option<Box<StorelocationStruct>>,
}

impl From<&Row<'_>> for StorelocationStruct {
    fn from(row: &Row) -> Self {
        dbg!(row);

        // Test if there is a parent storelocation.
        let maybe_parent_storelocation: Option<u64> = row.get_unwrap("parent_storelocation_id");

        Self {
            storelocation_id: row.get_unwrap("storelocation_id"),
            storelocation_name: row.get_unwrap("storelocation_name"),
            storelocation_canstore: row.get_unwrap("storelocation_canstore"),
            storelocation_color: row.get_unwrap("storelocation_color"),
            storelocation_fullpath: row.get_unwrap("storelocation_fullpath"),
            entity: Some(EntityStruct {
                entity_id: row.get_unwrap("entity_id"),
                entity_name: row.get_unwrap("entity_name"),
            }),
            // storelocation: Some(Box::new(StorelocationStruct {
            //     storelocation_id: row.get_unwrap("parent_storelocation_id"),
            //     storelocation_name: row.get_unwrap("parent_storelocation_name"),
            //     storelocation_canstore: row.get_unwrap("parent_storelocation_canstore"),
            //     storelocation_color: row.get_unwrap("parent_storelocation_color"),
            //     storelocation_fullpath: row.get_unwrap("parent_storelocation_fullpath"),
            //     entity: None,
            //     storelocation: None,
            // })),
            storelocation: maybe_parent_storelocation.map(|_| {
                Box::new(StorelocationStruct {
                    storelocation_id: row.get_unwrap("parent_storelocation_id"),
                    storelocation_name: row.get_unwrap("parent_storelocation_name"),
                    storelocation_canstore: row.get_unwrap("parent_storelocation_canstore"),
                    storelocation_color: row.get_unwrap("parent_storelocation_color"),
                    storelocation_fullpath: row.get_unwrap("parent_storelocation_fullpath"),
                    entity: None,
                    storelocation: None,
                })
            }),
        }
    }
}

pub fn get_storelocations(
    db_connection: &Connection,
    filter: RequestFilter,
) -> Result<(Vec<StorelocationStruct>, usize), Box<dyn std::error::Error>> {
    debug!("filter:{:?}", filter);

    let order_by = if let Some(order_by) = filter.order_by {
        Storelocation::from_str(order_by.as_str())?
    } else {
        Storelocation::StorelocationFullpath
    };

    let order = if filter.order.eq_ignore_ascii_case("desc") {
        Order::Desc
    } else {
        Order::Asc
    };

    let (sql, values) = Query::select()
        .columns([Entity::EntityId, Entity::EntityName])
        .expr(Expr::col((
            Storelocation::Table,
            Storelocation::StorelocationId,
        )))
        .expr(Expr::col((
            Storelocation::Table,
            Storelocation::StorelocationName,
        )))
        .expr(Expr::col((
            Storelocation::Table,
            Storelocation::StorelocationCanstore,
        )))
        .expr(Expr::col((
            Storelocation::Table,
            Storelocation::StorelocationColor,
        )))
        .expr(Expr::col((
            Storelocation::Table,
            Storelocation::StorelocationFullpath,
        )))
        .expr_as(
            Expr::col((Alias::new("parent"), Alias::new("storelocation_id"))),
            Alias::new("parent_storelocation_id"),
        )
        .expr_as(
            Expr::col((Alias::new("parent"), Alias::new("storelocation_name"))),
            Alias::new("parent_storelocation_name"),
        )
        .expr_as(
            Expr::col((Alias::new("parent"), Alias::new("storelocation_canstore"))),
            Alias::new("parent_storelocation_canstore"),
        )
        .expr_as(
            Expr::col((Alias::new("parent"), Alias::new("storelocation_color"))),
            Alias::new("parent_storelocation_color"),
        )
        .expr_as(
            Expr::col((Alias::new("parent"), Alias::new("storelocation_fullpath"))),
            Alias::new("parent_storelocation_fullpath"),
        )
        .from(Storelocation::Table)
        .join(
            JoinType::LeftJoin,
            Entity::Table,
            Expr::col((Storelocation::Table, Storelocation::Entity))
                .equals((Entity::Table, Entity::EntityId)),
        )
        .join_as(
            JoinType::LeftJoin,
            Storelocation::Table,
            Alias::new("parent"),
            Expr::col((Storelocation::Table, Storelocation::Storelocation))
                .equals((Alias::new("parent"), Alias::new("storelocation_id"))),
        )
        .conditions(
            filter.search.is_some(),
            |q| {
                q.and_where(
                    Expr::col((Storelocation::Table, Storelocation::StorelocationName))
                        .like(format!("%{}%", filter.search.clone().unwrap())),
                );
            },
            |_| {},
        )
        .conditions(
            filter.entity.is_some(),
            |q| {
                q.and_where(Expr::col(Storelocation::Entity).eq(filter.entity.unwrap()));
            },
            |_| {},
        )
        .conditions(
            filter.store_location_can_store,
            |q| {
                q.and_where(
                    Expr::col(Storelocation::StorelocationCanstore)
                        .eq(filter.store_location_can_store),
                );
            },
            |_| {},
        )
        .order_by(order_by, order)
        .build_rusqlite(SqliteQueryBuilder);

    dbg!(sql.clone());

    let mut stmt = db_connection.prepare(sql.as_str())?;
    let rows = stmt.query_map(&*values.as_params(), |row| {
        Ok(StorelocationStruct::from(row))
    })?;

    // Result and count.
    let mut storelocations = Vec::new();
    let mut count = 0;
    for maybe_storelocation in rows {
        let storelocation = maybe_storelocation?;

        storelocations.push(storelocation);
        count += 1;
    }

    debug!("storelocations: {:#?}", storelocations);

    Ok((storelocations, count))
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
    fn test_get_storelocations() {
        init_logger();

        let mut db_connection = init_test_db();
        init_db(&mut db_connection).unwrap();

        // insert fake entities.
        let _ = db_connection
            .execute(
                "INSERT INTO entity (entity_id, entity_name) VALUES (?1, ?2)",
                (200, String::from("FAKE_ENTITY_1")),
            )
            .unwrap();

        // insert fake storelocations.
        let _ = db_connection
            .execute(
                "INSERT INTO storelocation (storelocation_id, storelocation_name, entity) VALUES (?1, ?2, ?3)",
                (300, String::from("FAKE_STORELOCATION_1"), 200),
            )
            .unwrap();

        let filter = RequestFilter {
            ..Default::default()
        };
        debug!("{:?}", get_storelocations(&db_connection, filter));
    }
}