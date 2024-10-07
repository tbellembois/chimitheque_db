use chimitheque_types::productsupplierrefs::Productsupplierrefs as ProductsupplierrefsStruct;
use log::debug;
use rusqlite::Row;
use sea_query::Iden;
use serde::Serialize;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Productsupplierrefs {
    Table,
    ProductsupplierrefsProductId,
    ProductsupplierrefsSupplierRefId,
    ProductsupplierrefsSupplierRefLabel,
    ProductsupplierrefsSupplierId,
    ProductsupplierrefsSupplierLabel,
}

#[derive(Debug, Serialize, Default)]
pub struct ProductsupplierrefsWrapper(pub ProductsupplierrefsStruct);

impl From<&Row<'_>> for ProductsupplierrefsWrapper {
    fn from(row: &Row) -> Self {
        debug!("row:{:?}", row);

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
