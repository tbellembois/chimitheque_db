use crate::searchable::Searchable;
use chimitheque_types::symbol::Symbol;
use serde::Serialize;

#[derive(Debug, Serialize, Default)]
pub struct SymbolWrapper(pub Symbol);

impl Searchable for SymbolWrapper {
    fn new(&self) -> Self {
        SymbolWrapper {
            ..Default::default()
        }
    }

    fn set_exact_search(&mut self, match_exact_search: bool) {
        self.0.match_exact_search = match_exact_search;
    }

    fn get_exact_search(&self) -> bool {
        self.0.match_exact_search
    }

    fn get_table_name(&self) -> String {
        String::from("symbol")
    }

    fn get_id_field_name(&self) -> String {
        String::from("symbol_id")
    }

    fn set_id_field(&mut self, id: u64) {
        self.0.symbol_id = id;
    }

    fn get_text_field_name(&self) -> String {
        String::from("symbol_label")
    }

    fn set_text_field(&mut self, text: &str) {
        self.0.symbol_label = text.to_string();
    }

    fn get_id(&self) -> u64 {
        self.0.symbol_id
    }

    fn get_text(&self) -> String {
        self.0.symbol_label.clone()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::searchable::tests::test_searchable;

    #[test]
    fn test_get_symbols() {
        test_searchable(
            SymbolWrapper {
                ..Default::default()
            },
            vec!["symbol1", "aa symbol1", "bb sYmBoL1", "symbol2", "symbol3"],
            3,
            "symbol1",
        )
    }
}
