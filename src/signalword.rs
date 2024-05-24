use crate::searchable::Searchable;
use chimitheque_types::signalword::Signalword;
use serde::Serialize;

#[derive(Debug, Serialize, Default)]
pub struct SignalwordWrapper(pub Signalword);

impl Searchable for SignalwordWrapper {
    fn new(&self) -> Self {
        SignalwordWrapper {
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
        String::from("signalword")
    }

    fn get_id_field_name(&self) -> String {
        String::from("signalword_id")
    }

    fn set_id_field(&mut self, id: u64) {
        self.0.signalword_id = id;
    }

    fn get_text_field_name(&self) -> String {
        String::from("signalword_label")
    }

    fn set_text_field(&mut self, text: &str) {
        self.0.signalword_label = text.to_string();
    }

    fn get_id(&self) -> u64 {
        self.0.signalword_id
    }

    fn get_text(&self) -> String {
        self.0.signalword_label.clone()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::searchable::tests::test_searchable;

    #[test]
    fn test_get_signalwords() {
        test_searchable(
            SignalwordWrapper {
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
