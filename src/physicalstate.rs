use chimitheque_types::physicalstate::PhysicalState as PhysicalStateStruct;
use sea_query::Iden;
use serde::Serialize;

/// Table and column names of the `physical_state` table, for `sea_query`.
#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum PhysicalState {
    /// The table.
    Table,
    /// The `physical_state_id` column.
    PhysicalStateId,
    /// The `physical_state_label` column.
    PhysicalStateLabel,
}

/// A physical state read from a database row.
#[derive(Debug, Serialize, Default)]
pub struct PhysicalStateWrapper(pub PhysicalStateStruct);
