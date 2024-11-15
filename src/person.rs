use chimitheque_types::{person::Person as PersonStruct, requestfilter::RequestFilter};
use log::debug;
use rusqlite::{Connection, Row};
use sea_query::{Alias, Expr, Iden, JoinType, Order, Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use serde::Serialize;

use crate::{permission::Permission, personentities::Personentities};

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
            }
        })
    }
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
                    Expr::col((Alias::new("perm"), Alias::new("permission_item_name")))
                        .is_in(["all", "people"]),
                )
                .and(
                    Expr::col((Alias::new("perm"), Alias::new("permission_perm_name")))
                        .is_in(["r", "w", "all"]),
                )
                .and(
                    Expr::col((Alias::new("perm"), Alias::new("permission_entity_id")))
                        .equals(Personentities::PersonentitiesEntityId)
                        .or(
                            Expr::col((Alias::new("perm"), Alias::new("permission_entity_id")))
                                .eq(-1),
                        ),
                ),
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

        assert!(true);
    }
}
