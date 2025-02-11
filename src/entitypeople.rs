use chimitheque_types::entitypeople::Entitypeople as EntitypeopleStruct;
use rusqlite::Row;
use sea_query::Iden;
use serde::Serialize;

// Entity managers.
#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Entitypeople {
    Table,
    EntitypeopleEntityId,
    EntitypeoplePersonId,
}

#[derive(Debug, Serialize, Default)]
pub struct EntitypeopleWrapper(pub EntitypeopleStruct);

impl From<&Row<'_>> for EntitypeopleWrapper {
    fn from(row: &Row) -> Self {
        Self({
            EntitypeopleStruct {
                entitypeople_entity_id: row.get_unwrap("entitypeople_entity_id"),
                entitypeople_person_id: row.get_unwrap("entitypeople_person_id"),
                entitypeople_person_email: row.get_unwrap("person_email"),
            }
        })
    }
}
