use chimitheque_types::name::Name as NameStruct;
use sea_query::Iden;
use serde::Serialize;

/// Table and column names of the `name` table, for `sea_query`.
#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Name {
    /// The table.
    Table,
    /// The `name_id` column.
    NameId,
    /// The `name_label` column.
    NameLabel,
}

/// A name read from a database row.
#[derive(Debug, Serialize, Default)]
pub struct NameWrapper(pub NameStruct);
