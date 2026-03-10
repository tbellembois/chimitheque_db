use chimitheque_types::name::Name as NameStruct;
use sea_query::Iden;
use serde::Serialize;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Name {
    Table,
    NameId,
    NameLabel,
}

#[derive(Debug, Serialize, Default)]
pub struct NameWrapper(pub NameStruct);
