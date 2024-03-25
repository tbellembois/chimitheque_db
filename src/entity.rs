use sea_query::Iden;
use serde::Serialize;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Entity {
    Table,
    EntityId,
    EntityName,
}

#[derive(Debug, Serialize)]
pub struct EntityStruct {
    pub(crate) entity_id: u64,
    pub(crate) entity_name: String,
}
