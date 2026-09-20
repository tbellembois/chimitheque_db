use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
};

use chimitheque_types::{entity::Entity as EntityStruct, requestfilter::RequestFilter};
use log::debug;
use rusqlite::{Connection, Row, Transaction};
use sea_query::{
    Alias, ColumnRef, Expr, ExprTrait, Iden, IntoColumnRef, JoinType, Order, Query, SimpleExpr,
    SqliteQueryBuilder,
};

use sea_query_rusqlite::RusqliteBinder;
use serde::Serialize;

use crate::{
    entitypeople::Entitypeople,
    permission::Permission,
    person::{Person, set_person_manager},
    personentities::Personentities,
    storelocation::StoreLocation,
};

#[derive(Debug, PartialEq, Eq)]
pub enum EntityError {
    MissingEntityId,
    MissingPersonId,
}

impl Display for EntityError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            EntityError::MissingEntityId => write!(f, "missing entity id"),
            EntityError::MissingPersonId => write!(f, "missing person id"),
        }
    }
}

impl std::error::Error for EntityError {}

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Entity {
    Table,
    EntityId,
    EntityName,
    EntityDescription,
}

#[derive(Debug, Serialize)]
pub struct EntityWrapper(pub EntityStruct);

impl From<&Row<'_>> for EntityWrapper {
    fn from(row: &Row) -> Self {
        Self({
            EntityStruct {
                entity_id: row.get_unwrap("entity_id"),
                entity_name: row.get_unwrap("entity_name"),
                entity_description: row.get_unwrap("entity_description"),

                managers: None,
                entity_nb_store_locations: row.get("entity_nb_store_locations").unwrap_or_default(),
                entity_nb_people: row.get("entity_nb_people").unwrap_or_default(),
            }
        })
    }
}

fn populate_managers(
    db_connection: &Connection,
    entities: &mut [EntityStruct],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if entities.is_empty() {
        return Ok(());
    }

    let entity_ids: Vec<u64> = entities.iter().filter_map(|e| e.entity_id).collect();
    if entity_ids.is_empty() {
        return Ok(());
    }

    // Fetch all managers for all entities in one go to avoid N+1 query problem
    let (sql, values) = Query::select()
        .columns([
            Entitypeople::EntitypeopleEntityId,
            Entitypeople::EntitypeoplePersonId,
        ])
        .column(Person::PersonEmail)
        .from(Entitypeople::Table)
        .join(
            JoinType::LeftJoin,
            Person::Table,
            Expr::col((Entitypeople::Table, Entitypeople::EntitypeoplePersonId))
                .equals((Person::Table, Person::PersonId)),
        )
        .and_where(Expr::col(Entitypeople::EntitypeopleEntityId).is_in(entity_ids))
        .build_rusqlite(SqliteQueryBuilder);

    let mut stmt = db_connection.prepare(sql.as_str())?;
    let rows = stmt.query_map(&*values.as_params(), |row| {
        Ok((
            row.get::<_, u64>("entitypeople_entity_id")?,
            chimitheque_types::person::Person {
                person_id: Some(row.get("entitypeople_person_id")?),
                person_email: row.get("person_email")?,
                ..Default::default()
            },
        ))
    })?;

    // Group managers by entity_id
    let mut managers_map: HashMap<u64, Vec<chimitheque_types::person::Person>> = HashMap::new();
    for row in rows {
        let (e_id, person) = row?;
        managers_map.entry(e_id).or_default().push(person);
    }

    // Assign back to entities
    for entity in entities {
        if let Some(id) = entity.entity_id {
            entity.managers = managers_map.remove(&id).filter(|v| !v.is_empty());
        }
    }

    Ok(())
}

pub fn get_entities(
    db_connection: &Connection,
    filter: RequestFilter,
    person_id: u64,
) -> Result<(Vec<EntityStruct>, usize), Box<dyn std::error::Error + Send + Sync>> {
    debug!("filter:{filter:?}");
    debug!("person_id:{person_id:?}");

    let order_by: ColumnRef = (Entity::Table, Entity::EntityName).into_column_ref();

    let order = if filter.order.eq_ignore_ascii_case("desc") {
        Order::Desc
    } else {
        Order::Asc
    };

    // Subquery for permissions to reduce rows early.
    let permission_subquery = Query::select()
        .expr(Expr::col((Entity::Table, Entity::EntityId)))
        .from(Entity::Table)
        .join_as(
            JoinType::InnerJoin,
            Permission::Table,
            Alias::new("perm"),
            Expr::col((Alias::new("perm"), Alias::new("person")))
                .eq(person_id)
                .and(
                    Expr::col((Alias::new("perm"), Alias::new("permission_item")))
                        .is_in(["all", "entities"]),
                )
                .and(
                    Expr::col((Alias::new("perm"), Alias::new("permission_name")))
                        .is_in(["r", "w", "all"]),
                )
                .and(
                    Expr::col((Alias::new("perm"), Alias::new("permission_entity")))
                        .equals((Entity::Table, Entity::EntityId))
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
        .from(Entity::Table)
        // store locations for nb_store_locations
        .join(
            JoinType::LeftJoin,
            StoreLocation::Table,
            Expr::col((StoreLocation::Table, StoreLocation::Entity))
                .equals((Entity::Table, Entity::EntityId)),
        )
        // person for nb_people
        .join(
            JoinType::LeftJoin,
            Personentities::Table,
            Expr::col((
                Personentities::Table,
                Personentities::PersonentitiesEntityId,
            ))
            .equals((Entity::Table, Entity::EntityId)),
        )
        // Apply permission subquery as a filter.
        .and_where(Expr::col((Entity::Table, Entity::EntityId)).in_subquery(permission_subquery));

    // Apply filters.
    if let Some(ref search) = filter.search {
        expression = expression
            .and_where(Expr::col((Entity::Table, Entity::EntityName)).like(format!("%{search}%")));
    }
    if let Some(entity_name) = filter.entity_name {
        expression =
            expression.and_where(Expr::col((Entity::Table, Entity::EntityName)).eq(entity_name));
    }
    if let Some(id) = filter.id {
        expression = expression.and_where(Expr::col(Entity::EntityId).eq(id));
    }

    // Create count query.
    let (count_sql, count_values) = expression
        .clone()
        .expr(Expr::col((Entity::Table, Entity::EntityId)).count_distinct())
        .build_rusqlite(SqliteQueryBuilder);

    debug!("count_sql: {}", count_sql.as_str());
    debug!("count_values: {count_values:?}");

    // Create select query.
    let (select_sql, select_values) = expression
        .columns([
            Entity::EntityId,
            Entity::EntityName,
            Entity::EntityDescription,
        ])
        .expr_as(
            Expr::col(StoreLocation::StoreLocationId).count_distinct(),
            Alias::new("entity_nb_store_locations"),
        )
        .expr_as(
            Expr::col(Personentities::PersonentitiesPersonId).count_distinct(),
            Alias::new("entity_nb_people"),
        )
        .group_by_col((Entity::Table, Entity::EntityId))
        .order_by_expr(
            Expr::cust_with_expr("? COLLATE NOCASE", Expr::col(order_by)),
            order,
        )
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

    debug!("select_sql: {}", select_sql.as_str());
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
        Ok(EntityWrapper::from(row))
    })?;

    // Build select result.
    let mut entities = Vec::with_capacity(filter.limit.unwrap_or(100));
    for maybe_entity in rows {
        let entity = maybe_entity?;
        entities.push(entity.0);
    }

    populate_managers(db_connection, &mut entities)?;

    debug!("entities: {entities:#?}");

    Ok((entities, count))
}

fn create_update_entity_managers(
    db_transaction: &Transaction,
    entity: &EntityStruct,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!("create_update_entity_managers: {entity:#?}");

    debug!("entity: {entity:#?}");

    let Some(entity_id) = entity.entity_id else {
        return Err(Box::new(EntityError::MissingEntityId));
    };

    // Lazily remove all entity managers permissions.
    let (delete_sql, delete_values) = Query::delete()
        .from_table(Permission::Table)
        .and_where(Expr::col(Permission::PermissionItem).eq("all"))
        .and_where(Expr::col(Permission::PermissionName).eq("all"))
        .and_where(Expr::col(Permission::PermissionEntity).eq(entity_id))
        .build_rusqlite(SqliteQueryBuilder);

    debug!("delete_sql: {}", delete_sql.as_str());
    debug!("delete_values: {delete_values:?}");

    _ = db_transaction.execute(delete_sql.as_str(), &*delete_values.as_params())?;

    // Lazily remove all entity managers.
    let (delete_sql, delete_values) = Query::delete()
        .from_table(Entitypeople::Table)
        .and_where(Expr::col(Entitypeople::EntitypeopleEntityId).eq(entity_id))
        .build_rusqlite(SqliteQueryBuilder);

    debug!("delete_sql: {}", delete_sql.as_str());
    debug!("delete_values: {delete_values:?}");

    _ = db_transaction.execute(delete_sql.as_str(), &*delete_values.as_params())?;

    // Adding new ones.
    if let Some(managers) = &entity.managers {
        for manager in managers {
            let Some(person_id) = manager.person_id else {
                return Err(Box::new(EntityError::MissingPersonId));
            };

            set_person_manager(db_transaction, person_id, entity_id)?;
        }
    }

    Ok(())
}

pub fn create_update_entity(
    db_connection: &mut Connection,
    mut entity: EntityStruct,
) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    debug!("create_update_entity: {entity:#?}");

    let db_transaction = db_connection.transaction()?;

    let (sql_query, sql_values) = if let Some(entity_id) = entity.entity_id {
        // Update query: logic moved inside to avoid unnecessary clones
        Query::update()
            .table(Entity::Table)
            .values([
                (Entity::EntityName, entity.entity_name.clone().into()),
                (
                    Entity::EntityDescription,
                    entity.entity_description.clone().into(),
                ),
            ])
            .and_where(Expr::col(Entity::EntityId).eq(entity_id))
            .build_rusqlite(SqliteQueryBuilder)
    } else {
        // Insert query: logic moved inside to avoid unnecessary clones
        Query::insert()
            .into_table(Entity::Table)
            .columns([Entity::EntityName, Entity::EntityDescription])
            .values([
                SimpleExpr::Value(entity.entity_name.clone().into()),
                SimpleExpr::Value(entity.entity_description.clone().into()),
            ])?
            .build_rusqlite(SqliteQueryBuilder)
    };

    debug!("sql_query: {}", sql_query.as_str());
    debug!("sql_values: {sql_values:?}");

    _ = db_transaction.execute(&sql_query, &*sql_values.as_params())?;

    let last_insert_update_id: u64;

    if let Some(entity_id) = entity.entity_id {
        last_insert_update_id = entity_id;
    } else {
        last_insert_update_id = db_transaction.last_insert_rowid().try_into()?;
        entity.entity_id = Some(last_insert_update_id);
    }

    debug!("last_insert_update_id: {last_insert_update_id}");

    create_update_entity_managers(&db_transaction, &entity)?;

    db_transaction.commit()?;

    Ok(last_insert_update_id)
}

pub fn delete_entity(
    db_connection: &mut Connection,
    entity_id: u64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!("delete_entity: {entity_id:#?}");

    let (delete_sql, delete_values) = Query::delete()
        .from_table(Entity::Table)
        .and_where(Expr::col(Entity::EntityId).eq(entity_id))
        .build_rusqlite(SqliteQueryBuilder);

    _ = db_connection.execute(delete_sql.as_str(), &*delete_values.as_params())?;

    Ok(())
}

#[cfg(test)]
#[path = "entity_tests.rs"]
mod entity_tests;
