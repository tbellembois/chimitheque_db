use chimitheque_types::productclassofcompound::Productclassofcompound as ProductclassofcompoundStruct;
use rusqlite::Row;
use sea_query::Iden;
use serde::Serialize;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Productclassofcompound {
    Table,
    ProductclassofcompoundProductId,
    ProductclassofcompoundClassofcompoundId,
}

#[derive(Debug, Serialize, Default)]
pub struct ProductclassofcompoundWrapper(pub ProductclassofcompoundStruct);

impl From<&Row<'_>> for ProductclassofcompoundWrapper {
    fn from(row: &Row) -> Self {
        Self({
            ProductclassofcompoundStruct {
                productclassofcompound_product_id: row
                    .get_unwrap("productclassofcompound_product_id"),
                productclassofcompound_classofcompound_id: row
                    .get_unwrap("productclassofcompound_classofcompound_id"),
                productclassofcompound_classofcompound_label: row
                    .get_unwrap("classofcompound_label"),
            }
        })
    }
}
