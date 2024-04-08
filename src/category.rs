use crate::searchable::Searchable;
use serde::Serialize;

#[derive(Debug, Serialize, Default)]
pub struct CategoryStruct {
    pub match_exact_search: bool,
    pub category_id: u64,
    pub category_label: String,
}

impl Searchable for CategoryStruct {
    fn new(&self) -> Self {
        CategoryStruct {
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
        String::from("category")
    }

    fn get_id_field_name(&self) -> String {
        String::from("category_id")
    }

    fn set_id_field(&mut self, id: u64) {
        self.category_id = id;
    }

    fn get_text_field_name(&self) -> String {
        String::from("category_label")
    }

    fn set_text_field(&mut self, text: &str) {
        self.category_label = text.to_string();
    }

    fn get_id(&self) -> u64 {
        self.category_id
    }

    fn get_text(&self) -> String {
        self.category_label.clone()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::searchable::tests::test_searchable;

    #[test]
    fn test_get_categories() {
        test_searchable(
            CategoryStruct {
                ..Default::default()
            },
            vec![
                "category1",
                "aa category1",
                "bb cAtEgOrY1",
                "category2",
                "category3",
            ],
            3,
            "category1",
        )
    }
}
