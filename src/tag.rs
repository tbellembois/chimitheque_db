use chimitheque_types::tag::Tag as TagStruct;
use sea_query::Iden;
use serde::Serialize;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Tag {
    Table,
    TagId,
    TagLabel,
}

#[derive(Debug, Serialize, Default)]
pub struct TagWrapper(pub TagStruct);
