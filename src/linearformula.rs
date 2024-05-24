use crate::searchable::Searchable;
use chimitheque_types::linearformula::Linearformula;
use serde::Serialize;

#[derive(Debug, Serialize, Default)]
pub struct LinearformulaWrapper(pub Linearformula);

impl Searchable for LinearformulaWrapper {
    fn new(&self) -> Self {
        LinearformulaWrapper {
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
        String::from("linearformula")
    }

    fn get_id_field_name(&self) -> String {
        String::from("linearformula_id")
    }

    fn set_id_field(&mut self, id: u64) {
        self.0.linearformula_id = id;
    }

    fn get_text_field_name(&self) -> String {
        String::from("linearformula_label")
    }

    fn set_text_field(&mut self, text: &str) {
        self.0.linearformula_label = text.to_string();
    }

    fn get_id(&self) -> u64 {
        self.0.linearformula_id
    }

    fn get_text(&self) -> String {
        self.0.linearformula_label.clone()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::searchable::tests::test_searchable;

    #[test]
    fn test_get_linearformulas() {
        test_searchable(
            LinearformulaWrapper {
                ..Default::default()
            },
            vec![
                "linearformula1",
                "aa linearformula1",
                "bb lInEaRFoRmULa1",
                "linearformula2",
                "linearformula3",
            ],
            3,
            "linearformula1",
        )
    }
}
