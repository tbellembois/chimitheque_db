use crate::{
    entity::{Entity, EntityWrapper},
    permission::Permission,
};
use chimitheque_types::{
    entity::Entity as EntityStruct, requestfilter::RequestFilter,
    storelocation::Storelocation as StorelocationStruct,
};
use log::debug;
use rusqlite::{Connection, Row};
use sea_query::{
    Alias, ColumnRef, Expr, Iden, IntoColumnRef, JoinType, Order, Query, SqliteQueryBuilder,
};
use sea_query_rusqlite::RusqliteBinder;
use serde::Serialize;

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

#[derive(Debug, Serialize)]
pub struct StorelocationWrapper(pub StorelocationStruct);

impl From<&Row<'_>> for StorelocationWrapper {
    fn from(row: &Row) -> Self {
        // Test if there is a parent storelocation.
        let maybe_parent_storelocation: Option<u64> = row.get_unwrap("parent_storelocation_id");

        Self({
            StorelocationStruct {
                storelocation_id: row.get_unwrap("storelocation_id"),
                storelocation_name: row.get_unwrap("storelocation_name"),
                storelocation_canstore: row.get_unwrap("storelocation_canstore"),
                storelocation_color: row.get_unwrap("storelocation_color"),
                storelocation_fullpath: row.get_unwrap("storelocation_fullpath"),
                entity: Some(EntityStruct {
                    entity_id: row.get_unwrap("entity_id"),
                    entity_name: row.get_unwrap("entity_name"),
                }),
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
        })
    }
}

pub fn get_storelocations(
    db_connection: &Connection,
    filter: RequestFilter,
    person_id: u64,
) -> Result<(Vec<StorelocationStruct>, usize), Box<dyn std::error::Error>> {
    debug!("filter:{:?}", filter);
    debug!("person_id:{:?}", person_id);

    let order_by: ColumnRef = if let Some(order_by_string) = filter.order_by {
        match order_by_string.as_str() {
            "entity.entity_name" => Entity::EntityName.into_column_ref(),
            "storelocation" => Alias::new("parent_storelocation_fullpath").into_column_ref(),
            "storelocation_fullpath" => {
                (Storelocation::Table, Storelocation::StorelocationFullpath).into_column_ref()
            }
            _ => (Storelocation::Table, Storelocation::StorelocationFullpath).into_column_ref(),
        }
    } else {
        (Storelocation::Table, Storelocation::StorelocationFullpath).into_column_ref()
    };

    let order = if filter.order.eq_ignore_ascii_case("desc") {
        Order::Desc
    } else {
        Order::Asc
    };

    // Create common query statement.
    let mut expression = Query::select();
    expression
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
        .join_as(
            JoinType::InnerJoin,
            Permission::Table,
            Alias::new("perm"),
            Expr::col((Alias::new("perm"), Alias::new("person")))
                .eq(person_id)
                .and(
                    Expr::col((Alias::new("perm"), Alias::new("permission_item_name")))
                        .is_in(["all", "storages"]),
                )
                .and(
                    Expr::col((Alias::new("perm"), Alias::new("permission_perm_name")))
                        .is_in(["r", "w", "all"]),
                )
                .and(
                    Expr::col((Alias::new("perm"), Alias::new("permission_entity_id")))
                        .equals(Entity::EntityId)
                        .or(
                            Expr::col((Alias::new("perm"), Alias::new("permission_entity_id")))
                                .eq(-1),
                        ),
                ),
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
                q.and_where(Expr::col(Entity::EntityId).eq(filter.entity.unwrap()));
            },
            |_| {},
        )
        .conditions(
            filter.store_location_can_store,
            |q| {
                q.and_where(
                    Expr::col((Storelocation::Table, Storelocation::StorelocationCanstore))
                        .eq(filter.store_location_can_store),
                );
            },
            |_| {},
        );

    // Create count query.
    let (count_sql, count_values) = expression
        .clone()
        .expr(Expr::col((Storelocation::Table, Storelocation::StorelocationId)).count_distinct())
        .build_rusqlite(SqliteQueryBuilder);

    debug!("count_sql: {}", count_sql.clone().as_str());
    debug!("count_values: {:?}", count_values);

    // Create select query.
    let (select_sql, select_values) = expression
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
        .order_by(order_by, order)
        .group_by_col((Storelocation::Table, Storelocation::StorelocationId))
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
        Ok(StorelocationWrapper::from(row).0)
    })?;

    // Build select result.
    let mut storelocations = Vec::new();
    for maybe_storelocation in rows {
        let storelocation = maybe_storelocation?;

        storelocations.push(storelocation);
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
        for (entity_id, entity_name) in [
            (200, "FAKE_ENTITY_1"),
            (201, "FAKE_ENTITY_2"),
            (202, "FAKE_ENTITY_3"),
        ]
        .iter()
        {
            let _ = db_connection
                .execute(
                    "INSERT INTO entity (entity_id, entity_name) VALUES (?1, ?2)",
                    (entity_id, entity_name),
                )
                .unwrap();
        }

        // Insert fake users.
        let _ = db_connection
            .execute(
                "INSERT INTO person (person_id, person_email) VALUES (?1, ?2)",
                (2, "person1"),
            )
            .unwrap();
        let _ = db_connection
        .execute(
            "INSERT INTO personentities (personentities_person_id, personentities_entity_id) VALUES (?1, ?2)",
            (2, 200),
        )
        .unwrap();
        // set user an admin.
        let _ = db_connection
        .execute(
            "INSERT INTO permission (person, permission_perm_name, permission_item_name, permission_entity_id) VALUES (?1, ?2, ?3, ?4)",
            (2, "all", "all", -1),
        )
        .unwrap();

        // Insert fake storelocations.
        let mut storelocation_id = 300;
        for (storelocation_name, storelocaion_canstore, entity) in [
            ("FAKE_STORELOCATION_11", false, 200),
            ("FAKE_STORELOCATION_12", false, 200),
            ("FAKE_STORELOCATION_13", false, 200),
            ("FAKE_STORELOCATION_14", false, 200),
            ("FAKE_STORELOCATION_15", false, 200),
            ("FAKE_STORELOCATION_16", false, 200),
            ("FAKE_STORELOCATION_17", false, 200),
            ("FAKE_STORELOCATION_18", false, 200),
            ("FAKE_STORELOCATION_19", false, 200),
            ("FAKE_STORELOCATION_21", false, 201),
            ("FAKE_STORELOCATION_22", true, 201),
        ]
        .iter()
        {
            let _ = db_connection
            .execute(
                "INSERT INTO storelocation (storelocation_id, storelocation_name, storelocation_canstore, entity) VALUES (?1, ?2, ?3, ?4)",
                (storelocation_id, storelocation_name, storelocaion_canstore, entity),
            )
            .unwrap();
            storelocation_id += 1;
        }

        info!("testing total result");
        let filter = RequestFilter {
            ..Default::default()
        };
        let (storelocations, count) = get_storelocations(&db_connection, filter, 2).unwrap();
        assert_eq!(count, 11);
        assert_eq!(storelocations.len(), 11);

        info!("testing entity filter");
        let filter = RequestFilter {
            entity: Some(201),
            ..Default::default()
        };
        let (storelocations, count) = get_storelocations(&db_connection, filter, 2).unwrap();

        assert_eq!(count, 2);
        for storelocation in storelocations.iter() {
            assert!(
                (storelocation.storelocation_name.eq("FAKE_STORELOCATION_21")
                    || storelocation.storelocation_name.eq("FAKE_STORELOCATION_22"))
            )
        }

        info!("testing storelocation name filter");
        let filter = RequestFilter {
            search: Some(String::from("FAKE_STORELOCATION_22")),
            ..Default::default()
        };
        let (storelocations, count) = get_storelocations(&db_connection, filter, 2).unwrap();
        assert_eq!(count, 1);
        assert_eq!(
            storelocations[0].storelocation_name,
            "FAKE_STORELOCATION_22"
        );

        info!("testing storelocation canstore filter");
        let filter = RequestFilter {
            store_location_can_store: true,
            ..Default::default()
        };
        let (storelocations, count) = get_storelocations(&db_connection, filter, 2).unwrap();
        assert_eq!(count, 1);
        assert_eq!(
            storelocations[0].storelocation_name,
            "FAKE_STORELOCATION_22"
        );

        info!("testing limit");
        let filter = RequestFilter {
            limit: Some(5),
            ..Default::default()
        };
        let (storelocations, count) = get_storelocations(&db_connection, filter, 2).unwrap();
        assert_eq!(count, 11);
        assert_eq!(storelocations.len(), 5);

        info!("testing permissions filter");
        let filter = RequestFilter {
            ..Default::default()
        };
        let _ = db_connection.execute("DELETE FROM permission", ());
        let _ = db_connection
        .execute(
            "INSERT INTO permission (person, permission_perm_name, permission_item_name, permission_entity_id) VALUES (?1, ?2, ?3, ?4)",
            (2, "r", "storages", 200),
        )
        .unwrap();
        let (_, count) = get_storelocations(&db_connection, filter, 2).unwrap();
        assert_eq!(count, 9);
    }
}
