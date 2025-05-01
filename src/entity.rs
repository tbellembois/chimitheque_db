use chimitheque_types::{entity::Entity as EntityStruct, requestfilter::RequestFilter};
use log::debug;
use rusqlite::{Connection, Row};
use sea_query::{
    Alias, ColumnRef, Expr, Iden, IntoColumnRef, JoinType, Order, Query, SqliteQueryBuilder,
};

use sea_query_rusqlite::RusqliteBinder;
use serde::Serialize;

use crate::{
    entitypeople::{Entitypeople, EntitypeopleWrapper},
    permission::Permission,
    person::Person,
    personentities::Personentities,
    storelocation::StoreLocation,
};

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
                entity_nb_store_locations: row.get_unwrap("entity_nb_store_locations"),
                entity_nb_people: row.get_unwrap("entity_nb_people"),
            }
        })
    }
}

fn populate_managers(
    db_connection: &Connection,
    entity: &mut [EntityStruct],
) -> Result<(), Box<dyn std::error::Error>> {
    for entity in entity.iter_mut() {
        let entity_id = entity.entity_id;

        // Create select query.
        let (sql, values) = Query::select()
            .columns([
                Entitypeople::EntitypeopleEntityId,
                Entitypeople::EntitypeoplePersonId,
            ])
            .column(Person::PersonEmail)
            .column(Entity::EntityName)
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
                Expr::col((Entitypeople::Table, Entitypeople::EntitypeoplePersonId))
                    .equals((Person::Table, Person::PersonId)),
            )
            .and_where(Expr::col(Entitypeople::EntitypeopleEntityId).eq(entity_id))
            .build_rusqlite(SqliteQueryBuilder);

        debug!("sql: {}", sql.clone().as_str());
        debug!("values: {:?}", values);

        // Perform select query.
        let mut stmt = db_connection.prepare(sql.as_str())?;
        let rows = stmt.query_map(&*values.as_params(), |row| {
            Ok(EntitypeopleWrapper::from(row))
        })?;

        // Populate entity managers.
        let mut managers: Vec<chimitheque_types::person::Person> = vec![];
        for row in rows {
            let entity_person_wrapper = row?;
            managers.push(chimitheque_types::person::Person {
                person_id: entity_person_wrapper.0.entitypeople_person_id,
                person_email: entity_person_wrapper.0.entitypeople_person_email,
                entities: None,
                managed_entities: None,
                permissions: None,
            });
        }

        if !managers.is_empty() {
            entity.managers = Some(managers);
        } else {
            entity.managers = None;
        }
    }

    Ok(())
}

pub fn get_entities(
    db_connection: &Connection,
    filter: RequestFilter,
    person_id: u64,
) -> Result<(Vec<EntityStruct>, usize), Box<dyn std::error::Error>> {
    debug!("filter:{:?}", filter);
    debug!("person_id:{:?}", person_id);

    let order_by: ColumnRef = (Entity::Table, Entity::EntityName).into_column_ref();

    let order = if filter.order.eq_ignore_ascii_case("desc") {
        Order::Desc
    } else {
        Order::Asc
    };

    // Create common query statement.
    let mut expression = Query::select();
    expression
        .from(Entity::Table)
        //
        // store locations for nb_store_locations
        //
        .join(
            JoinType::LeftJoin,
            StoreLocation::Table,
            Expr::col((StoreLocation::Table, StoreLocation::Entity))
                .equals((Entity::Table, Entity::EntityId)),
        )
        //
        // person for nb_people
        //
        .join(
            JoinType::LeftJoin,
            Personentities::Table,
            Expr::col((
                Personentities::Table,
                Personentities::PersonentitiesEntityId,
            ))
            .equals((Entity::Table, Entity::EntityId)),
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
                        .is_in(["all", "entities"]),
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
                    Expr::col((Entity::Table, Entity::EntityName))
                        .like(format!("%{}%", filter.search.clone().unwrap())),
                );
            },
            |_| {},
        )
        .conditions(
            filter.entity_name.is_some(),
            |q| {
                q.and_where(
                    Expr::col((Entity::Table, Entity::EntityName)).eq(filter.entity_name.unwrap()),
                );
            },
            |_| {},
        )
        .conditions(
            filter.id.is_some(),
            |q| {
                q.and_where(Expr::col(Entity::EntityId).eq(filter.id.unwrap()));
            },
            |_| {},
        );

    // Create count query.
    let (count_sql, count_values) = expression
        .clone()
        .expr(Expr::col((Entity::Table, Entity::EntityId)).count_distinct())
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
        .expr_as(
            Expr::col(StoreLocation::StoreLocationId).count_distinct(),
            Alias::new("entity_nb_store_locations"),
        )
        .expr_as(
            Expr::col(Personentities::PersonentitiesPersonId).count_distinct(),
            Alias::new("entity_nb_people"),
        )
        .order_by(order_by, order)
        .group_by_col((Entity::Table, Entity::EntityId))
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
        Ok(EntityWrapper::from(row))
    })?;

    // Build select result.
    let mut entities = Vec::new();
    for maybe_entity in rows {
        let entiity = maybe_entity?;

        entities.push(entiity.0);
    }

    populate_managers(db_connection, &mut entities)?;

    debug!("entities: {:#?}", entities);

    Ok((entities, count))
}
