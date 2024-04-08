use crate::searchable::Searchable;
use serde::Serialize;

#[derive(Debug, Serialize, Default)]
pub struct SymbolStruct {
    pub match_exact_search: bool,
    pub symbol_id: u64,
    pub symbol_label: String,
}

impl Searchable for SymbolStruct {
    fn new(&self) -> Self {
        SymbolStruct {
            ..Default::default()
        }
    }

    fn set_exact_search(&mut self, match_exact_search: bool) {
        self.match_exact_search = match_exact_search;
    }

    fn get_exact_search(&self) -> bool {
        self.match_exact_search
    }

    fn get_table_name(&self) -> String {
        String::from("symbol")
    }

    fn get_id_field_name(&self) -> String {
        String::from("symbol_id")
    }

    fn set_id_field(&mut self, id: u64) {
        self.symbol_id = id;
    }

    fn get_text_field_name(&self) -> String {
        String::from("symbol_label")
    }

    fn set_text_field(&mut self, text: &str) {
        self.symbol_label = text.to_string();
    }

    fn get_id(&self) -> u64 {
        self.symbol_id
    }

    fn get_text(&self) -> String {
        self.symbol_label.clone()
    }
}

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
