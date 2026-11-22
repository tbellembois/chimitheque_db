use chimitheque_types::productprecautionarystatements::Productprecautionarystatements as ProductprecautionarystatementsStruct;
use log::debug;
use rusqlite::Row;
use sea_query::Iden;
use serde::Serialize;

/// Table and column names of the `productprecautionarystatements` table, for `sea_query`.
#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Productprecautionarystatements {
    /// The table.
    Table,
    /// The `productprecautionarystatements_product_id` column.
    ProductprecautionarystatementsProductId,
    /// The `productprecautionarystatements_precautionary_statement_id` column.
    ProductprecautionarystatementsPrecautionaryStatementId,
}

/// A productprecautionarystatements read from a database row.
#[derive(Debug, Serialize, Default)]
pub struct ProductprecautionarystatementsWrapper(pub ProductprecautionarystatementsStruct);

impl From<&Row<'_>> for ProductprecautionarystatementsWrapper {
    fn from(row: &Row) -> Self {
        debug!("row:{row:?}");

        Self({
            ProductprecautionarystatementsStruct {
                productprecautionarystatements_product_id: row
                    .get_unwrap("productprecautionarystatements_product_id"),
                productprecautionarystatements_precautionary_statement_id: row
                    .get_unwrap("productprecautionarystatements_precautionary_statement_id"),
                productprecautionarystatements_precautionary_statement_label: row
                    .get_unwrap("precautionary_statement_label"),
                productprecautionarystatements_precautionary_statement_reference: row
                    .get_unwrap("precautionary_statement_reference"),
            }
        })
    }
}
