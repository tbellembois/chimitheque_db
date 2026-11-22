use chimitheque_types::signalword::SignalWord as SignalWordStruct;
use sea_query::Iden;
use serde::Serialize;

/// Table and column names of the `signal_word` table, for `sea_query`.
#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum SignalWord {
    /// The table.
    Table,
    /// The `signal_word_id` column.
    SignalWordId,
    /// The `signal_word_label` column.
    SignalWordLabel,
}

/// A signalword read from a database row.
#[derive(Debug, Serialize, Default)]
pub struct SignalwordWrapper(pub SignalWordStruct);
