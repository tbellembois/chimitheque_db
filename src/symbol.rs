use chimitheque_types::symbol::Symbol;
use serde::Serialize;

#[derive(Debug, Serialize, Default)]
pub struct SymbolWrapper(pub Symbol);

#[cfg(test)]
mod tests {

    use super::*;
    use crate::searchable::tests::test_searchable;

    #[test]
    fn test_get_symbols() {
        test_searchable(
            Symbol {
                ..Default::default()
            },
            vec!["symbol1", "aa symbol1", "bb sYmBoL1", "symbol2", "symbol3"],
            3,
            "symbol1",
        )
    }
}
