use chimitheque_types::personentities::Personentities as PersonentitiesStruct;
use rusqlite::Row;
use sea_query::Iden;
use serde::Serialize;

// Person entities.
/// Table and column names of the `personentities` table, for `sea_query`.
#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Personentities {
    /// The table.
    Table,
    /// The `personentities_person_id` column.
    PersonentitiesPersonId,
    /// The `personentities_entity_id` column.
    PersonentitiesEntityId,
}

/// A personentities read from a database row.
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
