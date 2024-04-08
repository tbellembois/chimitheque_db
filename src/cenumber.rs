use crate::searchable::Searchable;
use serde::Serialize;

#[derive(Debug, Serialize, Default)]
pub struct CenumberStruct {
    pub match_exact_search: bool,
    pub cenumber_id: u64,
    pub cenumber_label: String,
}

impl Searchable for CenumberStruct {
    fn new(&self) -> Self {
        CenumberStruct {
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
        String::from("cenumber")
    }

    fn get_id_field_name(&self) -> String {
        String::from("cenumber_id")
    }

    fn set_id_field(&mut self, id: u64) {
        self.cenumber_id = id;
    }

    fn get_text_field_name(&self) -> String {
        String::from("cenumber_label")
    }

    fn set_text_field(&mut self, text: &str) {
        self.cenumber_label = text.to_string();
    }

    fn get_id(&self) -> u64 {
        self.cenumber_id
    }

    fn get_text(&self) -> String {
        self.cenumber_label.clone()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::searchable::tests::test_searchable;

    #[test]
    fn test_get_cenumbers() {
        test_searchable(
            CenumberStruct {
                ..Default::default()
            },
            vec![
                "cenumber1",
                "aa cenumber1",
                "bb cENuMbEr1",
                "cenumber2",
                "cenumber3",
            ],
            3,
            "cenumber1",
        )
    }
}
