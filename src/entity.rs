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

/// Populates the managers for each entity in the provided slice by fetching associated manager records
/// from the database and assigning them to the corresponding entities.
///
/// This function:
/// 1. Extracts all entity IDs from the input entities slice
/// 2. Performs a single join query to fetch all managers for those entities
/// 3. Groups the managers by their `entity_id`
/// 4. Assigns the grouped managers back to each entity in the slice
///
/// The function uses a `HashMap` to efficiently group managers by `entity_id`, avoiding the N+1 query problem
/// by fetching all required data in a single database query. It also filters out empty manager lists before
/// assignment to maintain consistency.
///
/// # Arguments
/// * `db_connection` - A reference to the SQLite database connection
/// * `entities` - A mutable slice of `EntityStruct` that will have their managers populated
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error + Send + Sync>>` - Returns Ok(()) on success,
///   or an error if any operation fails
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

/// Retrieves entities from the database based on the provided filter and permission context.
///
/// This function:
/// 1. Logs the filter and `person_id` for debugging purposes
/// 2. Determines the ordering direction (ascending or descending) based on the filter
/// 3. Builds a permission subquery to ensure users can only access entities for which they have
///    appropriate permissions (read/write/all permissions on entities or 'all' permission with
///    null entity context)
/// 4. Constructs a main query that:
///    - Joins with `StoreLocation` to count store locations per entity
///    - Joins with Personentities to count people per entity
///    - Applies the permission filter to limit results to authorized entities
///    - Applies any additional filters (search text, entity name, or specific ID)
/// 5. Executes a count query to determine the total number of matching entities
/// 6. Executes the main select query to fetch the entities with their counts
/// 7. Populates manager information for each entity
/// 8. Returns the entities along with the total count
///
/// The function uses COLLATE NOCASE for case-insensitive string sorting.
///
/// # Arguments
/// * `db_connection` - A reference to the SQLite database connection
/// * `filter` - A `RequestFilter` struct specifying search criteria, pagination, and ordering
/// * `person_id` - The ID of the person making the request, used for permission checking
///
/// # Returns
/// * `Result<(Vec<EntityStruct>, usize), Box<dyn std::error::Error + Send + Sync>>` - A tuple
///   containing:
///   - A vector of `EntityStruct` objects matching the filter criteria
///   - The total count of entities matching the filter (before pagination)
///
/// # Errors
/// Returns an error if any database operation fails or if permission checking encounters issues
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

/// Updates entity managers by synchronizing the database state with the entity's manager list.
///
/// This function implements a 'delete then recreate' pattern to ensure the database state
/// matches the provided entity's manager list exactly. It performs the following operations:
///
/// 1. Validates that the entity has an `entity_id` (returns `EntityError::MissingEntityId` if missing)
/// 2. Deletes all existing entity manager permissions for the entity (where `permission_item` = \"all\",
///    `permission_name` = \"all\", and `permission_entity` = `entity_id`)
/// 3. Deletes all existing entity-people associations for the entity
/// 4. If the entity has managers in the `EntityStruct`, creates new entity-people associations
///    and sets manager permissions for each manager using `set_person_manager()`
///
/// The function uses a database transaction to ensure all delete and insert operations
/// succeed or fail together, maintaining data consistency.
///
/// # Arguments
/// * `db_transaction` - A reference to an active database transaction
/// * `entity` - A reference to the `EntityStruct` containing the entity data and manager list
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error + Send + Sync>>` - Returns Ok(()) on success,
///   or an error if any operation fails
///
/// # Errors
/// Returns an error if:
/// - The entity is missing an `entity_id`
/// - A manager is missing a `person_id`
/// - Any database operation fails
/// - The transaction cannot be committed
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

/// Creates a new entity in the database or updates an existing entity.
///
/// This function handles both creation and update operations using a database transaction
/// for atomicity. For existing entities, it updates the name and description fields.
/// For new entities, it inserts a new record and returns the auto-generated ID.
///
/// The function performs the following operations:
/// 1. Starts a new database transaction
/// 2. Determines if the operation is an insert (no `entity_id`) or update (with `entity_id`)
/// 3. Builds and executes the appropriate SQL query (INSERT or UPDATE)
/// 4. Retrieves the last inserted ID if creating a new entity
/// 5. Calls `create_update_entity_managers()` to sync manager associations
/// 6. Commits the transaction if all operations succeed
///
/// # Arguments
/// * `db_connection` - A mutable reference to the SQLite database connection
/// * `entity` - A mutable `EntityStruct` that will be created or updated
///
/// # Returns
/// * `Result<u64, Box<dyn std::error::Error + Send + Sync>>` - Returns the entity ID
///   (either the existing one for updates or the newly created one for inserts)
///
/// # Errors
/// Returns an error if:
/// - The database transaction cannot be started
/// - The SQL execution fails
/// - The manager synchronization fails
/// - The transaction commit fails
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

/// Deletes an entity from the database by its ID.
///
/// This function removes an entity record from the database. It performs a simple
/// DELETE operation on the Entity table, targeting the record with the specified
/// `entity_id`.
///
/// # Arguments
/// * `db_connection` - A mutable reference to the SQLite database connection
/// * `entity_id` - The unique identifier of the entity to be deleted
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error + Send + Sync>>` - Returns Ok(()) on success,
///   or an error if the deletion fails
///
/// # Errors
/// Returns an error if:
/// - The DELETE SQL execution fails
/// - The database connection is invalid
///
/// # Notes
/// This function does not perform any cascade deletes or check for dependent records.
/// It assumes the caller has verified that deleting this entity is appropriate and
/// will not violate any data integrity constraints.
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
