use chimitheque_types::entity::Entity as EntityStruct;
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
pub struct EntityWrapper(pub EntityStruct);
