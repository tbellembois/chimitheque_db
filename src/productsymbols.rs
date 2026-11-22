use chimitheque_types::productsymbols::Productsymbols as ProductsymbolsStruct;
use rusqlite::Row;
use sea_query::Iden;
use serde::Serialize;

/// Table and column names of the `productsymbols` table, for `sea_query`.
#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Productsymbols {
    /// The table.
    Table,
    /// The `productsymbols_product_id` column.
    ProductsymbolsProductId,
    /// The `productsymbols_symbol_id` column.
    ProductsymbolsSymbolId,
}

/// A productsymbols read from a database row.
#[derive(Debug, Serialize, Default)]
pub struct ProductsymbolsWrapper(pub ProductsymbolsStruct);

impl From<&Row<'_>> for ProductsymbolsWrapper {
    fn from(row: &Row) -> Self {
        Self({
            ProductsymbolsStruct {
                productsymbols_product_id: row.get_unwrap("productsymbols_product_id"),
                productsymbols_symbol_id: row.get_unwrap("productsymbols_symbol_id"),
                productsymbols_symbol_label: row.get_unwrap("symbol_label"),
            }
        })
    }
}
