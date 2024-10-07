use crate::{entity::Entity, permission::Permission};
use chimitheque_types::{
    entity::Entity as EntityStruct, requestfilter::RequestFilter,
    storelocation::StoreLocation as StoreLocationStruct,
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
pub enum StoreLocation {
    Table,
    StoreLocationId,
    StoreLocationName,
    StoreLocationCanStore,
    StoreLocationColor,
    StoreLocationFullPath,
    Entity,
    StoreLocation,
}

#[derive(Debug, Serialize)]
pub struct StoreLocationWrapper(pub StoreLocationStruct);

impl From<&Row<'_>> for StoreLocationWrapper {
    fn from(row: &Row) -> Self {
        // Test if there is a parent store location.
        let maybe_parent_store_location: Option<u64> = row.get_unwrap("parent_store_location_id");

        Self({
            StoreLocationStruct {
                store_location_id: row.get_unwrap("store_location_id"),
                store_location_name: row.get_unwrap("store_location_name"),
                store_location_can_store: row.get_unwrap("store_location_can_store"),
                store_location_color: row.get_unwrap("store_location_color"),
                store_location_full_path: row.get_unwrap("store_location_full_path"),
                entity: Some(EntityStruct {
                    entity_id: row.get_unwrap("entity_id"),
                    entity_name: row.get_unwrap("entity_name"),
                }),
                store_location: maybe_parent_store_location.map(|_| {
                    Box::new(StoreLocationStruct {
                        store_location_id: row.get_unwrap("parent_store_location_id"),
                        store_location_name: row.get_unwrap("parent_store_location_name"),
                        store_location_can_store: row.get_unwrap("parent_store_location_can_store"),
                        store_location_color: row.get_unwrap("parent_store_location_color"),
                        store_location_full_path: row.get_unwrap("parent_store_location_full_path"),
                        entity: None,
                        store_location: None,
                    })
                }),
            }
        })
    }
}

pub fn get_store_locations(
    db_connection: &Connection,
    filter: RequestFilter,
    person_id: u64,
) -> Result<(Vec<StoreLocationStruct>, usize), Box<dyn std::error::Error>> {
    debug!("filter:{:?}", filter);
    debug!("person_id:{:?}", person_id);

    let order_by: ColumnRef = if let Some(order_by_string) = filter.order_by {
        match order_by_string.as_str() {
            "entity.entity_name" => Entity::EntityName.into_column_ref(),
            "store_location" => Alias::new("parent_store_location_fullpath").into_column_ref(),
            "store_location_full_path" => {
                (StoreLocation::Table, StoreLocation::StoreLocationFullPath).into_column_ref()
            }
            _ => (StoreLocation::Table, StoreLocation::StoreLocationFullPath).into_column_ref(),
        }
    } else {
        (StoreLocation::Table, StoreLocation::StoreLocationFullPath).into_column_ref()
    };

    let order = if filter.order.eq_ignore_ascii_case("desc") {
        Order::Desc
    } else {
        Order::Asc
    };

    // Create common query statement.
    let mut expression = Query::select();
    expression
        .from(StoreLocation::Table)
        .join(
            JoinType::LeftJoin,
            Entity::Table,
            Expr::col((StoreLocation::Table, StoreLocation::Entity))
                .equals((Entity::Table, Entity::EntityId)),
        )
        .join_as(
            JoinType::LeftJoin,
            StoreLocation::Table,
            Alias::new("parent"),
            Expr::col((StoreLocation::Table, StoreLocation::StoreLocation))
                .equals((Alias::new("parent"), Alias::new("store_location_id"))),
        )
        //
        // permissions
        //
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
                    Expr::col((StoreLocation::Table, StoreLocation::StoreLocationName))
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
                    Expr::col((StoreLocation::Table, StoreLocation::StoreLocationCanStore))
                        .eq(filter.store_location_can_store),
                );
            },
            |_| {},
        );

    // Create count query.
    let (count_sql, count_values) = expression
        .clone()
        .expr(Expr::col((StoreLocation::Table, StoreLocation::StoreLocationId)).count_distinct())
        .build_rusqlite(SqliteQueryBuilder);

    debug!("count_sql: {}", count_sql.clone().as_str());
    debug!("count_values: {:?}", count_values);

    // Create select query.
    let (select_sql, select_values) = expression
        .columns([Entity::EntityId, Entity::EntityName])
        .expr(Expr::col((
            StoreLocation::Table,
            StoreLocation::StoreLocationId,
        )))
        .expr(Expr::col((
            StoreLocation::Table,
            StoreLocation::StoreLocationName,
        )))
        .expr(Expr::col((
            StoreLocation::Table,
            StoreLocation::StoreLocationCanStore,
        )))
        .expr(Expr::col((
            StoreLocation::Table,
            StoreLocation::StoreLocationColor,
        )))
        .expr(Expr::col((
            StoreLocation::Table,
            StoreLocation::StoreLocationFullPath,
        )))
        .expr_as(
            Expr::col((Alias::new("parent"), Alias::new("store_location_id"))),
            Alias::new("parent_store_location_id"),
        )
        .expr_as(
            Expr::col((Alias::new("parent"), Alias::new("store_location_name"))),
            Alias::new("parent_store_location_name"),
        )
        .expr_as(
            Expr::col((Alias::new("parent"), Alias::new("store_location_can_store"))),
            Alias::new("parent_store_location_can_store"),
        )
        .expr_as(
            Expr::col((Alias::new("parent"), Alias::new("store_location_color"))),
            Alias::new("parent_store_location_color"),
        )
        .expr_as(
            Expr::col((Alias::new("parent"), Alias::new("store_location_full_path"))),
            Alias::new("parent_store_location_full_path"),
        )
        .order_by(order_by, order)
        .group_by_col((StoreLocation::Table, StoreLocation::StoreLocationId))
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
        Ok(StoreLocationWrapper::from(row).0)
    })?;

    // Build select result.
    let mut store_locations = Vec::new();
    for maybe_store_location in rows {
        let store_location = maybe_store_location?;

        store_locations.push(store_location);
    }

    debug!("store_locations: {:#?}", store_locations);

    Ok((store_locations, count))
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
    fn test_get_store_locations() {
        init_logger();

        let mut db_connection = init_test_db();
        init_db(&mut db_connection).unwrap();

        // Insert fake entities.
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
        // Set user an admin.
        let _ = db_connection
        .execute(
            "INSERT INTO permission (person, permission_perm_name, permission_item_name, permission_entity_id) VALUES (?1, ?2, ?3, ?4)",
            (2, "all", "all", -1),
        )
        .unwrap();

        // Insert fake store locations.
        let mut store_location_id = 300;
        for (store_location_name, storelocaion_canstore, entity) in [
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
                "INSERT INTO store_location (store_location_id, store_location_name, store_location_can_store, entity) VALUES (?1, ?2, ?3, ?4)",
                (store_location_id, store_location_name, storelocaion_canstore, entity),
            )
            .unwrap();
            store_location_id += 1;
        }

        info!("testing total result");
        let filter = RequestFilter {
            ..Default::default()
        };
        let (store_locations, count) = get_store_locations(&db_connection, filter, 2).unwrap();
        assert_eq!(count, 11);
        assert_eq!(store_locations.len(), 11);

        info!("testing entity filter");
        let filter = RequestFilter {
            entity: Some(201),
            ..Default::default()
        };
        let (store_locations, count) = get_store_locations(&db_connection, filter, 2).unwrap();

        assert_eq!(count, 2);
        for store_location in store_locations.iter() {
            assert!(
                (store_location
                    .store_location_name
                    .eq("FAKE_STORELOCATION_21")
                    || store_location
                        .store_location_name
                        .eq("FAKE_STORELOCATION_22"))
            )
        }

        info!("testing store location name filter");
        let filter = RequestFilter {
            search: Some(String::from("FAKE_STORELOCATION_22")),
            ..Default::default()
        };
        let (store_locations, count) = get_store_locations(&db_connection, filter, 2).unwrap();
        assert_eq!(count, 1);
        assert_eq!(
            store_locations[0].store_location_name,
            "FAKE_STORELOCATION_22"
        );

        info!("testing store location can store filter");
        let filter = RequestFilter {
            store_location_can_store: true,
            ..Default::default()
        };
        let (store_locations, count) = get_store_locations(&db_connection, filter, 2).unwrap();
        assert_eq!(count, 1);
        assert_eq!(
            store_locations[0].store_location_name,
            "FAKE_STORELOCATION_22"
        );

        info!("testing limit");
        let filter = RequestFilter {
            limit: Some(5),
            ..Default::default()
        };
        let (store_locations, count) = get_store_locations(&db_connection, filter, 2).unwrap();
        assert_eq!(count, 11);
        assert_eq!(store_locations.len(), 5);

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
        let (_, count) = get_store_locations(&db_connection, filter, 2).unwrap();
        assert_eq!(count, 9);
    }
}
