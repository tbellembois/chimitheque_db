use chimitheque_types::casnumber::CasNumber as CasNumberStruct;
use sea_query::Iden;
use serde::Serialize;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum CasNumber {
    Table,
    CasNumberId,
    CasNumberLabel,
    CasNumberCmr,
}

#[derive(Debug, Serialize, Default)]
pub struct CasNumberWrapper(pub CasNumberStruct);
