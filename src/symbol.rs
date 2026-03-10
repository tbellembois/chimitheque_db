use chimitheque_types::symbol::Symbol as SymbolStruct;
use sea_query::Iden;
use serde::Serialize;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Symbol {
    Table,
    SymbolId,
    SymbolLabel,
}

#[derive(Debug, Serialize, Default)]
pub struct SymbolWrapper(pub SymbolStruct);
