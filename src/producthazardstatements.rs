use chimitheque_types::producthazardstatements::Producthazardstatements as ProducthazardstatementsStruct;
use log::debug;
use rusqlite::Row;
use sea_query::Iden;
use serde::Serialize;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Producthazardstatements {
    Table,
    ProducthazardstatementsProductId,
    ProducthazardstatementsHazardstatementId,
}

#[derive(Debug, Serialize, Default)]
pub struct ProducthazardstatementsWrapper(pub ProducthazardstatementsStruct);

impl From<&Row<'_>> for ProducthazardstatementsWrapper {
    fn from(row: &Row) -> Self {
        debug!("row:{:?}", row);

        Self({
            ProducthazardstatementsStruct {
                producthazardstatements_product_id: row
                    .get_unwrap("producthazardstatements_product_id"),
                producthazardstatements_hazardstatement_id: row
                    .get_unwrap("producthazardstatements_hazardstatement_id"),
                producthazardstatements_hazardstatement_label: row
                    .get_unwrap("hazardstatement_label"),
                producthazardstatements_hazardstatement_reference: row
                    .get_unwrap("hazardstatement_reference"),
                producthazardstatements_hazardstatement_cmr: row.get_unwrap("hazardstatement_cmr"),
            }
        })
    }
}
