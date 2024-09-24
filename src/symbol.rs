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

#[cfg(test)]
mod tests {

    use super::*;
    use crate::searchable::tests::test_searchable;

    #[test]
    fn test_get_symbols() {
        test_searchable(
            SymbolStruct {
                ..Default::default()
            },
            vec!["symbol1", "aa symbol1", "bb sYmBoL1", "symbol2", "symbol3"],
            3,
            "symbol1",
        )
    }
}
