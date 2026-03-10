use chimitheque_types::category::Category as CategoryStruct;
use sea_query::Iden;
use serde::Serialize;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Category {
    Table,
    CategoryId,
    CategoryLabel,
}

#[derive(Debug, Serialize, Default)]
pub struct CategoryWrapper(pub CategoryStruct);
