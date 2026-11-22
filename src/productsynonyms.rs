use chimitheque_types::productsynonyms::Productsynonyms as ProductsynonymsStruct;
use rusqlite::Row;
use sea_query::Iden;
use serde::Serialize;

/// Table and column names of the `productsynonyms` table, for `sea_query`.
#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Productsynonyms {
    /// The table.
    Table,
    /// The `productsynonyms_product_id` column.
    ProductsynonymsProductId,
    /// The `productsynonyms_name_id` column.
    ProductsynonymsNameId,
}

/// A productsynonyms read from a database row.
#[derive(Debug, Serialize, Default)]
pub struct ProductsynonymsWrapper(pub ProductsynonymsStruct);

impl From<&Row<'_>> for ProductsynonymsWrapper {
    fn from(row: &Row) -> Self {
        Self({
            ProductsynonymsStruct {
                productsynonyms_product_id: row.get_unwrap("productsynonyms_product_id"),
                productsynonyms_name_id: row.get_unwrap("productsynonyms_name_id"),
                productsynonyms_name_label: row.get_unwrap("name_label"),
            }
        })
    }
}
