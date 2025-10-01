use chimitheque_types::{person::Person as PersonStruct, requestfilter::RequestFilter};
use log::debug;
use rusqlite::{Connection, Row};
use sea_query::{Alias, Expr, Iden, JoinType, Order, Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
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

#[cfg(test)]
mod tests {

    use super::*;
    use crate::init::{init_db, insert_fake_values};
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
    fn test_get_people() {
        init_logger();

        let mut db_connection = init_test_db();
        insert_fake_values(&mut db_connection).unwrap();

        info!("testing total result");
        let filter = RequestFilter {
            ..Default::default()
        };
        let (people, count) = get_people(&db_connection, filter, 1).unwrap();
        info!("people: {:?}", people);
        info!("count: {:?}", count);

        todo!()
    }
}
