use crate::searchable::Searchable;
use serde::Serialize;

#[derive(Debug, Serialize, Default)]
pub struct CasnumberStruct {
    pub match_exact_search: bool,
    pub casnumber_id: u64,
    pub casnumber_label: String,
}

impl Searchable for CasnumberStruct {
    fn new(&self) -> Self {
        CasnumberStruct {
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
        String::from("casnumber")
    }

    fn get_id_field_name(&self) -> String {
        String::from("casnumber_id")
    }

    fn set_id_field(&mut self, id: u64) {
        self.casnumber_id = id;
    }

    fn get_text_field_name(&self) -> String {
        String::from("casnumber_label")
    }

    fn set_text_field(&mut self, text: &str) {
        self.casnumber_label = text.to_string();
    }

    fn get_id(&self) -> u64 {
        self.casnumber_id
    }

    fn get_text(&self) -> String {
        self.casnumber_label.clone()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::searchable::tests::test_searchable;

    #[test]
    fn test_get_casnumbers() {
        test_searchable(
            CasnumberStruct {
                ..Default::default()
            },
            vec![
                "casnumber1",
                "aa casnumber1",
                "bb cAsNuMbEr1",
                "casnumber2",
                "casnumber3",
            ],
            3,
            "casnumber1",
        )
    }
}
