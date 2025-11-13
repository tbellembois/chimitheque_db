use chimitheque_types::{
    permission::{Permission as PermissionStruct, PermissionItem, PermissionName},
    person::Person as PersonStruct,
    requestfilter::RequestFilter,
};
use log::debug;
use rusqlite::{Connection, Row, Transaction};
use sea_query::{Alias, Expr, Iden, JoinType, Order, Query, SimpleExpr, SqliteQueryBuilder};
use sea_query_rusqlite::{RusqliteBinder, RusqliteValues};
use serde::Serialize;

use crate::{
    entity::Entity,
    entitypeople::{Entitypeople, EntitypeopleWrapper},
    permission::{Permission, PermissionWrapper},
    personentities::{Personentities, PersonentitiesWrapper},
};

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Person {
    Table,
    PersonId,
    PersonEmail,
}

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
) -> Result<(), Box<dyn std::error::Error>> {
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

        debug!("sql: {}", sql.clone().as_str());
        debug!("values: {:?}", values);

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

        if !entities.is_empty() {
            person.entities = Some(entities);
        } else {
            person.entities = None;
        }
    }

    Ok(())
}

fn populate_managed_entities(
    db_connection: &Connection,
    people: &mut [PersonStruct],
) -> Result<(), Box<dyn std::error::Error>> {
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

        debug!("sql: {}", sql.clone().as_str());
        debug!("values: {:?}", values);

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

        if !entities.is_empty() {
            person.managed_entities = Some(entities);
        } else {
            person.managed_entities = None;
        }
    }

    Ok(())
}

fn populate_permissions(
    db_connection: &Connection,
    people: &mut [PersonStruct],
) -> Result<(), Box<dyn std::error::Error>> {
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

        debug!("sql: {}", sql.clone().as_str());
        debug!("values: {:?}", values);

        // Perform select query.
        let mut stmt = db_connection.prepare(sql.as_str())?;
        let mut rows = stmt.query(&*values.as_params())?;
        let mut permissions = Vec::new();
        while let Some(row) = rows.next()? {
            let permission = PermissionWrapper::try_from(row)?;
            permissions.push(permission.0);
        }

        if !permissions.is_empty() {
            person.permissions = Some(permissions);
        } else {
            person.permissions = None;
        }
    }

    Ok(())
}

pub fn get_people(
    db_connection: &Connection,
    filter: RequestFilter,
    person_id: u64,
) -> Result<(Vec<PersonStruct>, usize), Box<dyn std::error::Error>> {
    debug!("filter:{:?}", filter);
    debug!("person_id:{:?}", person_id);

    let order = if filter.order.eq_ignore_ascii_case("desc") {
        Order::Desc
    } else {
        Order::Asc
    };

    // Create common query statement.
    let mut expression = Query::select();
    expression
        .from(Person::Table)
        //
        // entity -> permissions
        //
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
                            Expr::col((Alias::new("perm"), Alias::new("permission_entity"))).eq(-1),
                        ),
                ),
        )
        //
        // filters
        //
        .conditions(
            filter.entity.is_some(),
            |q| {
                q.and_where(
                    Expr::col((
                        Personentities::Table,
                        Personentities::PersonentitiesEntityId,
                    ))
                    .eq(filter.entity.unwrap()),
                );
            },
            |_| {},
        )
        .conditions(
            filter.id.is_some(),
            |q| {
                q.and_where(Expr::col((Person::Table, Person::PersonId)).eq(filter.id.unwrap()));
            },
            |_| {},
        )
        .conditions(
            filter.person_email.is_some(),
            |q| {
                q.and_where(
                    Expr::col((Person::Table, Person::PersonEmail))
                        .eq(filter.person_email.unwrap()),
                );
            },
            |_| {},
        )
        .conditions(
            filter.search.is_some(),
            |q| {
                q.and_where(
                    Expr::col((Person::Table, Person::PersonEmail))
                        .like(format!("%{}%", filter.search.clone().unwrap())),
                );
            },
            |_| {},
        );

    // Create count query.
    let (count_sql, count_values) = expression
        .clone()
        .expr(Expr::col((Person::Table, Person::PersonId)).count_distinct())
        .build_rusqlite(SqliteQueryBuilder);

    debug!("count_sql: {}", count_sql.clone().as_str());
    debug!("count_values: {:?}", count_values);

    // Create select query.
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
                            Expr::col((Permission::Table, Permission::PermissionEntity)).eq(-1),
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
        .order_by(Person::PersonEmail, order)
        .group_by_col((Person::Table, Person::PersonId))
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
        Ok(PersonWrapper::from(row))
    })?;

    // Build select result.
    let mut people = Vec::new();
    for maybe_person in rows {
        let person = maybe_person?;

        people.push(person.0);
    }

    populate_entities(db_connection, &mut people)?;
    populate_managed_entities(db_connection, &mut people)?;
    populate_permissions(db_connection, &mut people)?;

    debug!("people: {:#?}", people);

    Ok((people, count))
}

fn create_update_person_permissions(
    db_transaction: &Transaction,
    person: &PersonStruct,
) -> Result<(), Box<dyn std::error::Error>> {
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
        permission_entity: -1,
    });

    // Appending permissions for the managed entities.
    if let Some(managed_entities) = &person.managed_entities {
        for entity in managed_entities {
            new_permissions.push(PermissionStruct {
                person: PersonStruct {
                    person_id: person.person_id,
                    ..Default::default()
                },
                permission_name: PermissionName::All,
                permission_item: PermissionItem::All,
                permission_entity: entity.entity_id.unwrap() as i64,
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

        let sql_values: RusqliteValues = RusqliteValues(vec![]);
        let sql_query = Query::insert()
            .replace()
            .into_table(Permission::Table)
            .columns(columns)
            .values(values)?
            .to_string(SqliteQueryBuilder);

        debug!("sql_query: {}", sql_query.clone().as_str());
        debug!("sql_values: {:?}", sql_values);

        _ = db_transaction.execute(&sql_query, &*sql_values.as_params())?;
    }

    Ok(())
}

fn create_update_person_membership(
    db_transaction: &Transaction,
    person: &PersonStruct,
) -> Result<(), Box<dyn std::error::Error>> {
    // Lazily deleting former membership.
    let (delete_sql, delete_values) = Query::delete()
        .from_table(Personentities::Table)
        .and_where(Expr::col(Personentities::PersonentitiesPersonId).eq(person.person_id))
        .build_rusqlite(SqliteQueryBuilder);

    _ = db_transaction.execute(delete_sql.as_str(), &*delete_values.as_params())?;

    // Inserting new membership.
    if let Some(entities) = &person.entities {
        for entity in entities {
            let mut values: Vec<SimpleExpr> = vec![];
            let columns = [
                Personentities::PersonentitiesPersonId,
                Personentities::PersonentitiesEntityId,
            ];

            values.push(person.person_id.into());
            values.push(entity.entity_id.into());

            let sql_values: RusqliteValues = RusqliteValues(vec![]);
            let sql_query = Query::insert()
                .into_table(Personentities::Table)
                .columns(columns)
                .values(values)?
                .to_string(SqliteQueryBuilder);

            debug!("sql_query: {}", sql_query.clone().as_str());
            debug!("sql_values: {:?}", sql_values);

            _ = db_transaction.execute(&sql_query, &*sql_values.as_params())?;
        }
    }
    Ok(())
}

pub fn create_update_person(
    db_connection: &mut Connection,
    mut person: PersonStruct,
) -> Result<u64, Box<dyn std::error::Error>> {
    debug!("create_update_person: {:#?}", person);

    let db_transaction = db_connection.transaction()?;

    // Create request: list of columns and values to insert.
    let mut columns = vec![Person::PersonEmail];
    let mut values = vec![SimpleExpr::Value(person.person_email.clone().into())];

    let sql_query: String;
    let sql_values: RusqliteValues = RusqliteValues(vec![]);

    if let Some(person_id) = person.person_id {
        // Update query.
        columns.push(Person::PersonId);
        values.push(SimpleExpr::Value(person_id.into()));

        sql_query = Query::insert()
            .replace()
            .into_table(Person::Table)
            .columns(columns)
            .values(values)?
            .to_string(SqliteQueryBuilder);
    } else {
        // Insert query.
        sql_query = Query::insert()
            .into_table(Person::Table)
            .columns(columns)
            .values(values)?
            .to_string(SqliteQueryBuilder);
    }

    debug!("sql_query: {}", sql_query.clone().as_str());
    debug!("sql_values: {:?}", sql_values);

    _ = db_transaction.execute(&sql_query, &*sql_values.as_params())?;

    let last_insert_update_id: u64;

    if let Some(person_id) = person.person_id {
        last_insert_update_id = person_id;
    } else {
        last_insert_update_id = db_transaction.last_insert_rowid().try_into()?;
        person.person_id = Some(last_insert_update_id);
    }

    debug!("last_insert_update_id: {}", last_insert_update_id);

    create_update_person_permissions(&db_transaction, &person)?;
    create_update_person_membership(&db_transaction, &person)?;

    db_transaction.commit()?;

    Ok(last_insert_update_id)
}

pub fn unset_person_manager(
    db_transaction: &Transaction,
    person_id: u64,
    entity_id: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("unset_person_manager: {:#?} {:#?}", person_id, entity_id);

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

pub fn set_person_manager(
    db_transaction: &Transaction,
    person_id: u64,
    entity_id: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("set_person_manager: {:#?} {:#?}", person_id, entity_id);

    // Adding the manager to the entity.
    let columns = vec![
        Personentities::PersonentitiesPersonId,
        Personentities::PersonentitiesEntityId,
    ];
    let values = vec![
        SimpleExpr::Value(person_id.into()),
        SimpleExpr::Value(entity_id.into()),
    ];

    let sql_values: RusqliteValues = RusqliteValues(vec![]);
    let sql_query = Query::insert()
        .replace()
        .into_table(Personentities::Table)
        .columns(columns)
        .values(values)?
        .to_string(SqliteQueryBuilder);

    debug!("sql_query: {}", sql_query.clone().as_str());
    debug!("sql_values: {:?}", sql_values);

    _ = db_transaction.execute(&sql_query, &*sql_values.as_params())?;

    // Setting the person manager.
    let columns = vec![
        Entitypeople::EntitypeoplePersonId,
        Entitypeople::EntitypeopleEntityId,
    ];
    let values = vec![
        SimpleExpr::Value(person_id.into()),
        SimpleExpr::Value(entity_id.into()),
    ];

    let sql_values: RusqliteValues = RusqliteValues(vec![]);
    let sql_query = Query::insert()
        .replace()
        .into_table(Entitypeople::Table)
        .columns(columns)
        .values(values)?
        .to_string(SqliteQueryBuilder);

    debug!("sql_query: {}", sql_query.clone().as_str());
    debug!("sql_values: {:?}", sql_values);

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

    let sql_values: RusqliteValues = RusqliteValues(vec![]);
    let sql_query = Query::insert()
        .replace()
        .into_table(Permission::Table)
        .columns(columns)
        .values(values)?
        .to_string(SqliteQueryBuilder);

    debug!("sql_query: {}", sql_query.clone().as_str());
    debug!("sql_values: {:?}", sql_values);

    _ = db_transaction.execute(&sql_query, &*sql_values.as_params())?;

    Ok(())
}

pub fn get_admins(
    db_connection: &mut Connection,
) -> Result<Vec<PersonStruct>, Box<dyn std::error::Error>> {
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
                .and(Expr::col((Alias::new("perm"), Alias::new("permission_entity"))).eq(-1)),
        )
        .order_by(Person::PersonEmail, Order::Asc)
        .group_by_col((Person::Table, Person::PersonId))
        .build_rusqlite(SqliteQueryBuilder);

    debug!("select_sql: {}", select_sql.clone().as_str());
    debug!("select_values: {:?}", select_values);

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

pub fn set_person_admin(
    db_connection: &mut Connection,
    person_id: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("set_person_admin: {:#?}", person_id);

    let columns = vec![
        Permission::PermissionItem,
        Permission::PermissionName,
        Permission::PermissionEntity,
        Permission::Person,
    ];
    let values = vec![
        SimpleExpr::Value("all".into()),
        SimpleExpr::Value("all".into()),
        SimpleExpr::Value((-1 as i64).into()),
        SimpleExpr::Value(person_id.into()),
    ];

    let sql_values: RusqliteValues = RusqliteValues(vec![]);
    let sql_query = Query::insert()
        .replace()
        .into_table(Permission::Table)
        .columns(columns)
        .values(values)?
        .to_string(SqliteQueryBuilder);

    debug!("sql_query: {}", sql_query.clone().as_str());
    debug!("sql_values: {:?}", sql_values);

    _ = db_connection.execute(&sql_query, &*sql_values.as_params())?;

    Ok(())
}

pub fn unset_person_admin(
    db_connection: &mut Connection,
    person_id: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("unset_person_admin: {:#?}", person_id);

    let (delete_sql, delete_values) = Query::delete()
        .from_table(Permission::Table)
        .and_where(Expr::col(Permission::Person).eq(person_id))
        .and_where(Expr::col(Permission::PermissionItem).eq("all"))
        .and_where(Expr::col(Permission::PermissionName).eq("all"))
        .and_where(Expr::col(Permission::PermissionEntity).eq(-1))
        .build_rusqlite(SqliteQueryBuilder);

    _ = db_connection.execute(delete_sql.as_str(), &*delete_values.as_params())?;

    Ok(())
}

pub fn delete_person(
    db_connection: &mut Connection,
    person_id: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("delete_person: {:#?}", person_id);

    let (delete_sql, delete_values) = Query::delete()
        .from_table(Person::Table)
        .and_where(Expr::col(Person::PersonId).eq(person_id))
        .build_rusqlite(SqliteQueryBuilder);

    _ = db_connection.execute(delete_sql.as_str(), &*delete_values.as_params())?;

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
    fn test_get_people() {
        init_logger();

        let mut db_connection = init_test_db();

        info!("testing total result");
        let filter = RequestFilter {
            ..Default::default()
        };
        let (people, count) = get_people(&db_connection, filter, 1).unwrap();
        info!("people: {:?}", people);
        info!("count: {:?}", count);

        assert_eq!(count, 5);

        info!("testing get admins");
        let maybe_admins = get_admins(&mut db_connection);
        assert!(maybe_admins.is_ok());
        info!("maybe_admins: {:?}", maybe_admins);
    }
}
