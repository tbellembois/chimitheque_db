use chimitheque_types::symbol::Symbol as SymbolStruct;
use sea_query::Iden;
use serde::Serialize;

/// Table and column names of the `symbol` table, for `sea_query`.
#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Symbol {
    /// The table.
    Table,
    /// The `symbol_id` column.
    SymbolId,
    /// The `symbol_label` column.
    SymbolLabel,
}

/// A symbol read from a database row.
#[derive(Debug, Serialize, Default)]
pub struct SymbolWrapper(pub SymbolStruct);
