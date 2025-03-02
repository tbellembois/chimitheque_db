use chimitheque_types::personentities::Personentities as PersonentitiesStruct;
use rusqlite::Row;
use sea_query::Iden;
use serde::Serialize;

// Person entities.
#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Personentities {
    Table,
    PersonentitiesPersonId,
    PersonentitiesEntityId,
}

#[derive(Debug, Serialize, Default)]
pub struct PersonentitiesWrapper(pub PersonentitiesStruct);

impl From<&Row<'_>> for PersonentitiesWrapper {
    fn from(row: &Row) -> Self {
        Self({
            PersonentitiesStruct {
                personentities_person_id: row.get_unwrap("personentities_person_id"),
                personentities_entity_id: row.get_unwrap("personentities_entity_id"),
                personentities_entity_name: row.get_unwrap("entity_name"),
                personentities_person_email: row.get_unwrap("person_email"),
            }
        })
    }
}
