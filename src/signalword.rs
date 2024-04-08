use crate::searchable::Searchable;
use serde::Serialize;

#[derive(Debug, Serialize, Default)]
pub struct SignalwordStruct {
    pub match_exact_search: bool,
    pub signalword_id: u64,
    pub signalword_label: String,
}

impl Searchable for SignalwordStruct {
    fn new(&self) -> Self {
        SignalwordStruct {
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
        String::from("signalword")
    }

    fn get_id_field_name(&self) -> String {
        String::from("signalword_id")
    }

    fn set_id_field(&mut self, id: u64) {
        self.signalword_id = id;
    }

    fn get_text_field_name(&self) -> String {
        String::from("signalword_label")
    }

    fn set_text_field(&mut self, text: &str) {
        self.signalword_label = text.to_string();
    }

    fn get_id(&self) -> u64 {
        self.signalword_id
    }

    fn get_text(&self) -> String {
        self.signalword_label.clone()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::searchable::tests::test_searchable;

    #[test]
    fn test_get_signalwords() {
        test_searchable(
            SignalwordStruct {
                ..Default::default()
            },
            vec![
                "signalword1",
                "aa signalword1",
                "bb sIgNaLwOrD1",
                "signalword2",
                "signalword3",
            ],
            3,
            "signalword1",
        )
    }
}
