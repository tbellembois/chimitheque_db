use chimitheque_types::category::Category as CategoryStruct;
use sea_query::Iden;
use serde::Serialize;

/// Table and column names of the `category` table, for `sea_query`.
#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Category {
    /// The table.
    Table,
    /// The `category_id` column.
    CategoryId,
    /// The `category_label` column.
    CategoryLabel,
}

/// A category read from a database row.
#[derive(Debug, Serialize, Default)]
pub struct CategoryWrapper(pub CategoryStruct);
