use chimitheque_types::cenumber::CeNumber as CeNumberStruct;
use sea_query::Iden;
use serde::Serialize;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum CeNumber {
    Table,
    CeNumberId,
    CeNumberLabel,
}

#[derive(Debug, Serialize, Default)]
pub struct CeNumberWrapper(pub CeNumberStruct);
