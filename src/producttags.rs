use chimitheque_types::producttags::Producttags as ProducttagsStruct;
use rusqlite::Row;
use sea_query::Iden;
use serde::Serialize;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Producttags {
    Table,
    ProducttagsProductId,
    ProducttagsTagId,
}

#[derive(Debug, Serialize, Default)]
pub struct ProducttagsWrapper(pub ProducttagsStruct);

impl From<&Row<'_>> for ProducttagsWrapper {
    fn from(row: &Row) -> Self {
        Self({
            ProducttagsStruct {
                producttags_product_id: row.get_unwrap("producttags_product_id"),
                producttags_tag_id: row.get_unwrap("producttags_tag_id"),
                producttags_tag_label: row.get_unwrap("tag_label"),
            }
        })
    }
}
