use std::fmt::Write;

use crate::{entity::Entity, permission::Permission, storage::Storage};
use chimitheque_types::{
    entity::Entity as EntityStruct, requestfilter::RequestFilter,
    storelocation::StoreLocation as StoreLocationStruct,
};
use chimitheque_utils::string::{clean, Transform};
use log::debug;
use rusqlite::{Connection, Row};
use sea_query::{
    Alias, ColumnRef, CommonTableExpression, Cycle, Expr, Func, Iden, IntoColumnRef, IntoIden,
    JoinType, Order, Query, SelectStatement, SimpleExpr, SqliteQueryBuilder, UnionType, WithClause,
};
use sea_query_rusqlite::{RusqliteBinder, RusqliteValues};
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
                store_location_color: row.get("store_location_color").unwrap_or_default(),
                store_location_full_path: row.get("store_location_full_path").unwrap_or_default(),
                entity: Some(EntityStruct {
                    entity_id: row.get_unwrap("entity_id"),
                    entity_name: row.get_unwrap("entity_name"),
                    entity_description: row.get_unwrap("entity_description"),
                    managers: None,
                    entity_nb_store_locations: None,
                    entity_nb_people: None,
                }),
                store_location: maybe_parent_store_location.map(|_| {
                    Box::new(StoreLocationStruct {
                        store_location_id: row.get_unwrap("parent_store_location_id"),
                        store_location_name: row.get_unwrap("parent_store_location_name"),
                        store_location_can_store: row.get_unwrap("parent_store_location_can_store"),
                        store_location_color: row
                            .get("parent_store_location_color")
                            .unwrap_or_default(),
                        store_location_full_path: row
                            .get("parent_store_location_full_path")
                            .unwrap_or_default(),
                        entity: None,
                        store_location: None,
                        store_location_nb_storages: None,
                        store_location_nb_children: None,
                    })
                }),
                store_location_nb_storages: row
                    .get("store_location_nb_storages")
                    .unwrap_or_default(),

                store_location_nb_children: row
                    .get("store_location_nb_children")
                    .unwrap_or_default(),
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
            "store_location.store_location_name" => {
                Alias::new("parent_store_location_name").into_column_ref()
            }
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
        // storages for nb_storage
        //
        .join(
            JoinType::LeftJoin,
            Storage::Table,
            Expr::col((StoreLocation::Table, StoreLocation::StoreLocationId))
                .equals((Storage::Table, Storage::StoreLocation)),
        )
        //
        // store locations for nb_children
        //
        .join_as(
            JoinType::LeftJoin,
            StoreLocation::Table,
            Alias::new("children"),
            Expr::col((StoreLocation::Table, StoreLocation::StoreLocationId))
                .equals((Alias::new("children"), Alias::new("store_location"))),
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
                    Expr::col((Alias::new("perm"), Alias::new("permission_item")))
                        .is_in(["all", "storages"]),
                )
                .and(
                    Expr::col((Alias::new("perm"), Alias::new("permission_name")))
                        .is_in(["r", "w", "all"]),
                )
                .and(
                    Expr::col((Alias::new("perm"), Alias::new("permission_entity")))
                        .equals(Entity::EntityId)
                        .or(
                            Expr::col((Alias::new("perm"), Alias::new("permission_entity"))).eq(-1),
                        ),
                ),
        )
        //
        // filters
        //
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
            filter.id.is_some(),
            |q| {
                q.and_where(
                    Expr::col((StoreLocation::Table, StoreLocation::StoreLocationId))
                        .eq(filter.id.unwrap()),
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
            filter.store_location.is_some(),
            |q| {
                q.and_where(
                    Expr::col((StoreLocation::Table, StoreLocation::StoreLocation))
                        .eq(filter.store_location.unwrap()),
                );
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
        .columns([
            Entity::EntityId,
            Entity::EntityName,
            Entity::EntityDescription,
        ])
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
        .expr_as(
            Expr::col(Storage::StorageId).count_distinct(),
            Alias::new("store_location_nb_storages"),
        )
        .expr_as(
            Expr::col((Alias::new("children"), Alias::new("store_location_id"))).count_distinct(),
            Alias::new("store_location_nb_children"),
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
        Ok(StoreLocationWrapper::from(row))
    })?;

    // Build select result.
    let mut store_locations = Vec::new();
    for maybe_store_location in rows {
        let store_location = maybe_store_location?;

        store_locations.push(store_location.0);
    }

    debug!("store_locations: {:#?}", store_locations);

    Ok((store_locations, count))
}

fn populate_store_location_full_path(
    db_connection: &Connection,
    store_location: &mut StoreLocationStruct,
) -> Result<(), Box<dyn std::error::Error>> {
    // If the store location has no parent setting its name as full path.
    if store_location.store_location.is_none() {
        store_location.store_location_full_path = Some(store_location.store_location_name.clone());
        return Ok(());
    }

    let parent_id = store_location
        .store_location
        .as_ref()
        .unwrap()
        .store_location_id;

    /*    WITH ancestor AS (SELECT store_location as p, store_location_name as n FROM store_location WHERE store_location_id = ????
     *    UNION ALL
     *    SELECT store_location, store_location_name FROM ancestor, store_location
     *    WHERE ancestor.p = store_location.store_location_id)
     *    SELECT group_concat(n, '/') AS store_location_full_path FROM ancestor */

    struct MyGroupConcatFunction;

    impl Iden for MyGroupConcatFunction {
        fn unquoted(&self, s: &mut dyn Write) {
            write!(s, "group_concat").unwrap();
        }
    }

    let base_query = SelectStatement::new()
        .expr_as(Expr::col(StoreLocation::StoreLocation), Alias::new("p"))
        .expr_as(Expr::col(StoreLocation::StoreLocationName), Alias::new("n"))
        .from(StoreLocation::Table)
        .and_where(Expr::col(StoreLocation::StoreLocationId).eq(parent_id))
        .and_where(
            Expr::col(StoreLocation::StoreLocationId)
                .equals((StoreLocation::Table, StoreLocation::StoreLocationId)),
        )
        .to_owned();

    let cte_referencing = SelectStatement::new()
        .columns([
            StoreLocation::StoreLocation,
            StoreLocation::StoreLocationName,
        ])
        .from(StoreLocation::Table)
        .from(Alias::new("ancestor"))
        .and_where(
            Expr::col((Alias::new("ancestor"), Alias::new("p")))
                .equals((StoreLocation::Table, StoreLocation::StoreLocationId)),
        )
        .to_owned();

    let common_table_expression = CommonTableExpression::new()
        .query(
            base_query
                .clone()
                .union(UnionType::All, cte_referencing)
                .to_owned(),
        )
        .table_name(Alias::new("ancestor"))
        .to_owned();

    let select = SelectStatement::new()
        .expr_as(
            Func::cust(MyGroupConcatFunction).arg(ColumnRef::Column(Alias::new("n").into_iden())),
            Alias::new("store_location_parents"),
        )
        .from(Alias::new("ancestor"))
        .to_owned();

    let with_clause = WithClause::new()
        .recursive(false)
        .cte(common_table_expression)
        .cycle(Cycle::new_from_expr_set_using(
            SimpleExpr::Column(ColumnRef::Column(Alias::new("id").into_iden())),
            Alias::new("looped"),
            Alias::new("traversal_path"),
        ))
        .to_owned();

    let (select_sql, select_values) = select.with(with_clause).build_rusqlite(SqliteQueryBuilder);

    debug!("select_sql: {}", select_sql.clone().as_str());
    debug!("select_values: {:?}", select_values);

    // Perform select query.
    let mut stmt = db_connection.prepare(select_sql.as_str())?;
    let mut rows = stmt.query(&*select_values.as_params())?;
    store_location.store_location_full_path = if let Some(row) = rows.next()? {
        let store_location_parents: String = row.get_unwrap("store_location_parents");

        let mut store_location_full_path = store_location.store_location_name.clone();
        store_location_full_path.push(',');
        store_location_full_path.push_str(&store_location_parents);

        // At this point store_location_full_path is like:
        // store_location_name,parent1,parent2,root
        // We should revert it and replace , by /.
        let store_location_full_path_reversed = store_location_full_path
            .split(',')
            .rev()
            .collect::<Vec<_>>()
            .join("/");

        Some(store_location_full_path_reversed)
    } else {
        // We should always have a result.
        // The case where the store location has no parent is handled at the begining of the function.
        unreachable!()
    };

    Ok(())
}

pub fn create_update_store_location(
    db_connection: &mut Connection,
    mut store_location: StoreLocationStruct,
) -> Result<u64, Box<dyn std::error::Error>> {
    debug!("create_update_store_location: {:#?}", store_location);

    let db_transaction = db_connection.transaction()?;

    // Setting up the full path.
    populate_store_location_full_path(&db_transaction, &mut store_location)?;

    let clean_store_location_name = clean(&store_location.store_location_name, Transform::None);

    // Update request: list of (columns, values) pairs to insert.
    // let mut columns_values = vec![
    //     (
    //         StoreLocation::StoreLocationName,
    //         clean_store_location_name.clone().into(),
    //     ),
    //     (
    //         StoreLocation::StoreLocationCanStore,
    //         store_location.store_location_can_store.into(),
    //     ),
    // ];

    // Create request: list of columns and values to insert.
    let mut columns = vec![
        StoreLocation::StoreLocationName,
        StoreLocation::StoreLocationCanStore,
    ];
    let mut values = vec![
        SimpleExpr::Value(clean_store_location_name.into()),
        SimpleExpr::Value(store_location.store_location_can_store.into()),
    ];

    if let Some(color) = store_location.store_location_color {
        // columns_values.push((StoreLocation::StoreLocationColor, color.clone().into()));

        columns.push(StoreLocation::StoreLocationColor);
        values.push(SimpleExpr::Value(color.into()));
    }

    if let Some(full_path) = store_location.store_location_full_path {
        // columns_values.push((
        //     StoreLocation::StoreLocationFullPath,
        //     full_path.clone().into(),
        // ));

        columns.push(StoreLocation::StoreLocationFullPath);
        values.push(SimpleExpr::Value(full_path.into()));
    }

    if let Some(entity) = store_location.entity {
        // columns_values.push((StoreLocation::Entity, entity.entity_id.into()));

        columns.push(StoreLocation::Entity);
        values.push(SimpleExpr::Value(entity.entity_id.into()));
    }

    if let Some(store_location) = store_location.store_location {
        // columns_values.push((
        //     StoreLocation::StoreLocation,
        //     store_location.store_location_id.into(),
        // ));

        columns.push(StoreLocation::StoreLocation);
        values.push(SimpleExpr::Value(store_location.store_location_id.into()));
    }

    let sql_query: String;
    let sql_values: RusqliteValues = RusqliteValues(vec![]);

    if let Some(store_location_id) = store_location.store_location_id {
        // Update query.
        // (sql_query, sql_values) = Query::update()
        //     .table(StoreLocation::Table)
        //     .values(columns_values)
        //     .and_where(Expr::col(StoreLocation::StoreLocationId).eq(store_location_id))
        //     .build_rusqlite(SqliteQueryBuilder);
        columns.push(StoreLocation::StoreLocationId);
        values.push(SimpleExpr::Value(store_location_id.into()));

        sql_query = Query::insert()
            .replace()
            .into_table(StoreLocation::Table)
            .columns(columns)
            .values(values)?
            .to_string(SqliteQueryBuilder);
    } else {
        // Insert query.
        sql_query = Query::insert()
            .into_table(StoreLocation::Table)
            .columns(columns)
            .values(values)?
            .to_string(SqliteQueryBuilder);
    }

    debug!("sql_query: {}", sql_query.clone().as_str());
    debug!("sql_values: {:?}", sql_values);

    _ = db_transaction.execute(&sql_query, &*sql_values.as_params())?;

    let last_insert_update_id: u64;

    if let Some(store_location_id) = store_location.store_location_id {
        last_insert_update_id = store_location_id;
    } else {
        last_insert_update_id = db_transaction.last_insert_rowid().try_into()?;
    }

    debug!("last_insert_update_id: {}", last_insert_update_id);

    db_transaction.commit()?;

    Ok(last_insert_update_id)
}

pub fn delete_store_location(
    db_connection: &Connection,
    store_location_id: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("store_location_id: {}", store_location_id);

    let (sql_query, sql_values) = Query::delete()
        .from_table(StoreLocation::Table)
        .and_where(Expr::col(StoreLocation::StoreLocationId).eq(store_location_id))
        .build_rusqlite(SqliteQueryBuilder);

    _ = db_connection.execute(sql_query.as_str(), &*sql_values.as_params());

    Ok(())
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
    fn test_get_store_locations() {
        init_logger();

        let db_connection = init_test_db();

        info!("testing total result");
        let filter = RequestFilter {
            ..Default::default()
        };
        let (store_locations, count) = get_store_locations(&db_connection, filter, 1).unwrap();
        assert_eq!(count, 5);
        assert_eq!(store_locations.len(), 5);

        info!("testing entity filter");
        let filter = RequestFilter {
            entity: Some(2),
            ..Default::default()
        };
        let (store_locations, count) = get_store_locations(&db_connection, filter, 1).unwrap();

        assert_eq!(count, 3);
        for store_location in store_locations.iter() {
            assert!(
                (store_location.store_location_name.eq("location_2a")
                    || store_location.store_location_name.eq("location_2b")
                    || store_location.store_location_name.eq("location_2bb"))
            )
        }

        info!("testing store location name filter");
        let filter = RequestFilter {
            search: Some(String::from("location_1a")),
            ..Default::default()
        };
        let (store_locations, count) = get_store_locations(&db_connection, filter, 1).unwrap();
        assert_eq!(count, 1);
        assert_eq!(store_locations[0].store_location_name, "location_1a");

        info!("testing store location can store filter");
        let filter = RequestFilter {
            store_location_can_store: false,
            ..Default::default()
        };
        let (store_locations, count) = get_store_locations(&db_connection, filter, 1).unwrap();
        assert_eq!(count, 5);
        assert_eq!(store_locations[0].store_location_name, "location_1a");

        info!("testing limit");
        let filter = RequestFilter {
            limit: Some(2),
            ..Default::default()
        };
        let (store_locations, count) = get_store_locations(&db_connection, filter, 1).unwrap();
        assert_eq!(count, 5);
        assert_eq!(store_locations.len(), 2);
    }
}
