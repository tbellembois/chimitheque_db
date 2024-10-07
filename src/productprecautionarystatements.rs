use chimitheque_types::productprecautionarystatements::Productprecautionarystatements as ProductprecautionarystatementsStruct;
use log::debug;
use rusqlite::Row;
use sea_query::Iden;
use serde::Serialize;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Productprecautionarystatements {
    Table,
    ProductprecautionarystatementsProductId,
    ProductprecautionarystatementsPrecautionaryStatementId,
}

#[derive(Debug, Serialize, Default)]
pub struct ProductprecautionarystatementsWrapper(pub ProductprecautionarystatementsStruct);

impl From<&Row<'_>> for ProductprecautionarystatementsWrapper {
    fn from(row: &Row) -> Self {
        debug!("row:{:?}", row);

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
