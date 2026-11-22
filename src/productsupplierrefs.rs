use chimitheque_types::productsupplierrefs::Productsupplierrefs as ProductsupplierrefsStruct;
use log::debug;
use rusqlite::Row;
use sea_query::Iden;
use serde::Serialize;

/// Table and column names of the `productsupplierrefs` table, for `sea_query`.
#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Productsupplierrefs {
    /// The table.
    Table,
    /// The `productsupplierrefs_product_id` column.
    ProductsupplierrefsProductId,
    /// The `productsupplierrefs_supplier_ref_id` column.
    ProductsupplierrefsSupplierRefId,
    /// The `productsupplierrefs_supplier_ref_label` column.
    ProductsupplierrefsSupplierRefLabel,
    /// The `productsupplierrefs_supplier_id` column.
    ProductsupplierrefsSupplierId,
    /// The `productsupplierrefs_supplier_label` column.
    ProductsupplierrefsSupplierLabel,
}

/// A productsupplierrefs read from a database row.
#[derive(Debug, Serialize, Default)]
pub struct ProductsupplierrefsWrapper(pub ProductsupplierrefsStruct);

impl From<&Row<'_>> for ProductsupplierrefsWrapper {
    fn from(row: &Row) -> Self {
        debug!("row:{row:?}");

        Self({
            ProductsupplierrefsStruct {
                productsupplierrefs_product_id: row.get_unwrap("productsupplierrefs_product_id"),
                productsupplierrefs_supplier_ref_id: row
                    .get_unwrap("productsupplierrefs_supplier_ref_id"),
                productsupplierrefs_supplier_ref_label: row.get_unwrap("supplier_ref_label"),
                productsupplierrefs_supplier_id: row.get_unwrap("supplier_id"),
                productsupplierrefs_supplier_label: row.get_unwrap("supplier_label"),
            }
        })
    }
}
