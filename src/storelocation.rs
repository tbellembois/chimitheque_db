use crate::{entity::Entity, permission::Permission, reference::Reference, storage::Storage};
use chimitheque_types::sortorder::SortOrder;
use chimitheque_types::{
    entity::Entity as EntityStruct, requestfilter::RequestFilter,
    storelocation::StoreLocation as StoreLocationStruct,
};
use chimitheque_utils::string::{Transform, clean_required_text};
use log::debug;
use rusqlite::{Connection, Row, Transaction};
use sea_query::{
    Alias, Expr, ExprTrait, Iden, JoinType, Order, Query, SimpleExpr, SqliteQueryBuilder,
};
use sea_query_rusqlite::RusqliteBinder;
use serde::Serialize;
use std::collections::HashSet;

/// Table and column names of the `store_location` table, for `sea_query`.
#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum StoreLocation {
    /// The table.
    Table,
    /// The `store_location_id` column.
    StoreLocationId,
    /// The `store_location_name` column.
    StoreLocationName,
    /// The `store_location_can_store` column.
    StoreLocationCanStore,
    /// The `store_location_color` column.
    StoreLocationColor,
    /// The `store_location_full_path` column.
    StoreLocationFullPath,
    /// The `entity` column.
    Entity,
    /// The `store_location` column.
    StoreLocation,
}

/// A store location read from a database row.
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

/// Returns the store locations matching the filter that the person can see,
/// and their total count without limit and offset.
pub fn get_store_locations(
    db_connection: &Connection,
    filter: &RequestFilter,
    person_id: u64,
) -> Result<(Vec<StoreLocationStruct>, usize), Box<dyn std::error::Error + Send + Sync>> {
    debug!("filter:{filter:?}");
    debug!("person_id:{person_id:?}");

    let order = match filter.order {
        SortOrder::Asc => Order::Asc,
        SortOrder::Desc => Order::Desc,
    };

    // Subquery for permissions to reduce rows early.
    let permission_subquery = Query::select()
        .expr(Expr::col((
            StoreLocation::Table,
            StoreLocation::StoreLocationId,
        )))
        .from(StoreLocation::Table)
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
                        .equals((StoreLocation::Table, StoreLocation::Entity))
                        .or(
                            Expr::col((Alias::new("perm"), Alias::new("permission_entity")))
                                .is_null(),
                        ),
                ),
        )
        .to_owned();

    // Create common query statement.
    let mut binding = Query::select();
    let mut expression = binding
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
        // storages for nb_storage
        .join(
            JoinType::LeftJoin,
            Storage::Table,
            Expr::col((StoreLocation::Table, StoreLocation::StoreLocationId))
                .equals((Storage::Table, Storage::StoreLocation))
                .and(Expr::col((Storage::Table, Storage::Storage)).is_null()),
        )
        // store locations for nb_children
        .join_as(
            JoinType::LeftJoin,
            StoreLocation::Table,
            Alias::new("children"),
            Expr::col((StoreLocation::Table, StoreLocation::StoreLocationId))
                .equals((Alias::new("children"), Alias::new("store_location"))),
        )
        // Apply permission subquery as a filter.
        .and_where(
            Expr::col((StoreLocation::Table, StoreLocation::StoreLocationId))
                .in_subquery(permission_subquery),
        );

    // Apply filters.
    if let Some(ref search) = filter.search {
        expression = expression.and_where(
            Expr::col((StoreLocation::Table, StoreLocation::StoreLocationName))
                .like(format!("%{search}%"))
                .or(Expr::col((Entity::Table, Entity::EntityName)).like(format!("%{search}%"))),
        );
    }
    if let Some(id) = filter.id {
        expression = expression
            .and_where(Expr::col((StoreLocation::Table, StoreLocation::StoreLocationId)).eq(id));
    }
    if let Some(entity) = filter.entity {
        expression = expression.and_where(Expr::col((Entity::Table, Entity::EntityId)).eq(entity));
    }
    if let Some(store_location) = filter.store_location {
        expression = expression.and_where(
            Expr::col((StoreLocation::Table, StoreLocation::StoreLocation)).eq(store_location),
        );
    }
    if filter.store_location_can_store {
        expression = expression.and_where(
            Expr::col((StoreLocation::Table, StoreLocation::StoreLocationCanStore))
                .eq(filter.store_location_can_store),
        );
    }

    // Create count query.
    let (count_sql, count_values) = expression
        .clone()
        .expr(Expr::col((StoreLocation::Table, StoreLocation::StoreLocationId)).count_distinct())
        .build_rusqlite(SqliteQueryBuilder);

    debug!("count_sql: {}", count_sql.clone().as_str());
    debug!("count_values: {count_values:?}");

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
        .group_by_col((StoreLocation::Table, StoreLocation::StoreLocationId))
        // Apply sorting.
        .order_by_expr(
            Expr::cust_with_expr(
                "? COLLATE NOCASE",
                Expr::col((StoreLocation::Table, StoreLocation::StoreLocationName)),
            ),
            order,
        )
        // Apply pagination.
        .conditions(
            filter.limit.is_some(),
            |q| {
                q.limit(filter.limit.unwrap() as u64);
            },
            |_| {},
        )
        .conditions(
            filter.offset.is_some(),
            |q| {
                q.offset(filter.offset.unwrap() as u64);
            },
            |_| {},
        )
        .build_rusqlite(SqliteQueryBuilder);

    debug!("select_sql: {}", select_sql.clone().as_str());
    debug!("select_values: {select_values:?}");

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
    let mut store_locations = Vec::with_capacity(filter.limit.unwrap_or(100));
    for maybe_store_location in rows {
        let store_location = maybe_store_location?;
        store_locations.push(store_location.0);
    }

    debug!("store_locations: {store_locations:#?}");

    Ok((store_locations, count))
}

fn populate_store_location_full_path(
    db_connection: &Connection,
    store_location: &mut StoreLocationStruct,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // If the store location has no parent, set full path to just the name
    if store_location.store_location.is_none() {
        store_location.store_location_full_path = Some(store_location.store_location_name.clone());
        return Ok(());
    }

    // Build the full path by querying each parent level
    let mut path_parts = Vec::new();

    // Start traversing from the parent (don't include the leaf node's name here)
    let mut current_id = store_location
        .store_location
        .as_ref()
        .map(|p| p.store_location_id)
        .ok_or("store location parent is None")?;

    // Traverse up the parent chain by querying the database
    loop {
        // Get the parent's name
        let (sql, params) = Query::select()
            .column(StoreLocation::StoreLocationName)
            .from(StoreLocation::Table)
            .and_where(Expr::col(StoreLocation::StoreLocationId).eq(current_id))
            .build_rusqlite(SqliteQueryBuilder);

        let parent_name: String =
            db_connection.query_row(&sql, &*params.as_params(), |row| row.get::<_, String>(0))?;

        path_parts.push(parent_name);

        // Get the parent's parent ID
        let (parent_id_sql, parent_id_params) = Query::select()
            .column(StoreLocation::StoreLocation)
            .from(StoreLocation::Table)
            .and_where(Expr::col(StoreLocation::StoreLocationId).eq(current_id))
            .build_rusqlite(SqliteQueryBuilder);

        if let Some(parent_id) = db_connection.query_row::<Option<u64>, _, _>(
            &parent_id_sql,
            &*parent_id_params.as_params(),
            |row| row.get(0),
        )? {
            current_id = Some(parent_id);
            continue;
        }

        break;
    }

    // Reverse to get parent -> child order and join with slashes
    path_parts.reverse();
    let full_path = path_parts.join("/");

    // Add the leaf node's name to complete the path
    let full_path_with_leaf = format!("{}/{}", full_path, store_location.store_location_name);

    store_location.store_location_full_path = Some(full_path_with_leaf);

    Ok(())
}

/// Rebuilds the full paths of the descendants of the store location, under its
/// `full_path`.
fn update_descendant_full_paths(
    db_transaction: &Transaction,
    store_location_id: u64,
    full_path: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut parents = vec![(store_location_id, full_path.to_string())];
    // Guards against a loop in the tree.
    let mut visited = HashSet::from([store_location_id]);

    while let Some((parent_id, parent_full_path)) = parents.pop() {
        let mut stmt = db_transaction.prepare(
            "SELECT store_location_id, store_location_name FROM store_location WHERE store_location = ?1",
        )?;
        let children = stmt
            .query_map([parent_id], |row| {
                Ok((row.get::<_, u64>(0)?, row.get::<_, String>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        for (child_id, child_name) in children {
            if !visited.insert(child_id) {
                continue;
            }
            let child_full_path = format!("{parent_full_path}/{child_name}");
            db_transaction.execute(
                "UPDATE store_location SET store_location_full_path = ?1 WHERE store_location_id = ?2",
                (&child_full_path, child_id),
            )?;
            parents.push((child_id, child_full_path));
        }
    }

    Ok(())
}

/// Creates the store location, or updates it if it has an id, with its full
/// path. Returns its id.
pub fn create_update_store_location(
    db_connection: &mut Connection,
    mut store_location: StoreLocationStruct,
) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    debug!("create_update_store_location: {store_location:#?}");

    let db_transaction = db_connection.transaction()?;

    // Setting up the full path.
    populate_store_location_full_path(&db_transaction, &mut store_location)?;

    // Rejected here rather than by the database.
    let clean_store_location_name =
        clean_required_text(&store_location.store_location_name, Transform::None)
            .ok_or("store_location_name is empty")?;
    let full_path = store_location.store_location_full_path.clone();

    // Update request: list of (columns, values) pairs to insert.
    let mut columns_values = vec![
        (
            StoreLocation::StoreLocationName,
            clean_store_location_name.clone().into(),
        ),
        (
            StoreLocation::StoreLocationCanStore,
            store_location.store_location_can_store.into(),
        ),
    ];

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
        columns_values.push((StoreLocation::StoreLocationColor, color.clone().into()));

        columns.push(StoreLocation::StoreLocationColor);
        values.push(SimpleExpr::Value(color.into()));
    } else {
        columns_values.push((StoreLocation::StoreLocationColor, Expr::cust("NULL")));

        columns.push(StoreLocation::StoreLocationColor);
        values.push(Expr::cust("NULL"));
    }

    if let Some(full_path) = store_location.store_location_full_path {
        columns_values.push((
            StoreLocation::StoreLocationFullPath,
            full_path.clone().into(),
        ));

        columns.push(StoreLocation::StoreLocationFullPath);
        values.push(SimpleExpr::Value(full_path.into()));
    }

    if let Some(entity) = store_location.entity {
        columns_values.push((StoreLocation::Entity, entity.entity_id.into()));

        columns.push(StoreLocation::Entity);
        values.push(SimpleExpr::Value(entity.entity_id.into()));
    }

    if let Some(store_location) = store_location.store_location {
        columns_values.push((
            StoreLocation::StoreLocation,
            store_location.store_location_id.into(),
        ));

        columns.push(StoreLocation::StoreLocation);
        values.push(SimpleExpr::Value(store_location.store_location_id.into()));
    } else {
        columns_values.push((StoreLocation::StoreLocation, Expr::cust("NULL")));

        columns.push(StoreLocation::StoreLocation);
        values.push(Expr::cust("NULL"));
    }

    let (sql_query, sql_values) = if let Some(id) = store_location.store_location_id {
        Query::update()
            .table(StoreLocation::Table)
            .values(columns_values)
            .and_where(Expr::col(StoreLocation::StoreLocationId).eq(id))
            .build_rusqlite(SqliteQueryBuilder)
    } else {
        Query::insert()
            .into_table(StoreLocation::Table)
            .columns(columns)
            .values(values)?
            .build_rusqlite(SqliteQueryBuilder)
    };

    debug!("sql_query: {}", sql_query.clone().as_str());
    debug!("sql_values: {sql_values:?}");

    _ = db_transaction.execute(&sql_query, &*sql_values.as_params())?;

    let last_insert_update_id: u64;

    if let Some(store_location_id) = store_location.store_location_id {
        last_insert_update_id = store_location_id;

        // A renamed or moved store location: its descendants' paths change too.
        if let Some(full_path) = &full_path {
            update_descendant_full_paths(&db_transaction, store_location_id, full_path)?;
        }
    } else {
        last_insert_update_id = db_transaction.last_insert_rowid().try_into()?;
    }

    debug!("last_insert_update_id: {last_insert_update_id}");

    db_transaction.commit()?;

    Ok(last_insert_update_id)
}

/// Deletes the store location.
pub fn delete_store_location(
    db_connection: &Connection,
    store_location_id: u64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!("store_location_id: {store_location_id}");

    // History rows keep the store location of their version: a storage moved
    // to another store location leaves history rows here. They would block the
    // deletion (foreign key), so they are deleted first. The current storages
    // are checked by the handler: a store location with storages can not be
    // deleted.
    let (sql_query, sql_values) = Query::delete()
        .from_table(Storage::Table)
        .and_where(Expr::col(Storage::StoreLocation).eq(store_location_id))
        .and_where(Expr::col(Storage::Storage).is_not_null())
        .build_rusqlite(SqliteQueryBuilder);

    debug!("sql_query: {}", sql_query.clone().as_str());
    debug!("sql_values: {sql_values:?}");

    _ = db_connection.execute(sql_query.as_str(), &*sql_values.as_params())?;

    let (sql_query, sql_values) = Query::delete()
        .from_table(StoreLocation::Table)
        .and_where(Expr::col(StoreLocation::StoreLocationId).eq(store_location_id))
        .build_rusqlite(SqliteQueryBuilder);

    debug!("sql_query: {}", sql_query.clone().as_str());
    debug!("sql_values: {sql_values:?}");

    _ = db_connection.execute(sql_query.as_str(), &*sql_values.as_params())?;

    Ok(())
}

/// Returns whether the store location has storages, archived ones included.
/// History storages do not count: they are deleted with the store location.
/// A store location with storages can not be deleted.
///
/// # Errors
///
/// Returns an error if the query fails.
pub fn store_location_has_storages(
    db_connection: &Connection,
    store_location_id: u64,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    Ok(db_connection.query_row(
        "SELECT EXISTS (SELECT 1 FROM storage WHERE store_location = ?1 AND storage IS NULL)",
        [store_location_id],
        |row| row.get(0),
    )?)
}

/// Returns the entity id of the store location, or `None` for an unknown store
/// location.
///
/// # Errors
///
/// Returns an error if the query fails.
pub fn store_location_entity_id(
    db_connection: &Connection,
    store_location_id: u64,
) -> Result<Option<u64>, Box<dyn std::error::Error + Send + Sync>> {
    match db_connection.query_row(
        "SELECT entity FROM store_location WHERE store_location_id = ?1",
        [store_location_id],
        |row| row.get(0),
    ) {
        Ok(entity_id) => Ok(Some(entity_id)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(err) => Err(Box::new(err)),
    }
}

/// Returns whether the store location has child store locations. A store
/// location with children can not be deleted.
///
/// # Errors
///
/// Returns an error if the query fails.
pub fn store_location_has_children(
    db_connection: &Connection,
    store_location_id: u64,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    Ok(db_connection.query_row(
        "SELECT EXISTS (SELECT 1 FROM store_location WHERE store_location = ?1)",
        [store_location_id],
        |row| row.get(0),
    )?)
}

/// Lists the ids a store location save writes, for
/// [`find_missing_reference`](crate::reference::find_missing_reference).
///
/// A new store location must have an entity; an updated one keeps its entity
/// when none is given. A given entity or parent store location must have an
/// id.
#[must_use]
pub fn store_location_references(store_location: &StoreLocationStruct) -> Vec<Reference> {
    let mut references = vec![];

    if store_location.store_location_id.is_some() {
        references.push(Reference::new(
            "store_location",
            "store_location",
            store_location.store_location_id,
        ));
    }
    match &store_location.entity {
        Some(entity) => references.push(Reference::new("entity", "entity", entity.entity_id)),
        None if store_location.store_location_id.is_none() => {
            references.push(Reference::new("entity", "entity", None));
        }
        None => {}
    }
    if let Some(parent) = &store_location.store_location {
        references.push(Reference::new(
            "store_location",
            "store_location",
            parent.store_location_id,
        ));
    }

    references
}

/// Returns whether `store_location_id` is `parent_id` or one of its
/// ancestors: setting `parent_id` as its parent would then make a loop.
///
/// # Errors
///
/// Returns an error if the query fails.
pub fn store_location_is_ancestor_of(
    db_connection: &Connection,
    store_location_id: u64,
    parent_id: u64,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    // Walk up from the parent. UNION (not UNION ALL) stops on an existing loop.
    Ok(db_connection.query_row(
        "WITH RECURSIVE ancestor(id) AS (
            SELECT ?2
            UNION
            SELECT store_location.store_location FROM store_location
            JOIN ancestor ON store_location.store_location_id = ancestor.id
            WHERE store_location.store_location IS NOT NULL
        )
        SELECT EXISTS (SELECT 1 FROM ancestor WHERE id = ?1)",
        [store_location_id, parent_id],
        |row| row.get(0),
    )?)
}

#[cfg(test)]
#[path = "storelocation_tests.rs"]
mod storelocation_tests;
