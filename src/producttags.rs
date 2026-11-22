use chimitheque_types::producttags::Producttags as ProducttagsStruct;
use rusqlite::Row;
use sea_query::Iden;
use serde::Serialize;

/// Table and column names of the `producttags` table, for `sea_query`.
#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Producttags {
    /// The table.
    Table,
    /// The `producttags_product_id` column.
    ProducttagsProductId,
    /// The `producttags_tag_id` column.
    ProducttagsTagId,
}

/// A producttags read from a database row.
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
