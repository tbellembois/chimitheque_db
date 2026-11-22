use chimitheque_types::tag::Tag as TagStruct;
use sea_query::Iden;
use serde::Serialize;

/// Table and column names of the `tag` table, for `sea_query`.
#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Tag {
    /// The table.
    Table,
    /// The `tag_id` column.
    TagId,
    /// The `tag_label` column.
    TagLabel,
}

/// A tag read from a database row.
#[derive(Debug, Serialize, Default)]
pub struct TagWrapper(pub TagStruct);
