use chimitheque_types::productsymbols::Productsymbols as ProductsymbolsStruct;
use rusqlite::Row;
use sea_query::Iden;
use serde::Serialize;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Productsymbols {
    Table,
    ProductsymbolsProductId,
    ProductsymbolsSymbolId,
}

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
