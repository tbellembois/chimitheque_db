use crate::searchable::Searchable;
use serde::Serialize;

#[derive(Debug, Serialize, Default)]
pub struct EmpiricalformulaStruct {
    pub match_exact_search: bool,
    pub empiricalformula_id: u64,
    pub empiricalformula_label: String,
}

impl Searchable for EmpiricalformulaStruct {
    fn new(&self) -> Self {
        EmpiricalformulaStruct {
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
        String::from("empiricalformula")
    }

    fn get_id_field_name(&self) -> String {
        String::from("empiricalformula_id")
    }

    fn set_id_field(&mut self, id: u64) {
        self.empiricalformula_id = id;
    }

    fn get_text_field_name(&self) -> String {
        String::from("empiricalformula_label")
    }

    fn set_text_field(&mut self, text: &str) {
        self.empiricalformula_label = text.to_string();
    }

    fn get_id(&self) -> u64 {
        self.empiricalformula_id
    }

    fn get_text(&self) -> String {
        self.empiricalformula_label.clone()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::searchable::tests::test_searchable;

    #[test]
    fn test_get_empiricalformulas() {
        test_searchable(
            EmpiricalformulaStruct {
                ..Default::default()
            },
            vec![
                "empiricalformula1",
                "aa empiricalformula1",
                "bb eMpIrIcAlFoRmULa1",
                "empiricalformula2",
                "empiricalformula3",
            ],
            3,
            "empiricalformula1",
        )
    }
}
