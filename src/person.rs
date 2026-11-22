use chimitheque_types::sortorder::SortOrder;
use chimitheque_types::{
    permission::{Permission as PermissionStruct, PermissionItem, PermissionName},
    person::Person as PersonStruct,
    requestfilter::RequestFilter,
};
use log::debug;
use rusqlite::{Connection, Row, Transaction};
use sea_query::{
    Alias, Expr, ExprTrait, Iden, JoinType, Order, Query, SimpleExpr, SqliteQueryBuilder, Value,
};
use sea_query_rusqlite::{RusqliteBinder, RusqliteValues};
use serde::Serialize;

use crate::{
    entity::Entity,
    entitypeople::{Entitypeople, EntitypeopleWrapper},
    permission::{Permission, PermissionWrapper},
    personentities::{Personentities, PersonentitiesWrapper},
    reference::Reference,
};

/// Table and column names of the `person` table, for `sea_query`.
#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Person {
    /// The table.
    Table,
    /// The `person_id` column.
    PersonId,
    /// The `person_email` column.
    PersonEmail,
}

/// A person read from a database row.
#[derive(Debug, Serialize)]
pub struct PersonWrapper(pub PersonStruct);

impl From<&Row<'_>> for PersonWrapper {
    fn from(row: &Row) -> Self {
        Self({
            PersonStruct {
                person_id: row.get_unwrap("person_id"),
                person_email: row.get_unwrap("person_email"),
                entities: None,
                managed_entities: None,
                permissions: None,
                is_admin: row.get("is_admin").unwrap_or_default(),
            }
        })
    }
}

fn populate_entities(
    db_connection: &Connection,
    people: &mut [PersonStruct],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for person in people.iter_mut() {
        let person_id = person.person_id;

        // Create select query.
        let (sql, values) = Query::select()
            .columns([
                Personentities::PersonentitiesPersonId,
                Personentities::PersonentitiesEntityId,
            ])
            .column(Entity::EntityName)
            // Field not needed but required to build a PersonentitiesWrapper from Row.
            .column(Person::PersonEmail)
            .from(Personentities::Table)
            //
            // entity
            //
            .join(
                JoinType::LeftJoin,
                Entity::Table,
                Expr::col((
                    Personentities::Table,
                    Personentities::PersonentitiesEntityId,
                ))
                .equals((Entity::Table, Entity::EntityId)),
            )
            //
            // person
            //
            .join(
                JoinType::LeftJoin,
                Person::Table,
                Expr::col((Person::Table, Person::PersonId)).eq(person_id),
            )
            .and_where(Expr::col(Personentities::PersonentitiesPersonId).eq(person_id))
            .build_rusqlite(SqliteQueryBuilder);

        debug!("sql: {}", sql.as_str());
        debug!("values: {values:?}");

        // Perform select query.
        let mut stmt = db_connection.prepare(sql.as_str())?;
        let rows = stmt.query_map(&*values.as_params(), |row| {
            Ok(PersonentitiesWrapper::from(row))
        })?;

        // Populate person entities.
        let mut entities: Vec<chimitheque_types::entity::Entity> = vec![];
        for row in rows {
            let person_entity_wrapper = row?;
            entities.push(chimitheque_types::entity::Entity {
                entity_id: Some(person_entity_wrapper.0.personentities_entity_id),
                entity_name: person_entity_wrapper.0.personentities_entity_name,
                entity_description: None,
                managers: None,
                entity_nb_store_locations: None,
                entity_nb_people: None,
            });
        }

        if entities.is_empty() {
            person.entities = None;
        } else {
            person.entities = Some(entities);
        }
    }

    Ok(())
}

fn populate_managed_entities(
    db_connection: &Connection,
    people: &mut [PersonStruct],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for person in people.iter_mut() {
        let person_id = person.person_id;

        // Create select query.
        let (sql, values) = Query::select()
            .columns([
                Entitypeople::EntitypeopleEntityId,
                Entitypeople::EntitypeoplePersonId,
            ])
            .column(Entity::EntityName)
            // Field not needed but required to build a PersonentitiesWrapper from Row.
            .column(Person::PersonEmail)
            .from(Entitypeople::Table)
            //
            // entity
            //
            .join(
                JoinType::LeftJoin,
                Entity::Table,
                Expr::col((Entitypeople::Table, Entitypeople::EntitypeopleEntityId))
                    .equals((Entity::Table, Entity::EntityId)),
            )
            //
            // person
            //
            .join(
                JoinType::LeftJoin,
                Person::Table,
                Expr::col((Person::Table, Person::PersonId)).eq(person_id),
            )
            .and_where(Expr::col(Entitypeople::EntitypeoplePersonId).eq(person_id))
            .build_rusqlite(SqliteQueryBuilder);

        debug!("sql: {}", sql.as_str());
        debug!("values: {values:?}");

        // Perform select query.
        let mut stmt = db_connection.prepare(sql.as_str())?;
        let rows = stmt.query_map(&*values.as_params(), |row| {
            Ok(EntitypeopleWrapper::from(row))
        })?;

        // Populate person managed entities.
        let mut entities: Vec<chimitheque_types::entity::Entity> = vec![];
        for row in rows {
            let entity_people_wrapper = row?;
            entities.push(chimitheque_types::entity::Entity {
                entity_id: Some(entity_people_wrapper.0.entitypeople_entity_id),
                entity_name: entity_people_wrapper.0.entitypeople_entity_name,
                entity_description: None,
                managers: None,
                entity_nb_store_locations: None,
                entity_nb_people: None,
            });
        }

        if entities.is_empty() {
            person.managed_entities = None;
        } else {
            person.managed_entities = Some(entities);
        }
    }

    Ok(())
}

fn populate_permissions(
    db_connection: &Connection,
    people: &mut [PersonStruct],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for person in people.iter_mut() {
        let person_id = person.person_id;

        // Create select query.
        let (sql, values) = Query::select()
            .columns([
                Permission::PermissionItem,
                Permission::PermissionName,
                Permission::PermissionEntity,
            ])
            .columns([Person::PersonId, Person::PersonEmail])
            .from(Permission::Table)
            //
            // person
            //
            .join(
                JoinType::LeftJoin,
                Person::Table,
                Expr::col((Person::Table, Person::PersonId)).eq(person_id),
            )
            .and_where(Expr::col(Permission::Person).eq(person_id))
            .build_rusqlite(SqliteQueryBuilder);

        debug!("sql: {}", sql.as_str());
        debug!("values: {values:?}");

        // Perform select query.
        let mut stmt = db_connection.prepare(sql.as_str())?;
        let mut rows = stmt.query(&*values.as_params())?;
        let mut permissions = Vec::new();
        while let Some(row) = rows.next()? {
            let permission = PermissionWrapper::try_from(row)?;
            permissions.push(permission.0);
        }

        if permissions.is_empty() {
            person.permissions = None;
        } else {
            person.permissions = Some(permissions);
        }
    }

    Ok(())
}

/// Returns the people matching the filter that the person `person_id` can see,
/// and their total count without limit and offset.
pub fn get_people(
    db_connection: &Connection,
    filter: &RequestFilter,
    person_id: u64,
) -> Result<(Vec<PersonStruct>, usize), Box<dyn std::error::Error + Send + Sync>> {
    debug!("filter:{filter:?}");
    debug!("person_id:{person_id:?}");

    let order = match filter.order {
        SortOrder::Asc => Order::Asc,
        SortOrder::Desc => Order::Desc,
    };

    // Subquery for people with not entity.
    let orphan_people_subquery = Query::select()
        .expr(Expr::col((Person::Table, Person::PersonId)))
        .from(Person::Table)
        .and_where(
            Expr::col((Person::Table, Person::PersonId)).not_in_subquery(
                Query::select()
                    .expr(Expr::col((
                        Personentities::Table,
                        Personentities::PersonentitiesPersonId,
                    )))
                    .from(Personentities::Table)
                    .take(),
            ),
        )
        .to_owned();

    // Subquery for permissions to reduce rows early.
    let permission_subquery = Query::select()
        .expr(Expr::col((Person::Table, Person::PersonId)))
        .from(Person::Table)
        .join(
            JoinType::LeftJoin,
            Personentities::Table,
            Expr::col((
                Personentities::Table,
                Personentities::PersonentitiesPersonId,
            ))
            .equals((Person::Table, Person::PersonId)),
        )
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
                        .equals(Personentities::PersonentitiesEntityId)
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
        .from(Person::Table)
        .join(
            JoinType::LeftJoin,
            Personentities::Table,
            Expr::col((
                Personentities::Table,
                Personentities::PersonentitiesPersonId,
            ))
            .equals((Person::Table, Person::PersonId)),
        )
        // Apply subqueries as a filter.
        .and_where(
            Expr::col((Person::Table, Person::PersonId))
                .in_subquery(permission_subquery)
                .or(Expr::col((Person::Table, Person::PersonId))
                    .in_subquery(orphan_people_subquery)),
        );

    // Apply filters.
    if let Some(entity) = filter.entity {
        expression = expression.and_where(
            Expr::col((
                Personentities::Table,
                Personentities::PersonentitiesEntityId,
            ))
            .eq(entity),
        );
    }
    if let Some(id) = filter.id {
        expression = expression.and_where(Expr::col((Person::Table, Person::PersonId)).eq(id));
    }
    if let Some(person_email) = &filter.person_email {
        expression =
            expression.and_where(Expr::col((Person::Table, Person::PersonEmail)).eq(person_email));
    }
    if let Some(search) = &filter.search {
        expression = expression
            .and_where(Expr::col((Person::Table, Person::PersonEmail)).like(format!("%{search}%")));
    }

    // Create count query
    let (count_sql, count_values) = expression
        .clone()
        .expr(Expr::col((Person::Table, Person::PersonId)).count_distinct())
        .build_rusqlite(SqliteQueryBuilder);

    debug!("count_sql: {}", count_sql.as_str());
    debug!("count_values: {count_values:?}");

    // Create select query
    let (select_sql, select_values) = expression
        .columns([Person::PersonId, Person::PersonEmail])
        .expr_as(
            Expr::case(
                Expr::exists(
                    Query::select()
                        .expr(Expr::col((Permission::Table, Permission::PermissionItem)))
                        .from(Permission::Table)
                        .and_where(
                            Expr::col((Permission::Table, Permission::PermissionItem)).eq("all"),
                        )
                        .and_where(
                            Expr::col((Permission::Table, Permission::PermissionName)).eq("all"),
                        )
                        .and_where(
                            Expr::col((Permission::Table, Permission::PermissionEntity)).is_null(),
                        )
                        .and_where(
                            Expr::col((Permission::Table, Permission::Person))
                                .equals((Person::Table, Person::PersonId)),
                        )
                        .take(),
                ),
                Expr::val(true),
            )
            .finally(Expr::val(false)),
            Alias::new("is_admin"),
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
        .group_by_col((Person::Table, Person::PersonId))
        .order_by(Person::PersonEmail, order)
        .build_rusqlite(SqliteQueryBuilder);

    debug!("select_sql: {}", select_sql.as_str());
    debug!("select_values: {select_values:?}");

    // Perform count query
    let mut stmt = db_connection.prepare(count_sql.as_str())?;
    let mut rows = stmt.query(&*count_values.as_params())?;
    let count: usize = if let Some(row) = rows.next()? {
        row.get_unwrap(0)
    } else {
        0
    };

    // Perform select query
    let mut stmt = db_connection.prepare(select_sql.as_str())?;
    let rows = stmt.query_map(&*select_values.as_params(), |row| {
        Ok(PersonWrapper::from(row))
    })?;

    // Build select result
    let mut people = Vec::with_capacity(filter.limit.unwrap_or(100));
    for maybe_person in rows {
        let person = maybe_person?;
        people.push(person.0);
    }

    populate_entities(db_connection, &mut people)?;
    populate_managed_entities(db_connection, &mut people)?;
    populate_permissions(db_connection, &mut people)?;

    debug!("people: {people:#?}");

    Ok((people, count))
}

fn create_update_person_permissions(
    db_transaction: &Transaction,
    person: &PersonStruct,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Lazily deleting former permissions.
    let (delete_sql, delete_values) = Query::delete()
        .from_table(Permission::Table)
        .and_where(Expr::col(Permission::Person).eq(person.person_id))
        .build_rusqlite(SqliteQueryBuilder);

    _ = db_transaction.execute(delete_sql.as_str(), &*delete_values.as_params())?;

    // Creating new permissions to append default and managed entities permissions.
    let mut new_permissions = if let Some(permissions) = &person.permissions {
        permissions.clone()
    } else {
        vec![]
    };

    // Appending default read products permissions.
    new_permissions.push(PermissionStruct {
        person: PersonStruct {
            person_id: person.person_id,
            ..Default::default()
        },
        permission_name: PermissionName::Read,
        permission_item: PermissionItem::Products,
        permission_entity: None,
    });

    // Appending permissions for the managed entities, as stored: managers are
    // set on the entities, not from the saved person.
    if let Some(person_id) = person.person_id {
        for entity_id in managed_entity_ids(db_transaction, person_id)? {
            new_permissions.push(PermissionStruct {
                person: PersonStruct {
                    person_id: Some(person_id),
                    ..Default::default()
                },
                permission_name: PermissionName::All,
                permission_item: PermissionItem::All,
                permission_entity: Some(entity_id),
            });
        }
    }

    // Inserting permissions.
    for perm in new_permissions {
        let mut values: Vec<SimpleExpr> = vec![];
        let columns = [
            Permission::Person,
            Permission::PermissionName,
            Permission::PermissionItem,
            Permission::PermissionEntity,
        ];

        values.push(person.person_id.into());
        values.push(perm.permission_name.to_string().into());
        values.push(perm.permission_item.to_string().into());
        values.push(perm.permission_entity.into());

        let (sql_query, sql_values) = Query::insert()
            .replace()
            .into_table(Permission::Table)
            .columns(columns)
            .values(values)?
            .build_rusqlite(SqliteQueryBuilder);

        debug!("sql_query: {}", sql_query.as_str());
        debug!("sql_values: {sql_values:?}");

        _ = db_transaction.execute(&sql_query, &*sql_values.as_params())?;
    }

    Ok(())
}

fn create_update_person_membership(
    db_transaction: &Transaction,
    person: &PersonStruct,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Lazily deleting former membership.
    let (delete_sql, delete_values) = Query::delete()
        .from_table(Personentities::Table)
        .and_where(Expr::col(Personentities::PersonentitiesPersonId).eq(person.person_id))
        .build_rusqlite(SqliteQueryBuilder);

    _ = db_transaction.execute(delete_sql.as_str(), &*delete_values.as_params())?;

    // Inserting new membership and the permission to read the entity..
    if let Some(entities) = &person.entities {
        for entity in entities {
            let mut values: Vec<SimpleExpr> = vec![];
            let columns = [
                Personentities::PersonentitiesPersonId,
                Personentities::PersonentitiesEntityId,
            ];

            values.push(person.person_id.into());
            values.push(entity.entity_id.into());

            let (sql_query, sql_values) = Query::insert()
                .into_table(Personentities::Table)
                .columns(columns)
                .values(values)?
                .build_rusqlite(SqliteQueryBuilder);

            debug!("sql_query: {}", sql_query.as_str());
            debug!("sql_values: {sql_values:?}");

            _ = db_transaction.execute(&sql_query, &*sql_values.as_params())?;

            // --

            let mut values: Vec<SimpleExpr> = vec![];
            let columns = [
                Permission::Person,
                Permission::PermissionName,
                Permission::PermissionItem,
                Permission::PermissionEntity,
            ];

            values.push(person.person_id.into());
            values.push(PermissionName::Read.to_string().into());
            values.push(PermissionItem::Entities.to_string().into());
            values.push(entity.entity_id.into());

            let (sql_query, sql_values) = Query::insert()
                .replace()
                .into_table(Permission::Table)
                .columns(columns)
                .values(values)?
                .build_rusqlite(SqliteQueryBuilder);

            debug!("sql_query: {}", sql_query.as_str());
            debug!("sql_values: {sql_values:?}");

            _ = db_transaction.execute(&sql_query, &*sql_values.as_params())?;
        }
    }
    Ok(())
}

/// Creates the person, or updates it if it has an id, with its permissions and
/// entities. Returns its id.
pub fn create_update_person(
    db_connection: &mut Connection,
    mut person: PersonStruct,
) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    debug!("create_update_person: {person:#?}");

    let db_transaction = db_connection.transaction()?;

    // Create request: list of columns and values to insert.
    let columns = vec![Person::PersonEmail];
    let values = vec![SimpleExpr::Value(person.person_email.clone().into())];

    // Update request: list of (columns, values) pairs to insert.
    let columns_values = vec![(
        Person::PersonEmail,
        SimpleExpr::Value(person.person_email.clone().into()),
    )];

    let sql_query: String;
    let sql_values: RusqliteValues;

    if let Some(person_id) = person.person_id {
        // Update query.
        (sql_query, sql_values) = Query::update()
            .table(Person::Table)
            .values(columns_values)
            .and_where(Expr::col(Person::PersonId).eq(person_id))
            .build_rusqlite(SqliteQueryBuilder);
    } else {
        // Insert query.
        (sql_query, sql_values) = Query::insert()
            .into_table(Person::Table)
            .columns(columns)
            .values(values)?
            .build_rusqlite(SqliteQueryBuilder);
    }

    debug!("sql_query: {}", sql_query.as_str());
    debug!("sql_values: {sql_values:?}");

    _ = db_transaction.execute(&sql_query, &*sql_values.as_params())?;

    let last_insert_update_id: u64;

    if let Some(person_id) = person.person_id {
        last_insert_update_id = person_id;
    } else {
        last_insert_update_id = db_transaction.last_insert_rowid().try_into()?;
        person.person_id = Some(last_insert_update_id);
    }

    debug!("last_insert_update_id: {last_insert_update_id}");

    create_update_person_permissions(&db_transaction, &person)?;
    create_update_person_membership(&db_transaction, &person)?;

    db_transaction.commit()?;

    Ok(last_insert_update_id)
}

/// Removes the person from the managers of the entity.
pub fn unset_person_manager(
    db_transaction: &Transaction,
    person_id: u64,
    entity_id: u64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!("unset_person_manager: {person_id:#?} {entity_id:#?}");

    // Unsetting the person manager.
    let (delete_sql, delete_values) = Query::delete()
        .from_table(Entitypeople::Table)
        .and_where(Expr::col(Entitypeople::EntitypeoplePersonId).eq(person_id))
        .and_where(Expr::col(Entitypeople::EntitypeopleEntityId).eq(entity_id))
        .build_rusqlite(SqliteQueryBuilder);

    _ = db_transaction.execute(delete_sql.as_str(), &*delete_values.as_params())?;

    // Removing the manager permissions.
    let (delete_sql, delete_values) = Query::delete()
        .from_table(Permission::Table)
        .and_where(Expr::col(Permission::Person).eq(person_id))
        .and_where(Expr::col(Permission::PermissionItem).eq("all"))
        .and_where(Expr::col(Permission::PermissionName).eq("all"))
        .and_where(Expr::col(Permission::PermissionEntity).eq(entity_id))
        .build_rusqlite(SqliteQueryBuilder);

    _ = db_transaction.execute(delete_sql.as_str(), &*delete_values.as_params())?;

    Ok(())
}

/// Makes the person a manager of the entity, and a member if not already.
pub fn set_person_manager(
    db_transaction: &Transaction,
    person_id: u64,
    entity_id: u64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!("set_person_manager: {person_id:#?} {entity_id:#?}");

    if is_person_manager(db_transaction, person_id, entity_id)? {
        return Ok(());
    }

    // Adding the manager to the entity.
    if !is_person_member_of(db_transaction, person_id, entity_id)? {
        let columns = vec![
            Personentities::PersonentitiesPersonId,
            Personentities::PersonentitiesEntityId,
        ];
        let values = vec![
            SimpleExpr::Value(person_id.into()),
            SimpleExpr::Value(entity_id.into()),
        ];

        let (sql_query, sql_values) = Query::insert()
            .into_table(Personentities::Table)
            .columns(columns)
            .values(values)?
            .build_rusqlite(SqliteQueryBuilder);

        debug!("sql_query: {}", sql_query.as_str());
        debug!("sql_values: {sql_values:?}");

        _ = db_transaction.execute(&sql_query, &*sql_values.as_params())?;
    }

    // Setting the person manager.
    let columns = vec![
        Entitypeople::EntitypeoplePersonId,
        Entitypeople::EntitypeopleEntityId,
    ];
    let values = vec![
        SimpleExpr::Value(person_id.into()),
        SimpleExpr::Value(entity_id.into()),
    ];

    let (sql_query, sql_values) = Query::insert()
        .into_table(Entitypeople::Table)
        .columns(columns)
        .values(values)?
        .build_rusqlite(SqliteQueryBuilder);

    debug!("sql_query: {}", sql_query.as_str());
    debug!("sql_values: {sql_values:?}");

    _ = db_transaction.execute(&sql_query, &*sql_values.as_params())?;

    // Setting the manager permissions.
    let columns = vec![
        Permission::PermissionItem,
        Permission::PermissionName,
        Permission::PermissionEntity,
        Permission::Person,
    ];
    let values = vec![
        SimpleExpr::Value("all".into()),
        SimpleExpr::Value("all".into()),
        SimpleExpr::Value(entity_id.into()),
        SimpleExpr::Value(person_id.into()),
    ];

    let (sql_query, sql_values) = Query::insert()
        .into_table(Permission::Table)
        .columns(columns)
        .values(values)?
        .build_rusqlite(SqliteQueryBuilder);

    debug!("sql_query: {}", sql_query.as_str());
    debug!("sql_values: {sql_values:?}");

    _ = db_transaction.execute(&sql_query, &*sql_values.as_params())?;

    Ok(())
}

/// Returns the administrators.
pub fn get_admins(
    db_connection: &Connection,
) -> Result<Vec<PersonStruct>, Box<dyn std::error::Error + Send + Sync>> {
    debug!("get_admins");
    // Create select query.
    let (select_sql, select_values) = Query::select()
        .from(Person::Table)
        .columns([Person::PersonId, Person::PersonEmail])
        .expr_as(true, Alias::new("is_admin"))
        .join_as(
            JoinType::InnerJoin,
            Permission::Table,
            Alias::new("perm"),
            Expr::col((Alias::new("perm"), Alias::new("person")))
                .equals((Person::Table, Person::PersonId))
                .and(Expr::col((Alias::new("perm"), Alias::new("permission_item"))).eq("all"))
                .and(Expr::col((Alias::new("perm"), Alias::new("permission_name"))).eq("all"))
                .and(
                    Expr::col((Alias::new("perm"), Alias::new("person")))
                        .equals((Person::Table, Person::PersonId)),
                )
                .and(Expr::col((Alias::new("perm"), Alias::new("permission_entity"))).is_null()),
        )
        .order_by(Person::PersonEmail, Order::Asc)
        .group_by_col((Person::Table, Person::PersonId))
        .build_rusqlite(SqliteQueryBuilder);

    debug!("select_sql: {}", select_sql.as_str());
    debug!("select_values: {select_values:?}");

    // Perform select query.
    let mut stmt = db_connection.prepare(select_sql.as_str())?;
    let rows = stmt.query_map(&*select_values.as_params(), |row| {
        Ok(PersonWrapper::from(row))
    })?;

    // Build select result.
    let mut people = Vec::new();
    for maybe_person in rows {
        let person = maybe_person?;

        people.push(person.0);
    }

    Ok(people)
}

/// Makes the person an administrator.
pub fn set_person_admin(
    db_connection: &mut Connection,
    person_id: u64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!("set_person_admin: {person_id:#?}");

    if is_person_admin(db_connection, person_id)? {
        return Ok(());
    }

    let columns = vec![
        Permission::PermissionItem,
        Permission::PermissionName,
        Permission::PermissionEntity,
        Permission::Person,
    ];
    let values = vec![
        SimpleExpr::Value("all".into()),
        SimpleExpr::Value("all".into()),
        SimpleExpr::Value(Value::String(None)),
        SimpleExpr::Value(person_id.into()),
    ];

    let (sql_query, sql_values) = Query::insert()
        .replace()
        .into_table(Permission::Table)
        .columns(columns)
        .values(values)?
        .build_rusqlite(SqliteQueryBuilder);

    debug!("sql_query: {}", sql_query.as_str());
    debug!("sql_values: {sql_values:?}");

    _ = db_connection.execute(&sql_query, &*sql_values.as_params())?;

    Ok(())
}

/// Removes the administrator permission of the person.
pub fn unset_person_admin(
    db_connection: &mut Connection,
    person_id: u64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!("unset_person_admin: {person_id:#?}");

    let (delete_sql, delete_values) = Query::delete()
        .from_table(Permission::Table)
        .and_where(Expr::col(Permission::Person).eq(person_id))
        .and_where(Expr::col(Permission::PermissionItem).eq("all"))
        .and_where(Expr::col(Permission::PermissionName).eq("all"))
        .and_where(Expr::col(Permission::PermissionEntity).is_null())
        .build_rusqlite(SqliteQueryBuilder);

    _ = db_connection.execute(delete_sql.as_str(), &*delete_values.as_params())?;

    Ok(())
}

/// Returns whether the person is an administrator.
pub fn is_person_admin(
    db_connection: &Connection,
    person_id: u64,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    debug!("is_admin: {person_id:#?}");

    // Create select query to check if person is admin
    let (sql, values) = Query::select()
        .expr(Expr::col(Permission::PermissionItem).count())
        .from(Permission::Table)
        .and_where(Expr::col(Permission::Person).eq(person_id))
        .and_where(Expr::col(Permission::PermissionItem).eq("all"))
        .and_where(Expr::col(Permission::PermissionName).eq("all"))
        .and_where(Expr::col(Permission::PermissionEntity).is_null())
        .build_rusqlite(SqliteQueryBuilder);

    debug!("sql: {}", sql.as_str());
    debug!("values: {values:?}");

    // Perform select query
    let mut stmt = db_connection.prepare(sql.as_str())?;
    let mut rows = stmt.query(&*values.as_params())?;
    let count: i64 = if let Some(row) = rows.next()? {
        row.get_unwrap(0)
    } else {
        0
    };

    Ok(count > 0)
}

/// Returns whether the person manages the entity.
pub fn is_person_manager(
    db_connection: &Connection,
    person_id: u64,
    entity_id: u64,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    debug!("is_person_manager: {person_id:#?} {entity_id:#?}");

    // Create select query to check if person is manager of entity
    let (sql, values) = Query::select()
        .expr(Expr::col(Entitypeople::EntitypeoplePersonId).count())
        .from(Entitypeople::Table)
        .and_where(Expr::col(Entitypeople::EntitypeoplePersonId).eq(person_id))
        .and_where(Expr::col(Entitypeople::EntitypeopleEntityId).eq(entity_id))
        .build_rusqlite(SqliteQueryBuilder);

    debug!("sql: {}", sql.as_str());
    debug!("values: {values:?}");

    // Perform select query
    let mut stmt = db_connection.prepare(sql.as_str())?;
    let mut rows = stmt.query(&*values.as_params())?;
    let count: i64 = if let Some(row) = rows.next()? {
        row.get_unwrap(0)
    } else {
        0
    };

    Ok(count > 0)
}

/// Returns whether the person is a member of the entity.
pub fn is_person_member_of(
    db_connection: &Connection,
    person_id: u64,
    entity_id: u64,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    debug!("is_person_member_of: {person_id:#?} {entity_id:#?}");

    // Create select query to check if person is member of entity
    let (sql, values) = Query::select()
        .expr(Expr::col(Personentities::PersonentitiesPersonId).count())
        .from(Personentities::Table)
        .and_where(Expr::col(Personentities::PersonentitiesPersonId).eq(person_id))
        .and_where(Expr::col(Personentities::PersonentitiesEntityId).eq(entity_id))
        .build_rusqlite(SqliteQueryBuilder);

    debug!("sql: {}", sql.as_str());
    debug!("values: {values:?}");

    // Perform select query
    let mut stmt = db_connection.prepare(sql.as_str())?;
    let mut rows = stmt.query(&*values.as_params())?;
    let count: i64 = if let Some(row) = rows.next()? {
        row.get_unwrap(0)
    } else {
        0
    };

    Ok(count > 0)
}

/// Deletes the person.
pub fn delete_person(
    db_connection: &mut Connection,
    person_id: u64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!("delete_person: {person_id:#?}");

    let (delete_sql, delete_values) = Query::delete()
        .from_table(Person::Table)
        .and_where(Expr::col(Person::PersonId).eq(person_id))
        .build_rusqlite(SqliteQueryBuilder);

    _ = db_connection.execute(delete_sql.as_str(), &*delete_values.as_params())?;

    Ok(())
}

/// Returns whether a person other than `except_person_id` has the email
/// `person_email`. Pass the id of the person being updated, or `None` for a
/// new one.
///
/// # Errors
///
/// Returns an error if the query fails.
pub fn person_email_exists(
    db_connection: &Connection,
    person_email: &str,
    except_person_id: Option<u64>,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    Ok(db_connection.query_row(
        "SELECT EXISTS (SELECT 1 FROM person WHERE person_email = ?1 AND person_id IS NOT ?2)",
        rusqlite::params![person_email, except_person_id],
        |row| row.get(0),
    )?)
}

/// Returns the ids of the entities the person manages, sorted.
///
/// # Errors
///
/// Returns an error if the query fails.
pub fn managed_entity_ids(
    db_connection: &Connection,
    person_id: u64,
) -> Result<Vec<u64>, Box<dyn std::error::Error + Send + Sync>> {
    let mut stmt = db_connection.prepare(
        "SELECT entitypeople_entity_id FROM entitypeople WHERE entitypeople_person_id = ?1 ORDER BY 1",
    )?;
    let entity_ids = stmt
        .query_map([person_id], |row| row.get(0))?
        .collect::<Result<Vec<u64>, _>>()?;
    Ok(entity_ids)
}

/// Returns the stored permissions of the person.
///
/// # Errors
///
/// Returns an error if the query fails or a stored permission is invalid.
pub fn person_permissions(
    db_connection: &Connection,
    person_id: u64,
) -> Result<Vec<PermissionStruct>, Box<dyn std::error::Error + Send + Sync>> {
    let mut stmt = db_connection.prepare(
        "SELECT person_id, person_email, permission_name, permission_item, permission_entity
         FROM permission JOIN person ON person.person_id = permission.person
         WHERE permission.person = ?1",
    )?;
    let mut rows = stmt.query([person_id])?;
    let mut permissions = Vec::new();
    while let Some(row) = rows.next()? {
        permissions.push(PermissionWrapper::try_from(row)?.0);
    }
    Ok(permissions)
}

/// Lists the ids a person save writes, for
/// [`find_missing_reference`](crate::reference::find_missing_reference).
///
/// Member and managed entities must have an id: a managed entity without one
/// would be saved as a permission on all entities. A permission may have no
/// entity, meaning all entities.
#[must_use]
pub fn person_references(person: &PersonStruct) -> Vec<Reference> {
    let mut references = vec![];

    if person.person_id.is_some() {
        references.push(Reference::new("person", "person", person.person_id));
    }
    for entity in person.entities.iter().flatten() {
        references.push(Reference::new("entities", "entity", entity.entity_id));
    }
    for entity in person.managed_entities.iter().flatten() {
        references.push(Reference::new(
            "managed_entities",
            "entity",
            entity.entity_id,
        ));
    }
    for permission in person.permissions.iter().flatten() {
        if permission.permission_entity.is_some() {
            references.push(Reference::new(
                "permissions",
                "entity",
                permission.permission_entity,
            ));
        }
    }

    references
}

#[cfg(test)]
#[path = "person_tests.rs"]
mod person_tests;
