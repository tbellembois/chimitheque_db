use chimitheque_types::cenumber::CeNumber as CeNumberStruct;
use sea_query::Iden;
use serde::Serialize;

/// Table and column names of the `ce_number` table, for `sea_query`.
#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum CeNumber {
    /// The table.
    Table,
    /// The `ce_number_id` column.
    CeNumberId,
    /// The `ce_number_label` column.
    CeNumberLabel,
}

/// A ce number read from a database row.
#[derive(Debug, Serialize, Default)]
pub struct CeNumberWrapper(pub CeNumberStruct);
