use chimitheque_types::producthazardstatements::Producthazardstatements as ProducthazardstatementsStruct;
use log::debug;
use rusqlite::Row;
use sea_query::Iden;
use serde::Serialize;

/// Table and column names of the `producthazardstatements` table, for `sea_query`.
#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Producthazardstatements {
    /// The table.
    Table,
    /// The `producthazardstatements_product_id` column.
    ProducthazardstatementsProductId,
    /// The `producthazardstatements_hazard_statement_id` column.
    ProducthazardstatementsHazardStatementId,
}

/// A producthazardstatements read from a database row.
#[derive(Debug, Serialize, Default)]
pub struct ProducthazardstatementsWrapper(pub ProducthazardstatementsStruct);

impl From<&Row<'_>> for ProducthazardstatementsWrapper {
    fn from(row: &Row) -> Self {
        debug!("row:{row:?}");

        Self({
            ProducthazardstatementsStruct {
                producthazardstatements_product_id: row
                    .get_unwrap("producthazardstatements_product_id"),
                producthazardstatements_hazard_statement_id: row
                    .get_unwrap("producthazardstatements_hazard_statement_id"),
                producthazardstatements_hazard_statement_label: row
                    .get_unwrap("hazard_statement_label"),
                producthazardstatements_hazard_statement_reference: row
                    .get_unwrap("hazard_statement_reference"),
                producthazardstatements_hazard_statement_cmr: row
                    .get_unwrap("hazard_statement_cmr"),
            }
        })
    }
}
