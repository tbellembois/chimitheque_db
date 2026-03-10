use chimitheque_types::physicalstate::PhysicalState as PhysicalStateStruct;
use sea_query::Iden;
use serde::Serialize;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum PhysicalState {
    Table,
    PhysicalStateId,
    PhysicalStateLabel,
}

#[derive(Debug, Serialize, Default)]
pub struct PhysicalStateWrapper(pub PhysicalStateStruct);
