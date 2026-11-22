use chimitheque_types::casnumber::CasNumber as CasNumberStruct;
use sea_query::Iden;
use serde::Serialize;

/// Table and column names of the `cas_number` table, for `sea_query`.
#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum CasNumber {
    /// The table.
    Table,
    /// The `cas_number_id` column.
    CasNumberId,
    /// The `cas_number_label` column.
    CasNumberLabel,
    /// The `cas_number_cmr` column.
    CasNumberCmr,
}

/// A cas number read from a database row.
#[derive(Debug, Serialize, Default)]
pub struct CasNumberWrapper(pub CasNumberStruct);
