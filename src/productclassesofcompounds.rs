use chimitheque_types::productclassesofcompounds::Productclassesofcompounds as ProductclassesofcompoundsStruct;
use rusqlite::Row;
use sea_query::Iden;
use serde::Serialize;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Productclassesofcompounds {
    Table,
    ProductclassesofcompoundsProductId,
    ProductclassesofcompoundsClassOfCompoundId,
}

#[derive(Debug, Serialize, Default)]
pub struct ProductclassesofcompoundsWrapper(pub ProductclassesofcompoundsStruct);

impl From<&Row<'_>> for ProductclassesofcompoundsWrapper {
    fn from(row: &Row) -> Self {
        Self({
            ProductclassesofcompoundsStruct {
                productclassesofcompounds_product_id: row
                    .get_unwrap("productclassesofcompounds_product_id"),
                productclassesofcompounds_class_of_compound_id: row
                    .get_unwrap("productclassesofcompounds_class_of_compound_id"),
                productclassesofcompounds_class_of_compound_label: row
                    .get_unwrap("class_of_compound_label"),
            }
        })
    }
}
