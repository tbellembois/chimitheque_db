use chimitheque_types::productclassesofcompounds::Productclassesofcompounds as ProductclassesofcompoundsStruct;
use rusqlite::Row;
use sea_query::Iden;
use serde::Serialize;

/// Table and column names of the `productclassesofcompounds` table, for `sea_query`.
#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Productclassesofcompounds {
    /// The table.
    Table,
    /// The `productclassesofcompounds_product_id` column.
    ProductclassesofcompoundsProductId,
    /// The `productclassesofcompounds_class_of_compound_id` column.
    ProductclassesofcompoundsClassOfCompoundId,
}

/// A productclassesofcompounds read from a database row.
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
