use chimitheque_types::productprecautionarystatements::Productprecautionarystatements as ProductprecautionarystatementsStruct;
use log::debug;
use rusqlite::Row;
use sea_query::Iden;
use serde::Serialize;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Productsynonyms {
    Table,
    ProductsynonymsProductId,
    ProductsynonymsNameId,
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
                productprecautionarystatements_precautionarystatement_id: row
                    .get_unwrap("productprecautionarystatements_precautionarystatement_id"),
                productprecautionarystatements_precautionarystatement_label: row
                    .get_unwrap("precautionarystatement_label"),
                productprecautionarystatements_precautionarystatement_reference: row
                    .get_unwrap("precautionarystatement_reference"),
            }
        })
    }
}
